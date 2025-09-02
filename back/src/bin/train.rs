use env_logger;
use std::time::Instant;

use candle_core::{DType, Result as CandleResult, Tensor};
use candle_nn::{AdamW, Optimizer, ParamsAdamW};
use catan::players::nn::encoder;
use catan::players::nn::encoder::encode_state_tensor;
use catan::players::nn::loader::{device_auto, load_latest_weights_path, try_load, try_save};
use catan::players::nn::model;
use catan::players::nn::model::AlphaZeroNet;
use catan::players::nn::self_play::{
    generate_self_play_games, generate_self_play_games_parallel, Experience,
};
use rand::prelude::*;

// Hyperparameters (kept in code, no CLI flags)
const SELF_PLAY_GAMES: usize = 8; // run 8 games in parallel for speed
const BATCH_SIZE: usize = 32;
const LEARNING_RATE: f32 = 1e-3;
const WEIGHT_DECAY: f32 = 1e-4; // currently unused; add explicit L2 if needed
const USE_PARALLEL_SELF_PLAY: bool = true;
const REPLAY_CAPACITY: usize = 2000;
const UPDATES_PER_ITER: usize = 100;

struct ReplayBuffer {
    data: Vec<Experience>,
    cap: usize,
}

impl ReplayBuffer {
    fn new(cap: usize) -> Self {
        Self {
            data: Vec::with_capacity(cap),
            cap,
        }
    }
    fn push_many(&mut self, mut v: Vec<Experience>) {
        if self.data.len() + v.len() > self.cap {
            let overflow = self.data.len() + v.len() - self.cap;
            if overflow > 0 && overflow <= self.data.len() {
                self.data.drain(0..overflow);
            }
        }
        self.data.append(&mut v);
    }
    fn sample(&self, n: usize) -> Vec<Experience> {
        if self.data.is_empty() {
            return Vec::new();
        }
        let mut rng = thread_rng();
        (0..n)
            .map(|_| {
                let idx = rng.gen_range(0..self.data.len());
                self.data[idx].clone()
            })
            .collect()
    }
    fn len(&self) -> usize {
        self.data.len()
    }
}

fn main() {
    env_logger::init();
    println!("[train] start: Training entrypoint");

    // 1) Initialize model & device, try load weights
    let device = device_auto();
    let mut net = AlphaZeroNet::new(device).expect("init net");
    let weights_path = load_latest_weights_path();
    let loaded = try_load(&mut net.varmap, &weights_path);
    println!("[train] weights: loaded={} path={} ", loaded, weights_path);
    // 2) Generate self-play data (single-threaded skeleton)
    println!("[train] self-play: games={}", SELF_PLAY_GAMES);
    let sp_t0 = Instant::now();
    let mut experiences = if USE_PARALLEL_SELF_PLAY {
        println!("[train] self-play mode: parallel");
        generate_self_play_games_parallel(SELF_PLAY_GAMES)
    } else {
        println!("[train] self-play mode: single-threaded");
        generate_self_play_games(SELF_PLAY_GAMES)
    };
    let sp_ms = sp_t0.elapsed().as_millis();
    // Trim for plumbing validation so runs are fast
    if experiences.len() > 64 {
        experiences.truncate(64);
    }
    println!(
        "[train] self-play: collected experiences={} in {}ms",
        experiences.len(),
        sp_ms
    );

    // Build replay buffer
    let mut replay = ReplayBuffer::new(REPLAY_CAPACITY);
    replay.push_many(experiences);

    // 3) Training: a few mini-epochs over the small sample for fast validation
    println!(
        "[train] training: batch_size={} lr={} weight_decay={}",
        BATCH_SIZE, LEARNING_RATE, WEIGHT_DECAY
    );
    let device = &net.device;
    let mut opt = AdamW::new(
        net.varmap.all_vars(),
        ParamsAdamW {
            lr: LEARNING_RATE as f64,
            ..Default::default()
        },
    )
    .expect("adamw");

    let epochs = 3usize;
    let tr_t0 = Instant::now();
    let mut fwd_ms_acc: u128 = 0;
    let mut bwd_ms_acc: u128 = 0;
    for epoch in 0..epochs {
        let mut idx = 0;
        while idx < replay.len() {
            let end = (idx + BATCH_SIZE).min(replay.len());
            let batch = replay.sample(end - idx);
            idx = end;

            // Encode states and stack [B, C, H, W]
            let xs: Vec<Tensor> = batch
                .iter()
                .map(|ex| encode_state_tensor(&ex.state, device))
                .collect::<Result<Vec<_>, _>>()
                .expect("encode batch");
            let value_targets: Vec<f32> = batch.iter().map(|ex| ex.value_target).collect();
            let input = Tensor::stack(&xs, 0).expect("stack"); // [B,C,H,W]
            let batch_idx = (idx / BATCH_SIZE).saturating_sub(1);

            // Batched forward
            let f0 = Instant::now();
            let (embeds_b, values_b) = net.forward_embed_batch(&input).expect("fwd_b");
            fwd_ms_acc += f0.elapsed().as_millis();
            let pred_v = values_b; // [B,1]

            // Compute value loss MSE
            let target_v =
                Tensor::from_vec(value_targets.clone(), (value_targets.len(), 1), device)
                    .expect("vt");
            let diff = (&pred_v - &target_v).expect("diff");
            let mse = diff.sqr().unwrap().mean_all().unwrap();

            // Compute policy loss: sum over samples CE between predicted logits over K (via fused head) and visit-count distribution
            let mut policy_losses: Vec<Tensor> = Vec::new();
            for (i, ex) in batch.iter().enumerate() {
                let k = ex.policy_target.len().max(1);
                if k == 0 {
                    continue;
                }
                // Action features [K, AF]
                let af = encoder::action_features(
                    &ex.policy_target.iter().map(|(a, _)| *a).collect::<Vec<_>>(),
                );
                let af_tensor = Tensor::from_vec(
                    af.iter().flatten().copied().collect::<Vec<f32>>(),
                    (k, model::ACTION_FEAT_DIM),
                    device,
                )
                .expect("af");
                // Broadcast state embed -> [K,32]
                let se = embeds_b
                    .get(i)
                    .unwrap()
                    .squeeze(0)
                    .unwrap()
                    .repeat((k, 1))
                    .unwrap();
                let fused = Tensor::cat(&[se, af_tensor], 1).unwrap();
                let logits = net.policy_logits(&fused).unwrap().squeeze(1).unwrap(); // [K]
                let mask = Tensor::ones((k,), DType::F32, device).unwrap();
                let probs = softmax_logits_masked(&logits, &mask).unwrap(); // [K]
                let mut tgt: Vec<f32> = ex
                    .policy_target
                    .iter()
                    .map(|(_, p)| *p)
                    .collect::<Vec<f32>>();
                // Normalize target distribution to sum to 1 to prevent shape/sum errors
                let s: f32 = tgt.iter().copied().sum();
                if s > 0.0 {
                    for v in tgt.iter_mut() {
                        *v /= s;
                    }
                } else {
                    let u = 1.0f32 / (k as f32);
                    for v in tgt.iter_mut() {
                        *v = u;
                    }
                }
                let target = Tensor::from_vec(tgt, (k,), device).unwrap();
                let ce = cross_entropy_from_probs(&probs, &target).unwrap();
                policy_losses.push(ce);
            }
            let policy_loss = Tensor::stack(&policy_losses, 0)
                .unwrap()
                .mean_all()
                .unwrap();

            // Total loss
            let half_scalar = Tensor::from_vec(vec![0.5f32], (), device).expect("half scalar");
            let half = (&mse * &half_scalar).expect("half");
            let total = (&policy_loss + &half).expect("total");
            let b0 = Instant::now();
            opt.backward_step(&total).expect("opt step");
            bwd_ms_acc += b0.elapsed().as_millis();
            let samples_in_batch = batch.len();
            println!(
                "[train] epoch {} batch {}: samples={} (ok)",
                epoch, batch_idx, samples_in_batch
            );
        }
    }
    // Extra updates over replay for better utilization
    for u in 0..UPDATES_PER_ITER {
        let batch = replay.sample(BATCH_SIZE);
        if batch.is_empty() {
            break;
        }
        // Encode states
        let xs: Vec<Tensor> = batch
            .iter()
            .map(|ex| encode_state_tensor(&ex.state, device))
            .collect::<Result<Vec<_>, _>>()
            .expect("encode batch");
        let value_targets: Vec<f32> = batch.iter().map(|ex| ex.value_target).collect();
        let input = Tensor::stack(&xs, 0).expect("stack");
        // Batched forward
        let f0 = Instant::now();
        let (embeds_b, pred_v) = net.forward_embed_batch(&input).expect("fwd_b");
        fwd_ms_acc += f0.elapsed().as_millis();
        let target_v =
            Tensor::from_vec(value_targets.clone(), (value_targets.len(), 1), device).expect("vt");
        let diff = (&pred_v - &target_v).expect("diff");
        let mse = diff.sqr().unwrap().mean_all().unwrap();
        // Optional: simple policy update in extra phase
        let mut policy_losses: Vec<Tensor> = Vec::new();
        for (i, ex) in batch.iter().enumerate() {
            let k = ex.policy_target.len();
            if k == 0 {
                continue;
            }
            let af = encoder::action_features(
                &ex.policy_target.iter().map(|(a, _)| *a).collect::<Vec<_>>(),
            );
            let af_tensor = Tensor::from_vec(
                af.iter().flatten().copied().collect::<Vec<f32>>(),
                (k, model::ACTION_FEAT_DIM),
                device,
            )
            .expect("af");
            let se = embeds_b
                .get(i)
                .unwrap()
                .squeeze(0)
                .unwrap()
                .repeat((k, 1))
                .unwrap();
            let fused = Tensor::cat(&[se, af_tensor], 1).unwrap();
            let logits = net.policy_logits(&fused).unwrap().squeeze(1).unwrap();
            let mask = Tensor::ones((k,), DType::F32, device).unwrap();
            let probs = softmax_logits_masked(&logits, &mask).unwrap();
            let mut tgt: Vec<f32> = ex
                .policy_target
                .iter()
                .map(|(_, p)| *p)
                .collect::<Vec<f32>>();
            let s: f32 = tgt.iter().copied().sum();
            if s > 0.0 {
                for v in tgt.iter_mut() {
                    *v /= s;
                }
            } else {
                let u = 1.0f32 / (k as f32);
                for v in tgt.iter_mut() {
                    *v = u;
                }
            }
            let target = Tensor::from_vec(tgt, (k,), device).unwrap();
            policy_losses.push(cross_entropy_from_probs(&probs, &target).unwrap());
        }
        let half_scalar = Tensor::from_vec(vec![0.5f32], (), device).expect("half scalar");
        let total = (&mse * &half_scalar).expect("half");
        let total = if !policy_losses.is_empty() {
            let pl = Tensor::stack(&policy_losses, 0)
                .unwrap()
                .mean_all()
                .unwrap();
            (&pl + &total).unwrap()
        } else {
            total
        };
        let b0 = Instant::now();
        opt.backward_step(&total).expect("opt step");
        bwd_ms_acc += b0.elapsed().as_millis();
        if u % 25 == 0 {
            println!("[train] extra update {} (ok)", u);
        }
    }
    let tr_ms = tr_t0.elapsed().as_millis();
    println!(
        "[train] time: self_play={}ms, fwd={}ms, bwd={}ms, train_total={}ms",
        sp_ms, fwd_ms_acc, bwd_ms_acc, tr_ms
    );
    // 5) Save checkpoint stub
    let t0 = Instant::now();
    let saved = try_save(&net.varmap, &weights_path);
    println!(
        "[train] saved checkpoint: {} path={} in {}ms",
        saved,
        weights_path,
        t0.elapsed().as_millis()
    );
    println!("[train] done: skeleton ready. Next: implement replay buffer, optimizer step, and parallel self-play.");
}

fn softmax_logits_masked(logits: &Tensor, mask: &Tensor) -> CandleResult<Tensor> {
    // Convert to host for robust shape handling during plumbing
    let lvec: Vec<f32> = logits.to_vec1()?;
    let mvec: Vec<f32> = mask.to_vec1()?;
    let k = lvec.len();
    if k == 0 {
        return Tensor::from_vec(Vec::<f32>::new(), (0,), logits.device());
    }
    // Compute max over masked entries for numerical stability
    let mut max_val = f32::NEG_INFINITY;
    for i in 0..k {
        if mvec[i] > 0.0 && lvec[i] > max_val {
            max_val = lvec[i];
        }
    }
    if !max_val.is_finite() {
        // All masked off: return uniform zeros
        return Tensor::from_vec(vec![0.0; k], (k,), logits.device());
    }
    let mut exps = vec![0f32; k];
    let mut sum = 0f32;
    for i in 0..k {
        if mvec[i] > 0.0 {
            let e = (lvec[i] - max_val).exp();
            exps[i] = e;
            sum += e;
        }
    }
    if sum <= 0.0 {
        let cnt = mvec.iter().filter(|&&v| v > 0.0).count().max(1) as f32;
        let probs: Vec<f32> = mvec
            .iter()
            .map(|&v| if v > 0.0 { 1.0 / cnt } else { 0.0 })
            .collect();
        return Tensor::from_vec(probs, (k,), logits.device());
    }
    for i in 0..k {
        exps[i] /= sum;
    }
    Tensor::from_vec(exps, (k,), logits.device())
}

fn cross_entropy_from_probs(pred: &Tensor, target: &Tensor) -> CandleResult<Tensor> {
    let logp = pred.log()?;
    let mul = (target * logp)?;
    let neg = mul.neg()?;
    Ok(neg.sum(0)?)
}

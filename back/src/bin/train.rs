use env_logger;
use std::time::Instant;

use catan::players::nn::loader::{device_auto, load_latest_weights_path, try_load, try_save};
use catan::players::nn::model::AlphaZeroNet;
use catan::players::nn::self_play::generate_self_play_games;
use catan::players::nn::encoder;
use catan::players::nn::encoder::encode_state_tensor;
use catan::players::nn::model;
use candle_core::{DType, Tensor, Result as CandleResult};

// Hyperparameters (kept in code, no CLI flags)
const SELF_PLAY_GAMES: usize = 1;
const BATCH_SIZE: usize = 32;
const LEARNING_RATE: f32 = 1e-3;
const WEIGHT_DECAY: f32 = 1e-4; // currently unused; add explicit L2 if needed

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
    let mut experiences = generate_self_play_games(SELF_PLAY_GAMES);
    // Trim for plumbing validation so runs are fast
    if experiences.len() > 64 {
        experiences.truncate(64);
    }
    println!("[train] self-play: collected experiences={}", experiences.len());

    // 3) Single pass training over all experiences (skeleton)
    println!("[train] training: batch_size={} lr={} weight_decay={}", BATCH_SIZE, LEARNING_RATE, WEIGHT_DECAY);
    let device = &net.device;
    let mut idx = 0;
    while idx < experiences.len() {
        let end = (idx + BATCH_SIZE).min(experiences.len());
        let batch = &experiences[idx..end];
        idx = end;

        // Encode states and stack [B, C, H, W]
        let mut xs: Vec<Tensor> = Vec::with_capacity(batch.len());
        let mut value_targets: Vec<f32> = Vec::with_capacity(batch.len());
        let mut policy_dists: Vec<Vec<(usize, f32)>> = Vec::with_capacity(batch.len());
        for ex in batch.iter() {
            let t = encode_state_tensor(&ex.state, device).expect("encode");
            xs.push(t);
            value_targets.push(ex.value_target);
            // Convert actions into dummy indices [0..K) per-sample for CE; we’ll weight sample-wise
            let v = ex
                .policy_target
                .iter()
                .enumerate()
                .map(|(i, (_a, p))| (i, *p))
                .collect::<Vec<(usize, f32)>>();
            policy_dists.push(v);
        }
        let _input = Tensor::stack(&xs, 0).expect("stack"); // [B,C,H,W]
        let batch_idx = (idx / BATCH_SIZE).saturating_sub(1);

        // Forward using current API to get state embeddings and values per sample
        let mut pred_values: Vec<Tensor> = Vec::with_capacity(batch.len());
        let mut embeds: Vec<Tensor> = Vec::with_capacity(batch.len());
        for b in 0..batch.len() {
            let (embed, v) = net.forward_embed(&xs[b]).expect("fwd");
            embeds.push(embed.squeeze(0).unwrap()); // [32]
            pred_values.push(v.squeeze(0).unwrap()); // [1]
        }

        // Compute value loss MSE
        let pred_v = Tensor::stack(&pred_values, 0).expect("stack v"); // [B,1]
        let target_v = Tensor::from_vec(value_targets.clone(), (value_targets.len(), 1), device).expect("vt");
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
            let af = encoder::action_features(&ex.policy_target.iter().map(|(a, _)| *a).collect::<Vec<_>>());
            let af_tensor = Tensor::from_vec(af.iter().flatten().copied().collect::<Vec<f32>>(), (k, model::ACTION_FEAT_DIM), device).expect("af");
            // Broadcast state embed -> [K,32]
            let se = embeds[i].repeat((k, 1)).unwrap();
            let fused = Tensor::cat(&[se, af_tensor], 1).unwrap();
            let logits = net.policy_logits(&fused).unwrap().squeeze(1).unwrap(); // [K]
            let mask = Tensor::ones((k,), DType::F32, device).unwrap();
            let probs = softmax_logits_masked(&logits, &mask).unwrap(); // [K]
            let mut tgt: Vec<f32> = ex.policy_target.iter().map(|(_, p)| *p).collect::<Vec<f32>>();
            // Normalize target distribution to sum to 1 to prevent shape/sum errors
            let s: f32 = tgt.iter().copied().sum();
            if s > 0.0 {
                for v in tgt.iter_mut() { *v /= s; }
            } else {
                let u = 1.0f32 / (k as f32);
                for v in tgt.iter_mut() { *v = u; }
            }
            let target = Tensor::from_vec(tgt, (k,), device).unwrap();
            let ce = cross_entropy_from_probs(&probs, &target).unwrap();
            policy_losses.push(ce);
        }
        let policy_loss = Tensor::stack(&policy_losses, 0).unwrap().mean_all().unwrap();

        // Total loss
        let half_scalar = Tensor::from_vec(vec![0.5f32], (), device).expect("half scalar");
        let half = (&mse * &half_scalar).expect("half");
        let total = (&policy_loss + &half).expect("total");
        // TODO: add optimizer step once configured (AdamW/SGD)
        let _ = total; // suppress unused warnings for now
        println!("[train] batch {}: samples={} (ok)", batch_idx, end - (idx - BATCH_SIZE).min(end));
    }
    // 5) Save checkpoint stub
    let t0 = Instant::now();
    let saved = try_save(&net.varmap, &weights_path);
    println!("[train] saved checkpoint: {} path={} in {}ms", saved, weights_path, t0.elapsed().as_millis());
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
        let probs: Vec<f32> = mvec.iter().map(|&v| if v > 0.0 { 1.0 / cnt } else { 0.0 }).collect();
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



use crate::enums::Action;
use crate::players::nn::encoder::{action_features, encode_state_tensor};
use crate::players::nn::loader::{load_latest_weights_path, try_load};
use crate::players::nn::model::{AlphaZeroNet, ACTION_FEAT_DIM};
use crate::state::State;
use candle_core::{Device, Tensor};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, Instant};

pub struct InferenceRequest {
    pub state: State,
    pub legal: Vec<Action>,
    pub reply: Sender<(Vec<(Action, f32)>, f32)>,
}

pub struct InferenceWorker {
    tx: Sender<InferenceRequest>,
}

static GLOBAL_WORKER: OnceLock<InferenceWorker> = OnceLock::new();

impl InferenceWorker {
    pub fn start(device: Device, flush_ms: u64) -> Self {
        let (tx, rx): (Sender<InferenceRequest>, Receiver<InferenceRequest>) = channel();
        thread::spawn(move || worker_loop(device, rx, flush_ms));
        Self { tx }
    }

    pub fn infer(&self, state: State, legal: Vec<Action>) -> (Vec<(Action, f32)>, f32) {
        let (rtx, rrx) = channel();
        let _ = self.tx.send(InferenceRequest {
            state,
            legal,
            reply: rtx,
        });
        // Block until result (non-blocking call sites can offload)
        rrx.recv().unwrap_or((Vec::new(), 0.0))
    }

    pub fn init_global(device: Device, flush_ms: u64) {
        let _ = GLOBAL_WORKER.set(InferenceWorker::start(device, flush_ms));
    }

    pub fn global() -> Option<&'static InferenceWorker> {
        GLOBAL_WORKER.get()
    }
}

fn worker_loop(device: Device, rx: Receiver<InferenceRequest>, flush_ms: u64) {
    let mut net = AlphaZeroNet::new(device.clone()).expect("init net");
    let _ = try_load(&mut net.varmap, &load_latest_weights_path());
    let mut pending: Vec<InferenceRequest> = Vec::new();
    let min_wait = Duration::from_millis(flush_ms);
    loop {
        let t0 = Instant::now();
        // Collect initial request (blocking)
        match rx.recv() {
            Ok(req) => pending.push(req),
            Err(_) => break,
        }
        // Short flush window
        while t0.elapsed() < min_wait {
            match rx.try_recv() {
                Ok(req) => pending.push(req),
                Err(_) => break,
            }
        }
        // Encode batch
        let xs: Vec<Tensor> = pending
            .iter()
            .map(|r| encode_state_tensor(&r.state, &device))
            .collect::<Result<Vec<_>, _>>()
            .expect("encode");
        let input = Tensor::stack(&xs, 0).expect("stack");
        let (embeds_b, values_b) = net.forward_embed_batch(&input).expect("fwd");
        let values: Vec<f32> = values_b.flatten_all().unwrap().to_vec1().unwrap();
        for (i, req) in pending.drain(..).enumerate() {
            let k = req.legal.len().max(1);
            let af = action_features(&req.legal);
            let af_tensor = Tensor::from_vec(
                af.iter().flatten().copied().collect::<Vec<f32>>(),
                (k, ACTION_FEAT_DIM),
                &device,
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
            let probs =
                softmax_logits_masked_host(logits).unwrap_or_else(|| vec![1.0 / k as f32; k]);
            let priors: Vec<(Action, f32)> =
                req.legal.iter().copied().zip(probs.into_iter()).collect();
            let _ = req.reply.send((priors, values[i]));
        }
    }
}

fn softmax_logits_masked_host(logits: Tensor) -> Option<Vec<f32>> {
    let v = logits.to_vec1().ok()?;
    if v.is_empty() {
        return Some(Vec::new());
    }
    let maxv = v.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let mut exps = vec![0f32; v.len()];
    let mut sum = 0f32;
    for (i, &x) in v.iter().enumerate() {
        let e = (x - maxv).exp();
        exps[i] = e;
        sum += e;
    }
    if sum <= 0.0 {
        return None;
    }
    for x in exps.iter_mut() {
        *x /= sum;
    }
    Some(exps)
}

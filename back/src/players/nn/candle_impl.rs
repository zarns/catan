// Enabled: candle 0.9.1 is now a direct dependency.
use super::encoder::{action_features, encode_state_tensor, index_legal_actions};
use super::infer_worker::InferenceWorker;
use super::loader::{load_latest_weights_path, try_load};
use super::model::AlphaZeroNet;
use super::types::{PolicyValue, PolicyValueNet};
use crate::enums::Action;
use crate::state::State;

use candle_core as candle;

/// Placeholder Candle-backed model. This wires a future net but currently returns uniform priors.
pub struct CandleNet {
    device: candle::Device,
    net: AlphaZeroNet,
}

impl CandleNet {
    pub fn new_default_device() -> candle::Result<Self> {
        // Try CUDA first, fallback to CPU
        let device = match candle::Device::new_cuda(0) {
            Ok(d) => d,
            Err(_) => candle::Device::Cpu,
        };
        let mut net = AlphaZeroNet::new(device.clone())?;
        // Print device info only once across threads to reduce log noise
        use std::sync::Once;
        static INIT_DEVICE_LOG: Once = Once::new();
        INIT_DEVICE_LOG.call_once(|| {
            println!("[infer] device={:?}", net.device);
        });
        // Attempt to load weights if present
        let _loaded = try_load(&mut net.varmap, &load_latest_weights_path());
        // Initialize global inference worker for batched calls (5ms flush)
        InferenceWorker::init_global(device.clone(), 5);
        Ok(Self { device, net })
    }
}

impl PolicyValueNet for CandleNet {
    fn infer_policy_value(&self, _state: &State, legal_actions: &[Action]) -> PolicyValue {
        // Minimal end-to-end path: encode to tensor (zeros for now) on the configured device,
        // map legal actions to contiguous indices, and return uniform priors and zero value.
        // Touch the device by running the encoder on it to avoid dead_code warnings.
        let xs = match encode_state_tensor(_state, &self.device) {
            Ok(t) => t,
            Err(_) => {
                // Fallback uniform priors on encode failure
                if legal_actions.is_empty() {
                    return PolicyValue {
                        priors: vec![],
                        value: 0.0,
                    };
                }
                let p = 1.0f32 / (legal_actions.len() as f32);
                return PolicyValue {
                    priors: legal_actions.iter().copied().map(|a| (a, p)).collect(),
                    value: 0.0,
                };
            }
        };

        if legal_actions.is_empty() {
            return PolicyValue {
                priors: vec![],
                value: 0.0,
            };
        }

        // Use global inference worker if available
        if let Some(w) = InferenceWorker::global() {
            let (priors, value) = w.infer(_state.clone(), legal_actions.to_vec());
            return PolicyValue { priors, value };
        }

        // Fallback: local forward path
        // Get pooled state embedding and value
        let (state_embed, value_tensor) = match self.net.forward_embed(&xs) {
            Ok(out) => out,
            Err(_) => {
                if legal_actions.is_empty() {
                    return PolicyValue {
                        priors: vec![],
                        value: 0.0,
                    };
                }
                let p = 1.0f32 / (legal_actions.len() as f32);
                return PolicyValue {
                    priors: legal_actions.iter().copied().map(|a| (a, p)).collect(),
                    value: 0.0,
                };
            }
        };
        let (_map, mask) = index_legal_actions(legal_actions);
        let act_feats = action_features(legal_actions);
        let k = act_feats.len();
        let state_broadcast = state_embed
            .repeat((k as usize, 1))
            .unwrap_or_else(|_| state_embed.clone());
        let act_tensor = candle::Tensor::from_vec(
            act_feats.iter().flatten().copied().collect::<Vec<f32>>(),
            (k, super::model::ACTION_FEAT_DIM),
            &self.device,
        )
        .unwrap();
        let fused = candle::Tensor::cat(&[state_broadcast, act_tensor], 1).unwrap();
        let logits_tensor = self.net.policy_logits(&fused).unwrap_or_else(|_| {
            candle::Tensor::zeros((k, 1), candle::DType::F32, &self.device).unwrap()
        });
        let logits = logits_tensor
            .squeeze(1)
            .unwrap()
            .to_vec1()
            .unwrap_or(vec![0.0; k]);
        let probs = masked_softmax(&logits, &mask);
        let priors = legal_actions
            .iter()
            .copied()
            .zip(probs.into_iter())
            .collect();
        let value_vec: Vec<f32> = value_tensor
            .flatten_all()
            .expect("value flatten")
            .to_vec1()
            .unwrap_or(vec![0.0]);
        let value: f32 = *value_vec.get(0).unwrap_or(&0.0);
        PolicyValue { priors, value }
    }
}

fn masked_softmax(logits: &[f32], mask: &[f32]) -> Vec<f32> {
    if logits.is_empty() {
        return Vec::new();
    }
    let mut max_logit = f32::NEG_INFINITY;
    for (&l, &m) in logits.iter().zip(mask.iter()) {
        if m > 0.0 {
            if l > max_logit {
                max_logit = l;
            }
        }
    }
    let mut exps = vec![0f32; logits.len()];
    let mut sum = 0f32;
    for i in 0..logits.len() {
        if mask[i] > 0.0 {
            let e = (logits[i] - max_logit).exp();
            exps[i] = e;
            sum += e;
        } else {
            exps[i] = 0.0;
        }
    }
    if sum <= 0.0 {
        // fallback to uniform over masked entries
        let k = mask.iter().filter(|&&m| m > 0.0).count().max(1) as f32;
        return mask
            .iter()
            .map(|&m| if m > 0.0 { 1.0 / k } else { 0.0 })
            .collect();
    }
    for e in exps.iter_mut() {
        *e /= sum;
    }
    exps
}

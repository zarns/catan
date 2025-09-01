// Enabled: candle 0.9.1 is now a direct dependency.
use super::types::{PolicyValue, PolicyValueNet};
use crate::enums::Action;
use crate::state::State;

use candle_core as candle;

/// Placeholder Candle-backed model. This wires a future net but currently returns uniform priors.
pub struct CandleNet {
    device: candle::Device,
}

impl CandleNet {
    pub fn new_default_device() -> candle::Result<Self> {
        // Try CUDA first, fallback to CPU
        let device = match candle::Device::new_cuda(0) {
            Ok(d) => d,
            Err(_) => candle::Device::Cpu,
        };
        Ok(Self { device })
    }
}

impl PolicyValueNet for CandleNet {
    fn infer_policy_value(&self, _state: &State, legal_actions: &[Action]) -> PolicyValue {
        // Touch device to avoid unused-field warning until we wire real inference
        let _ = &self.device;
        if legal_actions.is_empty() {
            return PolicyValue {
                priors: vec![],
                value: 0.0,
            };
        }
        // TODO: encode state, run model forward, map logits -> priors with mask, get value
        let p = 1.0f32 / (legal_actions.len() as f32);
        PolicyValue {
            priors: legal_actions.iter().copied().map(|a| (a, p)).collect(),
            value: 0.0,
        }
    }
}

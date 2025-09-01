// Enabled: candle 0.9.1 is now a direct dependency.
use super::types::{PolicyValue, PolicyValueNet};
use super::encoder::{encode_state_tensor, index_legal_actions};
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
        // Minimal end-to-end path: encode to tensor (zeros for now), map legal actions
        // to contiguous indices, and return uniform priors and zero value.
        // This compiles and allows us to replace internals with a real model incrementally.
        let _ = encode_state_tensor; // reference to avoid warning if unused yet

        if legal_actions.is_empty() {
            return PolicyValue { priors: vec![], value: 0.0 };
        }

        let (_map, _mask) = index_legal_actions(legal_actions);
        let p = 1.0f32 / (legal_actions.len() as f32);
        let priors = legal_actions.iter().copied().map(|a| (a, p)).collect();
        PolicyValue { priors, value: 0.0 }
    }
}

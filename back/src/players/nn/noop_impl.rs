use super::types::{PolicyValue, PolicyValueNet};
use crate::enums::Action;
use crate::state::State;

/// A no-op network that produces uniform priors and zero value.
/// Used when the Candle feature is disabled.
pub struct NoopNet;

impl PolicyValueNet for NoopNet {
    fn infer_policy_value(&self, _state: &State, legal_actions: &[Action]) -> PolicyValue {
        if legal_actions.is_empty() {
            return PolicyValue {
                priors: vec![],
                value: 0.0,
            };
        }
        let p = 1.0f32 / (legal_actions.len() as f32);
        PolicyValue {
            priors: legal_actions.iter().copied().map(|a| (a, p)).collect(),
            value: 0.0,
        }
    }
}

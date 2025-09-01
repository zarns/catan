use crate::enums::Action;
use crate::state::State;

/// Output of the policy/value network
#[derive(Debug, Clone)]
pub struct PolicyValue {
    pub priors: Vec<(Action, f32)>,
    pub value: f32, // in [-1, 1]
}

/// Trait describing a network capable of producing policy and value
pub trait PolicyValueNet {
    fn infer_policy_value(&self, state: &State, legal_actions: &[Action]) -> PolicyValue;
}

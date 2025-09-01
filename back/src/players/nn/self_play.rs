use crate::enums::Action;
use crate::players::zero::AlphaZeroPlayer;
use crate::state::State;

#[derive(Clone)]
pub struct Experience {
    pub state: State,
    pub policy_target: Vec<(Action, f32)>,
    pub value_target: f32,
}

/// Placeholder for a self-play generator. Will be fleshed out once training loop lands.
pub fn generate_self_play_game(_player: &AlphaZeroPlayer) -> Vec<Experience> {
    Vec::new()
}



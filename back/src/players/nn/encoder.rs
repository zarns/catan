use crate::enums::Action;
use crate::state::State;

/// Placeholder encoder for future Candle input tensors.
/// For now this only defines the interface and simple helpers.
pub struct EncodedState {
    pub current_player: u8,
    pub hash64: u64,
    pub legal_actions: Vec<Action>,
}

pub fn encode_state(state: &State) -> EncodedState {
    EncodedState {
        current_player: state.get_current_color(),
        hash64: state.compute_hash64(),
        legal_actions: state.generate_playable_actions(),
    }
}

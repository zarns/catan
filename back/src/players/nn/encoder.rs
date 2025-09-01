use crate::enums::Action;
use crate::state::State;

use candle_core as candle;
use candle_core::Tensor;

/// Lightweight struct for non-tensor metadata alongside the tensor encoding.
pub struct EncodedStateMeta {
    pub current_player: u8,
    pub hash64: u64,
    pub legal_actions: Vec<Action>,
}

/// Returns non-tensor metadata commonly used by both search and training.
pub fn encode_state_meta(state: &State) -> EncodedStateMeta {
    EncodedStateMeta {
        current_player: state.get_current_color(),
        hash64: state.compute_hash64(),
        legal_actions: state.generate_playable_actions(),
    }
}

/// Encode the board state into a CNN-friendly tensor.
/// Shape: [C, H, W] = [23, 7, 7]. Currently a minimal scaffold returning zeros with a few
/// cheap feature hints; can be extended incrementally without breaking the API.
pub fn encode_state_tensor(state: &State, device: &candle::Device) -> candle::Result<Tensor> {
    let _ = state; // will be used as features expand
    let channels = 23usize;
    let h = 7usize;
    let w = 7usize;

    // Start with zeros; this compiles and allows us to wire end-to-end inference.
    // We keep the function signature stable so we can incrementally fill channels.
    let zeros = vec![0f32; channels * h * w];
    let t = Tensor::from_vec(zeros, &[channels as u64, h as u64, w as u64], device)?;
    Ok(t)
}

/// Map a slice of legal actions into contiguous indices [0..K) and return both the mapping
/// and a mask (1.0 for legal, 0.0 for illegal) for a provided action list.
pub fn index_legal_actions(legal_actions: &[Action]) -> (Vec<(Action, usize)>, Vec<f32>) {
    let mut mapping: Vec<(Action, usize)> = Vec::with_capacity(legal_actions.len());
    for (idx, &a) in legal_actions.iter().enumerate() {
        mapping.push((a, idx));
    }
    let mask = vec![1.0f32; legal_actions.len()];
    (mapping, mask)
}

use std::collections::HashMap;

use crate::enums::{Action, Resource};
use crate::map_instance::Tile;
use crate::map_template::Coordinate;
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
    const CHANNELS: usize = 23;
    const H: usize = 7;
    const W: usize = 7;

    let mut feat = vec![0f32; CHANNELS * H * W];
    let idx = |c: usize, y: usize, x: usize| -> usize { c * (H * W) + y * W + x };

    // Build lookup: land tile id -> coordinate for spatial placement.
    let map = state.get_map_instance();
    let mut id_to_coord: HashMap<u8, Coordinate> = HashMap::new();
    for (coord, lt) in map.get_land_tiles().iter() {
        id_to_coord.insert(lt.id, *coord);
    }

    // Helper: axial coord to 7x7 grid. Center board around (0,0) -> (3,3).
    let to_grid = |coord: Coordinate| -> Option<(usize, usize)> {
        let (q, r, _s) = coord;
        let gx = (q as i32 + 3).clamp(0, 6) as usize;
        let gy = (r as i32 + 3).clamp(0, 6) as usize;
        Some((gx, gy))
    };

    // 0: desert/water mask; 1..=5: resource one-hot; 7: dice number/12; 8: production prob.
    for (coord, tile) in map.get_tiles().iter() {
        if let Some((x, y)) = to_grid(*coord) {
            match tile {
                Tile::Water(_) => {
                    feat[idx(0, y, x)] = 1.0;
                }
                Tile::Land(land) => {
                    match land.resource {
                        None => {
                            // Desert
                            feat[idx(0, y, x)] = 1.0;
                        }
                        Some(res) => {
                            let ch = 1 + resource_channel(res);
                            feat[idx(ch, y, x)] = 1.0;
                        }
                    }
                    if let Some(num) = land.number {
                        feat[idx(7, y, x)] = (num as f32) / 12.0;
                        feat[idx(8, y, x)] = dice_probability(num);
                    }
                }
                Tile::Port(_port) => {
                    // Harbor location mask into channel 22 for now
                    feat[idx(22, y, x)] = 1.0;
                }
            }
        }
    }

    // Player pieces: settlements (9..12), cities (13..16). Roads left for a later pass.
    let num_players = state.get_num_players().min(4) as usize; // encoder currently assumes 4 max
    for color in 0..num_players {
        // Settlements
        for b in state.get_settlements(color as u8) {
            if let Some(adj) = map.get_adjacent_tiles(match b {
                crate::state::Building::Settlement(_, node) => node,
                _ => 0,
            }) {
                for land in adj {
                    if let Some(&coord) = id_to_coord.get(&land.id) {
                        if let Some((x, y)) = to_grid(coord) {
                            feat[idx(9 + color, y, x)] = 1.0;
                        }
                    }
                }
            }
        }
        // Cities
        for b in state.get_cities(color as u8) {
            if let Some(adj) = map.get_adjacent_tiles(match b {
                crate::state::Building::City(_, node) => node,
                _ => 0,
            }) {
                for land in adj {
                    if let Some(&coord) = id_to_coord.get(&land.id) {
                        if let Some((x, y)) = to_grid(coord) {
                            feat[idx(13 + color, y, x)] = 1.0;
                        }
                    }
                }
            }
        }
    }

    // Robber position mask in channel 21
    let robber_id = state.get_robber_tile();
    if let Some(&coord) = id_to_coord.get(&robber_id) {
        if let Some((x, y)) = to_grid(coord) {
            feat[idx(21, y, x)] = 1.0;
        }
    }

    let t = Tensor::from_vec(feat, (CHANNELS, H, W), device)?;
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

fn resource_channel(res: Resource) -> usize {
    match res {
        Resource::Wood => 0,
        Resource::Brick => 1,
        Resource::Sheep => 2,
        Resource::Wheat => 3,
        Resource::Ore => 4,
    }
}

fn dice_probability(number: u8) -> f32 {
    match number {
        2 | 12 => 1.0 / 36.0,
        3 | 11 => 2.0 / 36.0,
        4 | 10 => 3.0 / 36.0,
        5 | 9 => 4.0 / 36.0,
        6 | 8 => 5.0 / 36.0,
        7 => 0.0, // robber
        _ => 0.0,
    }
}

/// Produce simple fixed-size per-action features for the policy head fusion.
/// Returns [K, ACTION_FEAT_DIM] in row-major flattened vector.
pub fn action_features(
    legal_actions: &[Action],
) -> Vec<[f32; crate::players::nn::model::ACTION_FEAT_DIM]> {
    use crate::players::nn::model::ACTION_FEAT_DIM;
    let mut out = Vec::with_capacity(legal_actions.len());
    for a in legal_actions {
        let mut f = [0f32; ACTION_FEAT_DIM];
        match *a {
            Action::BuildSettlement { .. } => {
                f[0] = 1.0;
            }
            Action::BuildCity { .. } => {
                f[1] = 1.0;
            }
            Action::BuildRoad { .. } => {
                f[2] = 1.0;
            }
            Action::BuyDevelopmentCard { .. } => {
                f[3] = 1.0;
            }
            Action::MoveRobber { .. } => {
                f[4] = 1.0;
            }
            Action::EndTurn { .. } => {
                f[5] = 1.0;
            }
            _ => {
                f[6] = 1.0;
            } // other
        }
        out.push(f);
    }
    out
}

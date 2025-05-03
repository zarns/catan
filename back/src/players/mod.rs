mod random_player;
mod mcts_player;
mod weighted_random_player;

use crate::{enums::Action, state::State};
use log::{debug, warn};

pub use random_player::RandomPlayer;
pub use mcts_player::MctsPlayer;
pub use weighted_random_player::WeightedRandomPlayer;

pub trait Player {
    fn decide(&self, state: &State, playable_actions: &[Action]) -> Action;
}

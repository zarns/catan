mod greedy_player;
mod mcts_player;
mod random_player;
mod weighted_random_player;

use crate::{enums::Action, state::State};
use log::{debug, warn};

pub use greedy_player::GreedyPlayer;
pub use mcts_player::MctsPlayer;
pub use random_player::RandomPlayer;
pub use weighted_random_player::WeightedRandomPlayer;

pub trait Player {
    fn decide(&self, state: &State, playable_actions: &[Action]) -> Action;
}

mod random_player;
mod mcts_player;

use crate::{enums::Action, state::State};
use log::{debug, warn};

pub use random_player::RandomPlayer;
pub use mcts_player::MctsPlayer;

pub trait Player {
    fn decide(&self, state: &State, playable_actions: &[Action]) -> Action;
}

mod random_player;

use crate::{enums::Action, state::State};
use log::{debug, warn};

pub use random_player::RandomPlayer;

pub trait Player {
    fn decide(&self, state: &State, playable_actions: &[Action]) -> Action;
}

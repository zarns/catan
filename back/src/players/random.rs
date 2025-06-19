use rand::prelude::*;

use crate::enums::Action;
use crate::state::State;

pub struct RandomPlayer {
    pub id: String,
    pub name: String,
    pub color: String,
}

impl RandomPlayer {
    pub fn new(id: String, name: String, color: String) -> Self {
        RandomPlayer { id, name, color }
    }
}

use super::BotPlayer;

impl BotPlayer for RandomPlayer {
    fn decide(&self, _state: &State, playable_actions: &[Action]) -> Action {
        let mut rng = rand::thread_rng();
        *playable_actions
            .choose(&mut rng)
            .expect("There should always be at least one playable action")
    }
}

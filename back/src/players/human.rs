use crate::player::Player;
use crate::enums::Action;
use crate::game::GameState;
use std::fmt;

/// Human player implementation
pub struct HumanPlayer {
    id: String,
    name: String,
    color: String,
}

impl HumanPlayer {
    /// Create a new human player
    pub fn new(id: String, name: String, color: String) -> Self {
        HumanPlayer { id, name, color }
    }
}

impl Player for HumanPlayer {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn color(&self) -> &str {
        &self.color
    }

    fn is_bot(&self) -> bool {
        false
    }

    fn decide_action(&self, _game_state: &GameState) -> Action {
        // Human players don't automatically decide actions,
        // they receive them from client input
        panic!("Human players should not use decide_action()");
    }
}

impl fmt::Debug for HumanPlayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HumanPlayer")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("color", &self.color)
            .finish()
    }
}

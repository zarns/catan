// Player module - Parent module for all player implementations
//
// This file defines the Player trait and re-exports player implementations

use crate::enums::Action;
use crate::game::GameState;

// Define the Player trait that all implementations must implement
pub trait Player {
    /// Returns the player's unique identifier
    fn id(&self) -> &str;

    /// Returns the player's name
    fn name(&self) -> &str;

    /// Returns the player's color
    fn color(&self) -> &str;

    /// Returns true if the player is a bot
    fn is_bot(&self) -> bool;

    /// Decides the next action for the player
    fn decide_action(&self, state: &GameState) -> Action;
}

// Re-export player implementations
pub use crate::players::HumanPlayer;
// Temporarily disabled old implementations
// pub use crate::players::RandomPlayer;
// pub use crate::players::MinimaxPlayer;
// pub use crate::players::GreedyPlayer;
// pub use crate::players::MCTSPlayer;
// pub use crate::players::WeightedRandomPlayer;

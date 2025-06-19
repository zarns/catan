// Players module - Contains all player implementations
//
// This module organizes various player implementations

// Import the Player trait
use crate::player::Player as MainPlayer;
use crate::enums::Action;
use crate::state::State;

// Define the Player trait for bot players (separate from the main Player trait)
pub trait BotPlayer {
    fn decide(&self, state: &State, playable_actions: &[Action]) -> Action;
}

// Declare the player implementation modules
pub mod human;
pub mod random;
pub mod minimax;
pub mod greedy;
pub mod weighted_random;
// pub mod mcts;  // Keep disabled for now - may need fixes

// Re-export player implementations for ease of use
pub use self::human::HumanPlayer;
pub use self::random::RandomPlayer;
pub use self::minimax::AlphaBetaPlayer;
pub use self::greedy::GreedyPlayer;
pub use self::weighted_random::WeightedRandomPlayer;

// BotPlayer trait is defined above

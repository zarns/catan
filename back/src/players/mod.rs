// Players module - Contains all player implementations
//
// This module organizes various player implementations

// Import necessary types
use crate::enums::Action;
use crate::state::State;

// Define the Player trait for bot players (separate from the main Player trait)
pub trait BotPlayer {
    fn decide(&self, state: &State, playable_actions: &[Action]) -> Action;
}

// Declare the player implementation modules
pub mod greedy;
pub mod human;
pub mod minimax;
pub mod random;
pub mod value;
pub mod weighted_random;
// pub mod mcts;  // Keep disabled for now - may need fixes

// Re-export player implementations for ease of use
pub use self::greedy::GreedyPlayer;
pub use self::human::HumanPlayer;
pub use self::minimax::AlphaBetaPlayer;
pub use self::random::RandomPlayer;
pub use self::value::ValueFunctionPlayer;
pub use self::weighted_random::WeightedRandomPlayer;

// BotPlayer trait is defined above

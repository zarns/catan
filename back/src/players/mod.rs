// Players module - Contains all player implementations
//
// This module organizes various player implementations

// Import the Player trait
use crate::player::Player;

// Declare the player implementation modules
pub mod human;
// Temporarily disable old implementations that need refactoring
// pub mod random;
// pub mod minimax;
// pub mod greedy;
// pub mod mcts;
// pub mod weighted_random;

// Re-export player implementations for ease of use
pub use self::human::HumanPlayer;
// Temporarily disable old implementations
// pub use self::random::RandomPlayer;
// pub use self::minimax::AlphaBetaPlayer as MinimaxPlayer;
// pub use self::greedy::GreedyPlayer;
// pub use self::mcts::MonteCarloPlayer as MCTSPlayer;
// pub use self::weighted_random::WeightedRandomPlayer; 
// Catan Server Library - Core Module Organization
//
// This file serves as the central organization point for the Catan game server,
// exporting all the necessary modules and types in a clean, structured manner.

// Core game data structures and enums
pub mod enums;
pub mod game;
pub mod state;
pub mod state_vector;

// Game logic implementation
pub mod deck_slices;
pub mod global_state;
pub mod map_instance;
pub mod map_template;
pub mod ordered_hashmap;
pub mod player;
pub mod players;

// Server implementation
pub mod manager;
pub mod websocket;

// Re-export common types for convenient access
pub use crate::enums::{Action, ActionPrompt, DevCard, GameConfiguration, MapType, Resource};
pub use crate::game::{Game, GameState, Player};
pub use crate::manager::{GameManager, GameStatus};
pub use crate::player::{Player as GamePlayer, RandomPlayer, HumanPlayer};
pub use crate::websocket::{WebSocketManager, WsMessage};

// Error type for the catan server
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CatanError {
    #[error("Game not found: {0}")]
    GameNotFound(String),
    
    #[error("Invalid action: {0}")]
    InvalidAction(String),
    
    #[error("Not player's turn")]
    NotPlayerTurn,
    
    #[error("Game is not in progress")]
    GameNotInProgress,
    
    #[error("WebSocket error: {0}")]
    WebSocketError(String),
    
    #[error("Internal server error: {0}")]
    InternalError(String),
}

// Result type alias for Catan operations
pub type CatanResult<T> = Result<T, CatanError>;

// Common types used throughout the application
pub type GameId = String;
pub type PlayerId = String;
pub type PlayerColor = String;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Create a new game instance with the given configuration
pub fn create_game(config: GameConfiguration) -> CatanResult<Game> {
    Ok(Game::new(uuid::Uuid::new_v4().to_string(), 
                vec!["Player".to_string(); config.num_players as usize]))
}

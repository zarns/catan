// Catan Server Library - Core Module Organization
//
// This file serves as the central organization point for the Catan game server,
// exporting all the necessary modules and types in a clean, structured manner.

// New unified architecture modules
pub mod actions;
pub mod errors;
pub mod player_system;

// Clean architecture layers
pub mod application;
pub mod websocket;

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

// Server implementation - using modern GameService in application.rs

// Re-export new unified types
pub use crate::actions::{ActionResult, GameCommand, GameEvent, GameId, PlayerAction, PlayerId};
pub use crate::errors::{
    CatanError, CatanResult, GameError, GameResult, NetworkError, PlayerError,
};

// Re-export common types for convenient access
pub use crate::enums::{ActionPrompt, DevCard, GameConfiguration, MapType, Resource};
pub use crate::game::{Game, GameState, Player};
// Removed legacy GameManager - use GameService from application.rs instead
pub use crate::player::{HumanPlayer, Player as GamePlayer};
pub use crate::player_system::{Player as NewPlayer, PlayerFactory, PlayerStrategy};
pub use crate::websocket::{WebSocketService, WsMessage};

// Common types used throughout the application
pub type PlayerColor = String;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Create a new game instance with the given configuration
pub fn create_game(config: GameConfiguration) -> CatanResult<Game> {
    Ok(Game::new(
        uuid::Uuid::new_v4().to_string(),
        vec!["Player".to_string(); config.num_players as usize],
    ))
}

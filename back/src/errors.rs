use thiserror::Error;
use serde::{Deserialize, Serialize};

use crate::actions::{PlayerId, GameId};

/// Top-level error type for the entire Catan system
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum CatanError {
    #[error("Game error: {0}")]
    Game(#[from] GameError),
    
    #[error("Player error: {0}")]
    Player(#[from] PlayerError),
    
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    
    #[error("Infrastructure error: {0}")]
    Infrastructure(#[from] InfrastructureError),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Game-specific errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum GameError {
    #[error("Game not found: {game_id}")]
    GameNotFound { game_id: GameId },
    
    #[error("Game already exists: {game_id}")]
    GameAlreadyExists { game_id: GameId },
    
    #[error("Invalid action '{action}' for player {player_id}")]
    InvalidAction { action: String, player_id: PlayerId },
    
    #[error("Not player's turn: current={current_player}, attempted={attempted_player}")]
    NotPlayerTurn { current_player: PlayerId, attempted_player: PlayerId },
    
    #[error("Game is not in progress: {game_id}")]
    GameNotInProgress { game_id: GameId },
    
    #[error("Game rule violation: {rule}")]
    RuleViolation { rule: String },
    
    #[error("Invalid game state transition: {details}")]
    InvalidStateTransition { details: String },
    
    #[error("Maximum players reached: {max_players}")]
    MaxPlayersReached { max_players: u8 },
    
    #[error("Minimum players not met: {min_players}")]
    MinPlayersNotMet { min_players: u8 },
}

/// Player-specific errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum PlayerError {
    #[error("Player not found: {player_id}")]
    PlayerNotFound { player_id: PlayerId },
    
    #[error("Player already exists: {player_id}")]
    PlayerAlreadyExists { player_id: PlayerId },
    
    #[error("Player not in game: {player_id} in {game_id}")]
    PlayerNotInGame { player_id: PlayerId, game_id: GameId },
    
    #[error("Insufficient resources for player {player_id}")]
    InsufficientResources { player_id: PlayerId },
    
    #[error("Player strategy error: {details}")]
    StrategyError { details: String },
    
    #[error("Player authentication failed: {player_id}")]
    AuthenticationFailed { player_id: PlayerId },
}

/// Network/WebSocket errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum NetworkError {
    #[error("WebSocket connection failed: {details}")]
    ConnectionFailed { details: String },
    
    #[error("Message serialization failed: {details}")]
    SerializationFailed { details: String },
    
    #[error("Message deserialization failed: {details}")]
    DeserializationFailed { details: String },
    
    #[error("Connection timeout for player {player_id}")]
    Timeout { player_id: PlayerId },
    
    #[error("Connection closed unexpectedly: {details}")]
    ConnectionClosed { details: String },
}

/// Infrastructure errors (database, persistence, etc.)
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum InfrastructureError {
    #[error("Database error: {details}")]
    Database { details: String },
    
    #[error("Persistence error: {details}")]
    Persistence { details: String },
    
    #[error("Configuration error: {details}")]
    Configuration { details: String },
    
    #[error("Resource exhausted: {resource}")]
    ResourceExhausted { resource: String },
}

/// Result type aliases for convenience
pub type CatanResult<T> = Result<T, CatanError>;
pub type GameResult<T> = Result<T, GameError>;
pub type PlayerResult<T> = Result<T, PlayerError>;
pub type NetworkResult<T> = Result<T, NetworkError>;

/// Helper methods for creating common errors
impl GameError {
    pub fn invalid_action(action: impl Into<String>, player_id: impl Into<PlayerId>) -> Self {
        Self::InvalidAction {
            action: action.into(),
            player_id: player_id.into(),
        }
    }
    
    pub fn not_player_turn(current: impl Into<PlayerId>, attempted: impl Into<PlayerId>) -> Self {
        Self::NotPlayerTurn {
            current_player: current.into(),
            attempted_player: attempted.into(),
        }
    }
    
    pub fn rule_violation(rule: impl Into<String>) -> Self {
        Self::RuleViolation {
            rule: rule.into(),
        }
    }
}

impl PlayerError {
    pub fn not_found(player_id: impl Into<PlayerId>) -> Self {
        Self::PlayerNotFound {
            player_id: player_id.into(),
        }
    }
    
    pub fn not_in_game(player_id: impl Into<PlayerId>, game_id: impl Into<GameId>) -> Self {
        Self::PlayerNotInGame {
            player_id: player_id.into(),
            game_id: game_id.into(),
        }
    }
}

impl NetworkError {
    pub fn connection_failed(details: impl Into<String>) -> Self {
        Self::ConnectionFailed {
            details: details.into(),
        }
    }
    
    pub fn serialization_failed(details: impl Into<String>) -> Self {
        Self::SerializationFailed {
            details: details.into(),
        }
    }
}

/// Convert from string to CatanError for backwards compatibility
impl From<String> for CatanError {
    fn from(msg: String) -> Self {
        CatanError::Internal(msg)
    }
}

impl From<&str> for CatanError {
    fn from(msg: &str) -> Self {
        CatanError::Internal(msg.to_string())
    }
} 
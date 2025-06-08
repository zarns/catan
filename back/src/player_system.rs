use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::actions::{PlayerAction, PlayerId};
use crate::game::GameState;
use crate::errors::{PlayerResult, PlayerError};

/// Information about a player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub id: PlayerId,
    pub name: String,
    pub color: String,
    pub is_bot: bool,
}

/// Thread-safe trait for player strategies
#[async_trait]
pub trait PlayerStrategy: Send + Sync {
    /// Get basic information about this player
    fn get_info(&self) -> PlayerInfo;
    
    /// Decide what action to take given the current game state and valid actions
    async fn decide_action(
        &self, 
        game_state: &GameState, 
        valid_actions: &[PlayerAction]
    ) -> PlayerResult<PlayerAction>;
    
    /// Called when the player joins a game
    async fn on_game_joined(&self, _game_id: &str) -> PlayerResult<()> {
        Ok(())
    }
    
    /// Called when the player leaves a game
    async fn on_game_left(&self, _game_id: &str) -> PlayerResult<()> {
        Ok(())
    }
    
    /// Called when it becomes this player's turn
    async fn on_turn_started(&self, _game_state: &GameState) -> PlayerResult<()> {
        Ok(())
    }
    
    /// Called when the player's turn ends
    async fn on_turn_ended(&self, _game_state: &GameState) -> PlayerResult<()> {
        Ok(())
    }
}

/// A player in the game system
#[derive(Clone)]
pub struct Player {
    pub info: PlayerInfo,
    pub strategy: Arc<dyn PlayerStrategy>,
}

impl std::fmt::Debug for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Player")
            .field("info", &self.info)
            .field("strategy", &"<PlayerStrategy>")
            .finish()
    }
}

impl Player {
    pub fn new(id: PlayerId, name: String, color: String, strategy: Arc<dyn PlayerStrategy>) -> Self {
        let info = PlayerInfo {
            id,
            name,
            color,
            is_bot: true, // Will be overridden by strategy if needed
        };
        
        Player { info, strategy }
    }
    
    pub fn human(id: PlayerId, name: String, color: String) -> Self {
        let strategy = Arc::new(HumanPlayerStrategy { 
            id: id.clone(), 
            name: name.clone(), 
            color: color.clone() 
        });
        
        let info = PlayerInfo {
            id,
            name,
            color,
            is_bot: false,
        };
        
        Player { info, strategy }
    }
    
    pub async fn decide_action(
        &self, 
        game_state: &GameState, 
        valid_actions: &[PlayerAction]
    ) -> PlayerResult<PlayerAction> {
        self.strategy.decide_action(game_state, valid_actions).await
    }
}

/// Human player strategy - waits for external input
pub struct HumanPlayerStrategy {
    pub id: PlayerId,
    pub name: String,
    pub color: String,
}

#[async_trait]
impl PlayerStrategy for HumanPlayerStrategy {
    fn get_info(&self) -> PlayerInfo {
        PlayerInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            color: self.color.clone(),
            is_bot: false,
        }
    }
    
    async fn decide_action(
        &self, 
        _game_state: &GameState, 
        valid_actions: &[PlayerAction]
    ) -> PlayerResult<PlayerAction> {
        // For human players, this should not be called directly
        // Instead, actions come through the WebSocket/API
        // Return a default action for now
        if let Some(action) = valid_actions.first() {
            Ok(action.clone())
        } else {
            Err(PlayerError::StrategyError { 
                details: "No valid actions available".to_string() 
            })
        }
    }
}

/// Random bot strategy
pub struct RandomPlayerStrategy {
    pub id: PlayerId,
    pub name: String,
    pub color: String,
}

#[async_trait]
impl PlayerStrategy for RandomPlayerStrategy {
    fn get_info(&self) -> PlayerInfo {
        PlayerInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            color: self.color.clone(),
            is_bot: true,
        }
    }
    
    async fn decide_action(
        &self, 
        _game_state: &GameState, 
        valid_actions: &[PlayerAction]
    ) -> PlayerResult<PlayerAction> {
        use rand::seq::SliceRandom;
        
        if valid_actions.is_empty() {
            return Err(PlayerError::StrategyError { 
                details: "No valid actions available".to_string() 
            });
        }
        
        let mut rng = rand::thread_rng();
        let action = valid_actions
            .choose(&mut rng)
            .unwrap()
            .clone();
            
        Ok(action)
    }
}

/// Factory for creating different types of players
pub struct PlayerFactory;

impl PlayerFactory {
    pub fn create_human(id: PlayerId, name: String, color: String) -> Player {
        Player::human(id, name, color)
    }
    
    pub fn create_random_bot(id: PlayerId, name: String, color: String) -> Player {
        let strategy = Arc::new(RandomPlayerStrategy { id: id.clone(), name: name.clone(), color: color.clone() });
        Player::new(id, name, color, strategy)
    }
    
    pub fn create_bot(id: PlayerId, name: String, color: String, bot_type: &str) -> PlayerResult<Player> {
        match bot_type {
            "random" => Ok(Self::create_random_bot(id, name, color)),
            _ => Err(PlayerError::StrategyError { 
                details: format!("Unknown bot type: {}", bot_type) 
            }),
        }
    }
} 
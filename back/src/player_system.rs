use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::actions::{PlayerAction, PlayerId};
use crate::errors::{PlayerError, PlayerResult};
use crate::game::GameState;

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
        valid_actions: &[PlayerAction],
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
    pub fn new(
        id: PlayerId,
        name: String,
        color: String,
        strategy: Arc<dyn PlayerStrategy>,
    ) -> Self {
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
            color: color.clone(),
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
        valid_actions: &[PlayerAction],
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
        valid_actions: &[PlayerAction],
    ) -> PlayerResult<PlayerAction> {
        // TODO: CRITICAL ISSUE - This implementation is broken for human players!
        //
        // Current problems:
        // 1. Human players should NOT automatically decide actions
        // 2. This method returns the first available action, making humans behave like bots
        // 3. There's no mechanism to inject external actions from WebSocket/HTTP
        // 4. The application layer calls this method even for human players
        //
        // Required solution:
        // 1. Human players need an action queue/channel system
        // 2. WebSocket/HTTP handlers should inject actions into this queue
        // 3. This method should wait for external action injection
        // 4. Or preferably, human players should never call decide_action()
        //    and the application layer should handle human vs bot logic differently

        // For human players, this should not be called directly
        // Instead, actions come through the WebSocket/API
        // Return a default action for now
        if let Some(action) = valid_actions.first() {
            Ok(action.clone())
        } else {
            Err(PlayerError::StrategyError {
                details: "No valid actions available".to_string(),
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
        valid_actions: &[PlayerAction],
    ) -> PlayerResult<PlayerAction> {
        use rand::seq::SliceRandom;

        if valid_actions.is_empty() {
            return Err(PlayerError::StrategyError {
                details: "No valid actions available".to_string(),
            });
        }

        let mut rng = rand::thread_rng();
        let action = valid_actions.choose(&mut rng).unwrap().clone();

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
        let strategy = Arc::new(RandomPlayerStrategy {
            id: id.clone(),
            name: name.clone(),
            color: color.clone(),
        });
        Player::new(id, name, color, strategy)
    }

    pub fn create_bot(
        id: PlayerId,
        name: String,
        color: String,
        bot_type: &str,
    ) -> PlayerResult<Player> {
        match bot_type {
            "random" => Ok(Self::create_random_bot(id, name, color)),
            _ => Err(PlayerError::StrategyError {
                details: format!("Unknown bot type: {}", bot_type),
            }),
        }
    }
}

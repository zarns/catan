use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::actions::{PlayerAction, GameCommand, GameEvent, GameId, PlayerId};
use crate::errors::{CatanResult, CatanError, GameError};
use crate::game::{Game, GameState};
use crate::player_system::{Player, PlayerFactory};

/// Core application service for game management
/// This is the main orchestration layer that coordinates between domain and infrastructure
#[derive(Clone)]
pub struct GameService {
    games: Arc<RwLock<HashMap<GameId, Arc<RwLock<Game>>>>>,
    players: Arc<RwLock<HashMap<GameId, Vec<Player>>>>,
}

impl GameService {
    pub fn new() -> Self {
        Self {
            games: Arc::new(RwLock::new(HashMap::new())),
            players: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new game with the specified configuration
    pub async fn create_game(&self, num_players: u8, bot_type: &str) -> CatanResult<GameId> {
        let game_id = Uuid::new_v4().to_string();
        
        // Create player names
        let player_names: Vec<String> = (0..num_players)
            .map(|i| format!("Player {}", i + 1))
            .collect();
        
        // Create the game instance
        let game = Game::new(game_id.clone(), player_names.clone());
        
        // Create player instances using the new player system
        let mut players = Vec::new();
        let colors = vec!["red", "blue", "white", "orange"];
        
        for (i, name) in player_names.iter().enumerate() {
            let player_id = format!("player_{}", i);
            let color = colors[i % colors.len()].to_string();
            
            let player = match bot_type {
                "random" => PlayerFactory::create_random_bot(player_id, name.clone(), color),
                "human" => PlayerFactory::create_human(player_id, name.clone(), color),
                _ => PlayerFactory::create_random_bot(player_id, name.clone(), color),
            };
            
            players.push(player);
        }
        
        // Store the game and players
        {
            let mut games = self.games.write().await;
            games.insert(game_id.clone(), Arc::new(RwLock::new(game)));
        }
        
        {
            let mut game_players = self.players.write().await;
            game_players.insert(game_id.clone(), players);
        }
        
        Ok(game_id)
    }

    /// Get a game by ID
    pub async fn get_game(&self, game_id: &str) -> CatanResult<Game> {
        let games = self.games.read().await;
        
        if let Some(game_arc) = games.get(game_id) {
            let game = game_arc.read().await;
            Ok(game.clone())
        } else {
            Err(CatanError::Game(GameError::GameNotFound { 
                game_id: game_id.to_string() 
            }))
        }
    }

    /// Check if a game exists
    pub async fn game_exists(&self, game_id: &str) -> bool {
        let games = self.games.read().await;
        games.contains_key(game_id)
    }

    /// Process a player action
    pub async fn process_action(
        &self, 
        game_id: &str, 
        player_id: &str, 
        action: PlayerAction
    ) -> CatanResult<Vec<GameEvent>> {
        let games = self.games.read().await;
        
        let game_arc = games.get(game_id)
            .ok_or_else(|| CatanError::Game(GameError::GameNotFound { 
                game_id: game_id.to_string() 
            }))?;
            
        let mut game = game_arc.write().await;
        
        // Convert PlayerAction to the internal Action type
        let internal_action = action.clone().into();
        
        // Process the action
        match game.process_action(player_id, internal_action) {
            Ok(()) => {
                // Generate events based on the action
                let events = vec![
                    GameEvent::ActionExecuted {
                        game_id: game_id.to_string(),
                        player_id: player_id.to_string(),
                        action,
                        success: true,
                        message: "Action processed successfully".to_string(),
                    }
                ];
                
                Ok(events)
            },
            Err(error) => {
                let events = vec![
                    GameEvent::ActionExecuted {
                        game_id: game_id.to_string(),
                        player_id: player_id.to_string(),
                        action,
                        success: false,
                        message: error,
                    }
                ];
                
                Ok(events)
            }
        }
    }

    /// Get the current game state
    pub async fn get_game_state(&self, game_id: &str) -> CatanResult<GameState> {
        let game = self.get_game(game_id).await?;
        Ok(game.game_state)
    }

    /// Get players for a game
    pub async fn get_players(&self, game_id: &str) -> CatanResult<Vec<Player>> {
        let players = self.players.read().await;
        
        if let Some(game_players) = players.get(game_id) {
            Ok(game_players.clone())
        } else {
            Err(CatanError::Game(GameError::GameNotFound { 
                game_id: game_id.to_string() 
            }))
        }
    }

    /// Process bot turn if it's a bot's turn
    pub async fn process_bot_turn(&self, game_id: &str) -> CatanResult<Option<Vec<GameEvent>>> {
        // Get the game to check current player
        let game = self.get_game(game_id).await?;
        let players = self.get_players(game_id).await?;
        
        if game.current_player_index >= players.len() {
            return Ok(None);
        }
        
        let current_player = &players[game.current_player_index];
        
        // Check if current player is a bot by examining their strategy type
        if current_player.info.is_bot {
            // Let the bot decide what action to take
            let available_actions = vec![PlayerAction::EndTurn]; // Simplified for now
            
            match current_player.decide_action(&game.game_state, &available_actions).await {
                Ok(action) => {
                    // Process the bot's action
                    self.process_action(game_id, &current_player.info.id, action).await
                        .map(Some)
                },
                Err(_) => {
                    // If bot fails to decide, just end turn
                    self.process_action(game_id, &current_player.info.id, PlayerAction::EndTurn).await
                        .map(Some)
                }
            }
        } else {
            Ok(None) // Not a bot's turn
        }
    }

    /// Remove a game (cleanup)
    pub async fn remove_game(&self, game_id: &str) -> CatanResult<()> {
        {
            let mut games = self.games.write().await;
            games.remove(game_id);
        }
        
        {
            let mut players = self.players.write().await;
            players.remove(game_id);
        }
        
        Ok(())
    }

    /// Get all active game IDs
    pub async fn list_games(&self) -> Vec<GameId> {
        let games = self.games.read().await;
        games.keys().cloned().collect()
    }
}

impl Default for GameService {
    fn default() -> Self {
        Self::new()
    }
} 
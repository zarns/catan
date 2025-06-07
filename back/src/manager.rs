use std::collections::HashSet;
use std::fmt;
use uuid::Uuid;
use thiserror::Error;
use serde::{Deserialize, Serialize};

use crate::game::Game;
use crate::player::{Player, RandomPlayer};
use crate::enums::Action as GameAction;

// Game status enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameStatus {
    Waiting,
    InProgress,
    Finished { winner: Option<String> },
}

// Game manager errors
#[derive(Error, Debug)]
pub enum GameError {
    #[error("Invalid action: {0}")]
    InvalidAction(String),
    
    #[error("Game is not in progress")]
    GameNotInProgress,
    
    #[error("Not player's turn")]
    NotPlayerTurn,
    
    #[error("Invalid player index")]
    InvalidPlayerIndex,
    
    #[error("Action failed: {0}")]
    ActionFailed(String),
    
    #[error("Game not found: {0}")]
    GameNotFound(String),
}

impl fmt::Display for GameStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameStatus::Waiting => write!(f, "waiting"),
            GameStatus::InProgress => write!(f, "in_progress"),
            GameStatus::Finished { winner } => {
                if let Some(w) = winner {
                    write!(f, "finished (winner: {})", w)
                } else {
                    write!(f, "finished (no winner)")
                }
            }
        }
    }
}

// Action type that represents any game action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub action_type: String,
    pub edge_id: Option<String>,
    pub node_id: Option<String>,
    pub coordinate: Option<(i8, i8, i8)>,
    pub resource: Option<String>,
    pub resources: Option<Vec<String>>,
    pub target_color: Option<String>,
}

// Action result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    pub message: String,
}

// The GameManager struct
pub struct GameManager {
    pub id: String,
    pub game: Option<Game>,
    pub players: Vec<Box<dyn Player>>,
    pub bot_colors: HashSet<String>,
    pub current_turn: usize,
    pub status: GameStatus,
}

impl GameManager {
    /// Create a new empty game manager
    pub fn new() -> Self {
        // Create with default/empty values - actual game will be created via create_game
        GameManager {
            id: String::new(),
            game: None,
            players: Vec::new(),
            bot_colors: HashSet::new(),
            current_turn: 0,
            status: GameStatus::Waiting,
        }
    }
    
    /// Create a new game and return its ID
    pub fn create_game(&mut self, num_players: u8, bot_type: &str) -> String {
        let id = Uuid::new_v4().to_string();
        
        // Set up players (all bots for now)
        let mut players: Vec<Box<dyn Player>> = Vec::new();
        let mut bot_colors = HashSet::new();
        let colors = vec!["red", "blue", "white", "orange"];
        
        // Create bot players
        let player_names: Vec<String> = (0..num_players).map(|i| format!("Bot {}", i + 1)).collect();
        
        for i in 0..num_players as usize {
            let color = colors[i % colors.len()].to_string();
            let name = format!("Bot {}", i + 1);
            bot_colors.insert(color.clone());
            
            // Create the appropriate bot type
            match bot_type {
                "random" => players.push(Box::new(RandomPlayer {})),
                // Add other bot types here
                _ => players.push(Box::new(RandomPlayer {})),
            }
        }
        
        // Create the actual Game instance
        let game = Game::new(id.clone(), player_names);
        
        
        // Update manager properties
        self.id = id.clone();
        self.game = Some(game);
        self.players = players;
        self.bot_colors = bot_colors;
        self.current_turn = 0;
        self.status = GameStatus::Waiting;
        
        id
    }
    
    /// Update the game state after an action has been applied
    pub fn update_state(&mut self) {
        if let Some(game) = &self.game {
            
            // Check for game ending conditions
            if let Some(player) = game.players.iter().find(|p| p.victory_points >= 10) {
                self.status = GameStatus::Finished { winner: Some(player.name.clone()) };
            }
        }
    }
    
    /// Process a player action and update the game state
    pub fn process_action(&mut self, player_idx: usize, action: Action) -> Result<ActionResult, GameError> {
        // Ensure game is in progress
        if self.status != GameStatus::InProgress {
            return Err(GameError::GameNotInProgress);
        }
        
        // Ensure it's the player's turn
        if player_idx != self.current_turn {
            return Err(GameError::NotPlayerTurn);
        }
        
        // Get the game instance
        let game = match &mut self.game {
            Some(game) => game,
            None => return Err(GameError::ActionFailed("Game state not found".to_string())),
        };
        
        // Get player ID
        let player_id = match game.players.get(player_idx) {
            Some(player) => player.id.clone(),
            None => return Err(GameError::InvalidPlayerIndex),
        };
        
        // Convert our Action to GameAction
        let game_action = self.convert_to_game_action(&action);
        
        // Delegate to the Game's process_action
        match game.process_action(&player_id, game_action) {
            Ok(()) => {
                // Update our state from the game's state
                self.update_state();
                
                Ok(ActionResult {
                    success: true,
                    message: "Action processed successfully".to_string(),
                })
            },
            Err(err_msg) => {
                Err(GameError::ActionFailed(err_msg))
            }
        }
    }
    
    // Helper function to convert between action types
    fn convert_to_game_action(&self, action: &Action) -> GameAction {
        // This is a simplified conversion - you would need to implement a proper mapping
        // between your manager's Action type and the crate::enums::Action type
        match action.action_type.as_str() {
            "roll_dice" => GameAction::Roll { 
                color: self.current_turn as u8, 
                dice_opt: None 
            },
            "end_turn" => GameAction::EndTurn { 
                color: self.current_turn as u8 
            },
            // Add more conversions as needed
            _ => GameAction::EndTurn { 
                color: self.current_turn as u8 
            }, // Default fallback
        }
    }
    
    /// Start the bot's turn
    pub fn process_bot_turn(&mut self, _game_id: &str) -> Result<(Action, ActionResult), GameError> {
        // Check if it's a bot's turn
        if let Some(game) = &self.game {
            let current_player_idx = game.current_player_index;
            let player_color = &game.players[current_player_idx].color;
            
            if !self.bot_colors.contains(player_color) {
                return Err(GameError::NotPlayerTurn); // Not a bot's turn
            }
            
            // Get the bot player
            if let Some(bot) = self.players.get(current_player_idx) {
                // Let the bot decide what action to take
                let action = bot.decide_action(&self.game_state);
                
                // Process the action using our delegated process_action method
                match self.process_action(current_player_idx, action.clone()) {
                    Ok(result) => Ok((action, result)),
                    Err(e) => {
                        // Bot action failed - log error and try to end turn
                        eprintln!("Bot action failed: {}", e);
                        
                        // Try to end the turn as a fallback
                        let end_turn_action = Action {
                            action_type: "end_turn".to_string(),
                            edge_id: None,
                            node_id: None,
                            coordinate: None,
                            resource: None,
                            resources: None,
                            target_color: None,
                        };
                        
                        match self.process_action(current_player_idx, end_turn_action.clone()) {
                            Ok(result) => Ok((end_turn_action, result)),
                            Err(e) => Err(e),
                        }
                    }
                }
            } else {
                Err(GameError::InvalidPlayerIndex)
            }
        } else {
            Err(GameError::GameNotInProgress)
        }
    }
    
    /// End the game, optionally specifying a winner
    pub fn end_game(&mut self, winner_color: Option<String>) {
        self.status = GameStatus::Finished { winner: winner_color.clone().map(|_| "Unknown".to_string()) };
        self.game_state.status = self.status.clone();
        self.game_state.winning_color = winner_color;
    }
    
    /// Get the current game state
    pub fn get_state(&self) -> GameState {
        self.game_state.clone()
    }
    
    /// Start the game
    pub fn start_game(&mut self) {
        self.status = GameStatus::InProgress;
        self.game_state.status = self.status.clone();
        
        // Initialize game elements like the board, initial placements, etc.
        // This would be more complex in a real implementation
    }
    
    /// Check if the next player is a bot
    pub fn next_player_is_bot(&self) -> bool {
        if let Some(game) = &self.game {
            let next_player_idx = (game.current_player_index + 1) % game.players.len();
            let next_player_color = &game.players[next_player_idx].color;
            return self.bot_colors.contains(next_player_color);
        }
        false
    }
}

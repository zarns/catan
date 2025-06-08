use serde::{Deserialize, Serialize};
use axum::extract::ws::{Message, WebSocket};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tokio::time::Duration;
use log;
use std::collections::HashMap;
use thiserror::Error;

use crate::game::GameState;
use crate::manager::{GameManager, Action, ActionResult};
use crate::{CatanResult, CatanError};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    #[serde(rename = "game_state")]
    GameState(GameState),
    
    #[serde(rename = "error")]
    Error(String),
    
    #[serde(rename = "greeting")]
    Greeting(String),
    
    #[serde(rename = "player_action")]
    PlayerAction(Action),
    
    #[serde(rename = "action_result")]
    ActionResult(ActionResult),
    
    #[serde(rename = "bot_action")]
    BotAction(Action),
    
    #[serde(rename = "thinking")]
    Thinking { player_id: String },
}

// Main WebSocket connection handler
pub struct WebSocketManager {
    tx: broadcast::Sender<(String, WsMessage)>,
    game_managers: Arc<Mutex<HashMap<String, Arc<Mutex<GameManager>>>>>,
}

impl WebSocketManager {
    // Create a new WebSocket manager
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        WebSocketManager {
            tx,
            game_managers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    // Get the broadcast sender
    pub fn sender(&self) -> broadcast::Sender<(String, WsMessage)> {
        self.tx.clone()
    }
    
    // Create a new game
    pub async fn create_game(
        &self, 
        num_players: u8,
        bot_type: &str
    ) -> CatanResult<String> {
        // Create a new GameManager
        let mut game_manager = GameManager::new();
        
        // Use GameManager to create a game and get its ID
        let game_id = game_manager.create_game(num_players, bot_type);
        
        // Start the game
        game_manager.start_game();
        
        // Store the GameManager in our collection
        let mut game_managers = self.game_managers.lock().await;
        game_managers.insert(game_id.clone(), Arc::new(Mutex::new(game_manager)));
        
        Ok(game_id)
    }
    
    // Get a game state
    pub async fn get_game_state(&self, game_id: &str) -> CatanResult<GameState> {
        let game_managers = self.game_managers.lock().await;
        
        if let Some(game_manager) = game_managers.get(game_id) {
            let game_manager = game_manager.lock().await;
            Ok(game_manager.get_state())
        } else {
            Err(CatanError::Game(crate::errors::GameError::GameNotFound { 
                game_id: game_id.to_string() 
            }))
        }
    }
    
    // Get the full game object
    pub async fn get_game(&self, game_id: &str) -> CatanResult<crate::game::Game> {
        let game_managers = self.game_managers.lock().await;
        
        if let Some(game_manager) = game_managers.get(game_id) {
            let game_manager = game_manager.lock().await;
            if let Some(game) = &game_manager.game {
                Ok(game.clone())
            } else {
                Err(CatanError::Game(crate::errors::GameError::GameNotFound { 
                    game_id: game_id.to_string() 
                }))
            }
        } else {
            Err(CatanError::Game(crate::errors::GameError::GameNotFound { 
                game_id: game_id.to_string() 
            }))
        }
    }

    // Check if game exists
    pub async fn game_exists(&self, game_id: &str) -> bool {
        let game_managers = self.game_managers.lock().await;
        game_managers.contains_key(game_id)
    }
    
    // Handle a new WebSocket connection
    pub async fn handle_connection(&self, socket: WebSocket, game_id: String) {
        log::info!("New WebSocket connection for game {}", game_id);
        
        // Split socket into sender and receiver
        let (mut sender, mut receiver) = socket.split();
        
        // Send a greeting message
        let greeting = WsMessage::Greeting("Welcome to Catan WebSocket server!".to_string());
        if let Ok(greeting_json) = serde_json::to_string(&greeting) {
            if let Err(e) = sender.send(Message::Text(greeting_json.into())).await {
                log::error!("Error sending greeting: {}", e);
                return;
            }
        }
        
        // Check if game exists
        let game_exists = self.game_exists(&game_id).await;
        
        if !game_exists {
            // Send error message if game doesn't exist
            let error_msg = WsMessage::Error(format!("Game with ID {} not found", game_id));
            if let Ok(error_json) = serde_json::to_string(&error_msg) {
                let _ = sender.send(Message::Text(error_json.into())).await;
            }
            return;
        }
        
        // Send initial game state
        match self.get_game_state(&game_id).await {
            Ok(game_state) => {
                let state_msg = WsMessage::GameState(game_state);
                if let Ok(state_json) = serde_json::to_string(&state_msg) {
                    if let Err(e) = sender.send(Message::Text(state_json.into())).await {
                        log::error!("Error sending initial state: {}", e);
                        return;
                    }
                }
            },
            Err(e) => {
                log::error!("Error getting initial game state: {}", e);
                return;
            }
        }
        
        // Subscribe to broadcast channel
        let mut rx = self.tx.subscribe();
        
        // Message handling task
        let tx_clone = self.tx.clone();
        let game_id_clone = game_id.clone();
        let game_managers = self.game_managers.clone();
        
        // Task that forwards broadcasts to client
        let game_id_for_forward = game_id.clone();
        let mut forward_task = tokio::spawn(async move {
            while let Ok((msg_game_id, msg)) = rx.recv().await {
                // Only forward messages for this game
                if msg_game_id == game_id_for_forward {
                    match serde_json::to_string(&msg) {
                        Ok(json) => {
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                        }
                        Err(e) => log::error!("Serialization error: {}", e),
                    }
                }
            }
        });
        
        // Task that receives messages from client  
        let game_id_for_receive = game_id.clone();
        let websocket_manager = self.clone();
        let mut receive_task = tokio::spawn(async move {
            while let Some(Ok(message)) = receiver.next().await {
                match message {
                    Message::Text(text) => {
                        // Parse the message
                        if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                            match ws_msg {
                                WsMessage::PlayerAction(action) => {
                                    // Get player ID from connection (simplified for now)
                                    let player_id = "player_0".to_string(); // In reality would come from auth
                                    
                                    // Process player action
                                    let result = {
                                        let game_managers_guard = game_managers.lock().await;
                                        if let Some(game_manager) = game_managers_guard.get(&game_id_for_receive) {
                                            let mut game_manager_lock = game_manager.lock().await;
                                            let player_idx = 0; // For now, hardcoded as the first player
                                            game_manager_lock.process_action(player_idx, action.clone())
                                                                            } else {
                                        Err(crate::manager::GameError::GameNotFound(game_id_for_receive.clone()))
                                    }
                                    };
                                    
                                    // Send action result
                                    match result {
                                        Ok(action_result) => {
                                            // Send success result
                                            let _ = tx_clone.send((
                                                game_id_for_receive.clone(),
                                                WsMessage::ActionResult(action_result.clone()),
                                            ));
                                            
                                            // Send updated game state
                                            if action_result.success {
                                                if let Ok(updated_state) = websocket_manager.get_game_state(&game_id_for_receive).await {
                                                    let _ = tx_clone.send((
                                                        game_id_for_receive.clone(),
                                                        WsMessage::GameState(updated_state),
                                                    ));
                                                    
                                                    // Process bot turns if it's a bot's turn next
                                                    websocket_manager.process_bot_turns(&game_id_for_receive, tx_clone.clone()).await;
                                                }
                                            }
                                        },
                                        Err(e) => {
                                            // Send error message
                                            let _ = tx_clone.send((
                                                game_id_for_receive.clone(),
                                                WsMessage::Error(format!("Action failed: {}", e)),
                                            ));
                                        }
                                    }
                                },
                                _ => {
                                    // Handle other message types if needed
                                    log::info!("Received message: {:?}", ws_msg);
                                }
                            }
                        } else {
                            log::warn!("Failed to parse message: {}", text);
                            let error_msg = WsMessage::Error("Invalid message format".to_string());
                            let _ = tx_clone.send((game_id_for_receive.clone(), error_msg));
                        }
                    },
                    Message::Close(_) => break,
                    _ => { /* Ignore other message types */ }
                }
            }
        });
        
        // Wait for any task to finish
        tokio::select! {
            _ = &mut forward_task => {
                receive_task.abort();
            },
            _ = &mut receive_task => {
                forward_task.abort();
            }
        }
        
        log::info!("WebSocket connection closed for game {}", game_id);
    }
    
    // Process bot turns until a human player's turn
    async fn process_bot_turns(&self, game_id: &str, tx: broadcast::Sender<(String, WsMessage)>) {
        let mut bot_turn = true;
        
        while bot_turn {
            // Check if it's a bot's turn
            let is_bot_turn = {
                let game_managers = self.game_managers.lock().await;
                if let Some(game_manager) = game_managers.get(game_id) {
                    let game_manager = game_manager.lock().await;
                    game_manager.next_player_is_bot()
                } else {
                    false
                }
            };
            
            if !is_bot_turn {
                break;
            }
            
            // Signal that the bot is thinking
            let bot_thinking = WsMessage::Thinking { 
                player_id: "bot".to_string() 
            };
            let _ = tx.send((game_id.to_string(), bot_thinking));
            
            // Wait a bit to simulate thinking
            tokio::time::sleep(Duration::from_millis(1000)).await;
            
            // Process the bot turn
            let result = {
                let game_managers = self.game_managers.lock().await;
                if let Some(game_manager) = game_managers.get(game_id) {
                    let mut game_manager = game_manager.lock().await;
                    game_manager.process_bot_turn(game_id)
                } else {
                    Err(crate::manager::GameError::GameNotFound(game_id.to_string()))
                }
            };
            
            match result {
                Ok((action, action_result)) => {
                    // Send the bot action
                    let _ = tx.send((
                        game_id.to_string(), 
                        WsMessage::BotAction(action)
                    ));
                    
                    // Send the action result
                    let _ = tx.send((
                        game_id.to_string(),
                        WsMessage::ActionResult(action_result),
                    ));
                    
                    // Send updated game state
                    if let Ok(updated_state) = self.get_game_state(game_id).await {
                        let _ = tx.send((
                            game_id.to_string(),
                            WsMessage::GameState(updated_state),
                        ));
                    }
                },
                Err(e) => {
                    log::error!("Failed to process bot turn for game {}: {}", game_id, e);
                    bot_turn = false;
                }
            }
            
            // Small delay between bot turns
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
    
    // Create and start a bot game simulation
    pub async fn simulate_bot_game(&self, num_players: u8) -> CatanResult<String> {
        // Create game with bots
        let game_id = self.create_game(num_players, "random").await?;
        
        // Start the first bot turn in a separate task
        let tx = self.tx.clone();
        let game_id_clone = game_id.clone();
        let self_clone = self.clone();
        
        tokio::spawn(async move {
            self_clone.process_bot_turns(&game_id_clone, tx).await;
        });
        
        Ok(game_id)
    }
}

impl Clone for WebSocketManager {
    fn clone(&self) -> Self {
        WebSocketManager {
            tx: self.tx.clone(),
            game_managers: self.game_managers.clone(),
        }
    }
}

    // Removed duplicate GameError - using the unified error system from errors.rs

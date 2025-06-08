use axum::extract::ws::{Message, WebSocket};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use log;

use crate::actions::{PlayerAction, GameEvent, GameId};
use crate::application::GameService;
use crate::errors::CatanResult;
use crate::game::{Game, GameState};

/// WebSocket message types for client-server communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "game_state")]
    GameState { game: Game },
    
    #[serde(rename = "game_updated")]
    GameUpdated { game: Game },
    
    #[serde(rename = "player_action")]
    PlayerAction { action: PlayerAction },
    
    #[serde(rename = "action_result")]
    ActionResult { 
        success: bool, 
        message: String,
        events: Vec<GameEvent>,
    },
    
    #[serde(rename = "error")]
    Error { message: String },
    
    #[serde(rename = "greeting")]
    Greeting { message: String },
    
    #[serde(rename = "bot_thinking")]
    BotThinking { player_id: String },
}

/// WebSocket service that handles real-time communication
/// This is purely an infrastructure concern - no business logic here
#[derive(Clone)]
pub struct WebSocketService {
    game_service: Arc<GameService>,
    broadcaster: broadcast::Sender<(GameId, WsMessage)>,
}

impl WebSocketService {
    pub fn new(game_service: Arc<GameService>) -> Self {
        let (broadcaster, _) = broadcast::channel(1000);
        
        Self {
            game_service,
            broadcaster,
        }
    }

    /// Handle a new WebSocket connection
    pub async fn handle_connection(&self, socket: WebSocket, game_id: String) {
        log::info!("New WebSocket connection for game {}", game_id);

        // Split socket for concurrent read/write
        let (mut sender, mut receiver) = socket.split();

        // Send greeting
        let greeting = WsMessage::Greeting {
            message: "Connected to Catan game".to_string(),
        };
        
        if let Err(e) = self.send_message(&mut sender, &greeting).await {
            log::error!("Failed to send greeting: {}", e);
            return;
        }

        // Check if game exists and send initial state
        if !self.game_service.game_exists(&game_id).await {
            let error = WsMessage::Error {
                message: format!("Game {} not found", game_id),
            };
            let _ = self.send_message(&mut sender, &error).await;
            return;
        }

        // Send initial game state
        match self.game_service.get_game(&game_id).await {
            Ok(game) => {
                let state_msg = WsMessage::GameState { game };
                if let Err(e) = self.send_message(&mut sender, &state_msg).await {
                    log::error!("Failed to send initial game state: {}", e);
                    return;
                }
            },
            Err(e) => {
                log::error!("Failed to get initial game state: {}", e);
                return;
            }
        }

        // Subscribe to game updates
        let mut game_updates = self.broadcaster.subscribe();

        // Task to forward game updates to this client
        let game_id_for_updates = game_id.clone();
        let mut update_task = tokio::spawn(async move {
            while let Ok((update_game_id, message)) = game_updates.recv().await {
                if update_game_id == game_id_for_updates {
                    if let Err(_) = Self::send_message_static(&mut sender, &message).await {
                        break; // Client disconnected
                    }
                }
            }
        });

        // Task to handle incoming messages
        let game_service = self.game_service.clone();
        let broadcaster = self.broadcaster.clone();
        let game_id_for_messages = game_id.clone();
        
        let mut message_task = tokio::spawn(async move {
            while let Some(Ok(message)) = receiver.next().await {
                match message {
                                         Message::Text(text) => {
                         if let Err(e) = Self::handle_text_message(
                             &game_service, 
                             &broadcaster, 
                             &game_id_for_messages, 
                             text.to_string()
                         ).await {
                             log::error!("Error handling message: {}", e);
                         }
                     },
                    Message::Close(_) => {
                        log::info!("WebSocket connection closed for game {}", game_id_for_messages);
                        break;
                    },
                    _ => {
                        // Ignore other message types
                    }
                }
            }
        });

        // Wait for either task to complete (client disconnect or error)
        tokio::select! {
            _ = &mut update_task => {
                message_task.abort();
            }
            _ = &mut message_task => {
                update_task.abort();
            }
        }

        log::info!("WebSocket connection terminated for game {}", game_id);
    }

    /// Handle incoming text messages
    async fn handle_text_message(
        game_service: &GameService,
        broadcaster: &broadcast::Sender<(GameId, WsMessage)>,
        game_id: &str,
        text: String,
    ) -> CatanResult<()> {
        // Parse the incoming message
        let ws_message: WsMessage = serde_json::from_str(&text)
            .map_err(|e| crate::errors::CatanError::Network(
                crate::errors::NetworkError::DeserializationFailed { 
                    details: e.to_string() 
                }
            ))?;

        match ws_message {
            WsMessage::PlayerAction { action } => {
                // For now, assume first player (in real app, get from authentication)
                let player_id = "player_0";
                
                // Process the action through the game service
                match game_service.process_action(game_id, player_id, action).await {
                    Ok(events) => {
                        // Send action result
                        let result_msg = WsMessage::ActionResult {
                            success: true,
                            message: "Action processed".to_string(),
                            events: events.clone(),
                        };
                        let _ = broadcaster.send((game_id.to_string(), result_msg));

                        // Send updated game state
                        if let Ok(updated_game) = game_service.get_game(game_id).await {
                            let update_msg = WsMessage::GameUpdated { game: updated_game };
                            let _ = broadcaster.send((game_id.to_string(), update_msg));
                        }

                        // Process bot turns if needed
                        Self::process_bot_turns(game_service, broadcaster, game_id).await;
                    },
                    Err(e) => {
                        let error_msg = WsMessage::Error {
                            message: format!("Action failed: {}", e),
                        };
                        let _ = broadcaster.send((game_id.to_string(), error_msg));
                    }
                }
            },
            _ => {
                log::debug!("Received unhandled message type: {:?}", ws_message);
            }
        }

        Ok(())
    }

    /// Process bot turns
    async fn process_bot_turns(
        game_service: &GameService,
        broadcaster: &broadcast::Sender<(GameId, WsMessage)>,
        game_id: &str,
    ) {
        // Keep processing bot turns until it's a human's turn or game ends
        while let Ok(Some(_events)) = game_service.process_bot_turn(game_id).await {
            // Send bot thinking indicator
            let thinking_msg = WsMessage::BotThinking {
                player_id: "current_bot".to_string(), // Simplified
            };
            let _ = broadcaster.send((game_id.to_string(), thinking_msg));

            // Small delay to make bot moves visible
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            // Send updated game state after bot move
            if let Ok(updated_game) = game_service.get_game(game_id).await {
                let update_msg = WsMessage::GameUpdated { game: updated_game };
                let _ = broadcaster.send((game_id.to_string(), update_msg));
            }
        }
    }

    /// Send a message to a WebSocket sender
    async fn send_message(
        &self,
        sender: &mut futures::stream::SplitSink<WebSocket, Message>,
        message: &WsMessage,
    ) -> Result<(), axum::Error> {
        Self::send_message_static(sender, message).await
    }

    /// Static version of send_message for use in spawned tasks
    async fn send_message_static(
        sender: &mut futures::stream::SplitSink<WebSocket, Message>,
        message: &WsMessage,
    ) -> Result<(), axum::Error> {
        let json = serde_json::to_string(message)
            .map_err(|e| axum::Error::new(e))?;
        
        sender.send(Message::Text(json.into())).await
            .map_err(|e| axum::Error::new(e))
    }

    /// Get the broadcaster for sending messages to all clients
    pub fn broadcaster(&self) -> broadcast::Sender<(GameId, WsMessage)> {
        self.broadcaster.clone()
    }
} 
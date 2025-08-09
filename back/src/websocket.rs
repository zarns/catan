use axum::extract::ws::{Message, WebSocket};
use futures::{sink::SinkExt, stream::StreamExt};
use log;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::actions::{GameEvent, GameId, PlayerAction};
use crate::application::GameService;
use crate::errors::CatanResult;
use crate::game::Game;

/// WebSocket message types for client-server communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "game_state")]
    GameState { game: Game },

    #[serde(rename = "game_updated")]
    GameUpdated { game: Game },

    #[serde(rename = "player_action")]
    PlayerAction {
        action: PlayerAction, // Accept enum format directly: {Roll: {}}
    },

    #[serde(rename = "get_game_state")]
    GetGameState,

    // ✅ REMOVED: BotAction - Bot actions are now automatic, not triggered by frontend
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

    #[serde(rename = "create_game")]
    CreateGame {
        mode: String, // 'HUMAN_VS_CATANATRON' | 'RANDOM_BOTS' | 'CATANATRON_BOTS'
        num_players: u8,
    },

    #[serde(rename = "game_created")]
    GameCreated { game_id: String, game: Game },
}

// Convert array action format to PlayerAction enum
// Expected format: [player_color, action_type, action_data]
// Removed array_to_player_action function - now accepting enum format directly

/// WebSocket service that handles real-time communication
/// This is purely an infrastructure concern - no business logic here
#[derive(Clone)]
pub struct WebSocketService {
    game_service: Arc<GameService>,
    broadcaster: broadcast::Sender<(GameId, WsMessage)>,
    // Track active connections per game
    active_connections: Arc<RwLock<HashMap<GameId, HashSet<String>>>>,
    // Track bot simulation tasks that can be cancelled
    bot_tasks: Arc<RwLock<HashMap<GameId, tokio::sync::broadcast::Sender<()>>>>,
}

impl WebSocketService {
    pub fn new(game_service: Arc<GameService>) -> Self {
        let (broadcaster, _) = broadcast::channel(1000);

        Self {
            game_service,
            broadcaster,
            active_connections: Arc::new(RwLock::new(HashMap::new())),
            bot_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Handle a new WebSocket connection
    pub async fn handle_connection(&self, socket: WebSocket, game_id: String) {
        // Generate a unique connection ID
        let connection_id = format!("conn_{}", uuid::Uuid::new_v4());
        log::info!(
            "🔌 WebSocket connected: {} (game {})",
            connection_id,
            game_id
        );

        // Add this connection to our tracking
        self.add_connection(&game_id, &connection_id).await;

        // Split socket for concurrent read/write
        let (mut sender, mut receiver) = socket.split();

        // Send greeting
        let greeting = WsMessage::Greeting {
            message: "Connected to Catan game".to_string(),
        };

        if let Err(e) = self.send_message(&mut sender, &greeting).await {
            log::error!("❌ Failed to send greeting: {}", e);
            self.remove_connection(&game_id, &connection_id).await;
            return;
        }

        // Check if game exists and send initial state
        if !self.game_service.game_exists(&game_id).await {
            let error = WsMessage::Error {
                message: format!("Game {game_id} not found"),
            };
            let _ = self.send_message(&mut sender, &error).await;
            self.remove_connection(&game_id, &connection_id).await;
            return;
        }

        // Send initial game state
        match self.game_service.get_game(&game_id).await {
            Ok(game) => {
                let state_msg = WsMessage::GameState { game };
                if let Err(e) = self.send_message(&mut sender, &state_msg).await {
                    log::error!("❌ Failed to send initial game state: {}", e);
                    self.remove_connection(&game_id, &connection_id).await;
                    return;
                }
            }
            Err(e) => {
                log::error!("❌ Failed to get initial game state: {}", e);
                self.remove_connection(&game_id, &connection_id).await;
                return;
            }
        }

        // Subscribe to game updates FIRST
        let mut game_updates = self.broadcaster.subscribe();

        // Start bot gameplay only if this is the first connection for this game
        let should_start_bots = {
            let connections = self.active_connections.read().await;
            connections
                .get(&game_id)
                .is_some_and(|conns| conns.len() == 1)
        };

        if should_start_bots {
            log::info!("🤖 Starting bots for game {}", game_id);
            self.start_bot_simulation(&game_id).await;
        }

        // Task to forward game updates to this client
        let game_id_for_updates = game_id.clone();
        let connection_id_for_updates = connection_id.clone();
        let mut update_task = tokio::spawn(async move {
            while let Ok((update_game_id, message)) = game_updates.recv().await {
                if update_game_id == game_id_for_updates {
                    if let Err(e) = Self::send_message_static(&mut sender, &message).await {
                        log::error!(
                            "Failed to send message to connection {}: {:?}",
                            connection_id_for_updates,
                            e
                        );
                        break; // Client disconnected
                    }
                }
            }
        });

        // Task to handle incoming messages
        let game_service = self.game_service.clone();
        let broadcaster = self.broadcaster.clone();
        let game_id_for_messages = game_id.clone();
        let connection_id_for_messages = connection_id.clone();
        let service_for_messages = self.clone();

        let mut message_task = tokio::spawn(async move {
            while let Some(Ok(message)) = receiver.next().await {
                match message {
                    Message::Text(text) => {
                        if let Err(e) = Self::handle_text_message(
                            &game_service,
                            &broadcaster,
                            &game_id_for_messages,
                            text.to_string(),
                            &service_for_messages,
                        )
                        .await
                        {
                            log::error!(
                                "Error handling message from connection {}: {}",
                                connection_id_for_messages,
                                e
                            );
                        }
                    }
                    Message::Close(_) => {
                        log::info!(
                            "WebSocket connection {} closed for game {}",
                            connection_id_for_messages,
                            game_id_for_messages
                        );
                        break;
                    }
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

        // Connection cleanup
        self.remove_connection(&game_id, &connection_id).await;
        log::info!(
            "WebSocket connection {} terminated for game {}",
            connection_id,
            game_id
        );
    }

    /// Add a connection to tracking
    async fn add_connection(&self, game_id: &str, connection_id: &str) {
        let mut connections = self.active_connections.write().await;
        connections
            .entry(game_id.to_string())
            .or_insert_with(HashSet::new)
            .insert(connection_id.to_string());
    }

    /// Remove a connection from tracking and stop bots if no connections remain
    async fn remove_connection(&self, game_id: &str, connection_id: &str) {
        let should_stop_bots = {
            let mut connections = self.active_connections.write().await;
            if let Some(game_connections) = connections.get_mut(game_id) {
                game_connections.remove(connection_id);
                let remaining = game_connections.len();

                if remaining == 0 {
                    connections.remove(game_id);
                    log::info!(
                        "➖ Last client disconnected from game {}. Stopping bots.",
                        game_id
                    );
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };

        if should_stop_bots {
            self.stop_bot_simulation(game_id).await;
        }
    }

    /// Start bot simulation for a game
    async fn start_bot_simulation(&self, game_id: &str) {
        // Create a cancellation channel for this game's bots
        let (cancel_tx, _) = broadcast::channel(1);

        // Store the cancellation sender
        {
            let mut bot_tasks = self.bot_tasks.write().await;
            bot_tasks.insert(game_id.to_string(), cancel_tx.clone());
        }

        // Start the bot processing task
        let game_service = self.game_service.clone();
        let broadcaster = self.broadcaster.clone();
        let game_id_owned = game_id.to_string();
        let active_connections = self.active_connections.clone();

        tokio::spawn(async move {
            let mut cancel_rx = cancel_tx.subscribe();

            // Small delay to ensure WebSocket subscription is fully established
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

            // Process bot turns with cancellation support
            Self::process_bot_turns_with_cancellation(
                &game_service,
                &broadcaster,
                &game_id_owned,
                &active_connections,
                &mut cancel_rx,
            )
            .await;
        });
    }

    /// Stop bot simulation for a game
    async fn stop_bot_simulation(&self, game_id: &str) {
        let mut bot_tasks = self.bot_tasks.write().await;
        if let Some(cancel_tx) = bot_tasks.remove(game_id) {
            // Send cancellation signal (ignore if no receivers)
            let _ = cancel_tx.send(());
            log::info!("🛑 Stopped bot simulation for game {}", game_id);
        }
    }

    /// Handle incoming text messages
    async fn handle_text_message(
        game_service: &GameService,
        broadcaster: &broadcast::Sender<(GameId, WsMessage)>,
        game_id: &str,
        text: String,
        service: &WebSocketService,
    ) -> CatanResult<()> {
        // Debug: Log the exact message received
        log::debug!("🔍 WebSocket received raw message: {}", text);

        // Try to parse as a generic JSON first to see the structure
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&text) {
            log::debug!("🔍 Parsed as JSON: {:#}", json_value);
            if let Some(msg_type) = json_value.get("type") {
                log::debug!("🔍 Message type: {}", msg_type);
            }
        }

        // Parse the incoming message
        let ws_message: WsMessage = serde_json::from_str(&text).map_err(|e| {
            log::error!("❌ Failed to deserialize WebSocket message: {}", e);
            log::error!("❌ Raw message was: {}", text);

            // Try to give more specific error information
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&text) {
                log::error!("❌ JSON structure: {:#}", json_value);
                if let Some(msg_type) = json_value.get("type") {
                    log::error!("❌ Message type was: {}", msg_type);
                }

                // Check if this looks like the old message format
                if json_value.get("data").is_some() {
                    log::error!(
                        "❌ This looks like the old WebSocket message format with 'data' field"
                    );
                    log::error!("❌ New format expects 'action' field for player_action messages");
                }
            }

            crate::errors::CatanError::Network(crate::errors::NetworkError::DeserializationFailed {
                details: format!("Message deserialization failed: {e}"),
            })
        })?;

        match ws_message {
            WsMessage::PlayerAction { action } => {
                log::info!("🎯 Processing action for game {}: {:?}", game_id, action);

                // Use the PlayerAction enum directly - no conversion needed!
                log::info!("✅ Received PlayerAction enum: {:?}", action);

                // Process the action through the game service (use game_id from function parameter)
                match game_service
                    .process_action(game_id, "player_0", action)
                    .await
                {
                    Ok(events) => {
                        log::info!("✅ Action processed successfully");

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

                        // Restart bot simulation after human action (only if connections exist)
                        if service.has_active_connections(game_id).await {
                            log::debug!(
                                "🔄 Restarting bot simulation after human action for game {}",
                                game_id
                            );
                            service.start_bot_simulation(game_id).await;
                        }
                    }
                    Err(e) => {
                        log::error!("❌ Action processing failed: {}", e);
                        let error_msg = WsMessage::Error {
                            message: format!("Action failed: {e}"),
                        };
                        let _ = broadcaster.send((game_id.to_string(), error_msg));
                    }
                }
            }
            WsMessage::GetGameState => {
                log::info!("📡 Getting game state for game {}", game_id);

                match game_service.get_game(game_id).await {
                    Ok(game) => {
                        let state_msg = WsMessage::GameState { game };
                        let _ = broadcaster.send((game_id.to_string(), state_msg));
                    }
                    Err(e) => {
                        log::error!("❌ Failed to get game state: {}", e);
                        let error_msg = WsMessage::Error {
                            message: format!("Failed to get game: {e}"),
                        };
                        let _ = broadcaster.send((game_id.to_string(), error_msg));
                    }
                }
            }
            // ✅ REMOVED: BotAction handler - Bot actions are now automatic
            WsMessage::CreateGame { mode, num_players } => {
                log::info!(
                    "🎮 Creating new game: mode={}, players={}",
                    mode,
                    num_players
                );

                // Determine bot type from mode
                let bot_type = match mode.as_str() {
                    "HUMAN_VS_CATANATRON" => "mcts",
                    "RANDOM_BOTS" => "random",
                    "CATANATRON_BOTS" => "mcts",
                    _ => "random",
                };

                match game_service.create_game(num_players, bot_type).await {
                    Ok(new_game_id) => {
                        log::info!("✅ Game created successfully: {}", new_game_id);

                        // Get the created game
                        match game_service.get_game(&new_game_id).await {
                            Ok(game) => {
                                let created_msg = WsMessage::GameCreated {
                                    game_id: new_game_id.clone(),
                                    game,
                                };
                                let _ = broadcaster.send((new_game_id, created_msg));
                            }
                            Err(e) => {
                                log::error!("❌ Failed to get created game: {}", e);
                                let error_msg = WsMessage::Error {
                                    message: format!("Failed to get created game: {e}"),
                                };
                                let _ = broadcaster.send((game_id.to_string(), error_msg));
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("❌ Game creation failed: {}", e);
                        let error_msg = WsMessage::Error {
                            message: format!("Game creation failed: {e}"),
                        };
                        let _ = broadcaster.send((game_id.to_string(), error_msg));
                    }
                }
            }
            _ => {
                log::debug!("Received unhandled message type: {:?}", ws_message);
            }
        }

        Ok(())
    }

    /// Check if a game has active connections
    async fn has_active_connections(&self, game_id: &str) -> bool {
        let connections = self.active_connections.read().await;
        connections
            .get(game_id)
            .is_some_and(|conns| !conns.is_empty())
    }

    // ✅ REMOVED: ensure_bot_simulation_running() - now using event-driven bot simulation

    /// Process bot turns with cancellation support
    async fn process_bot_turns_with_cancellation(
        game_service: &GameService,
        broadcaster: &broadcast::Sender<(GameId, WsMessage)>,
        game_id: &str,
        active_connections: &Arc<RwLock<HashMap<GameId, HashSet<String>>>>,
        cancel_rx: &mut broadcast::Receiver<()>,
    ) {
        loop {
            // Check if we should continue (has active connections)
            let has_connections = {
            let connections = active_connections.read().await;
            connections
                .get(game_id)
                .is_some_and(|conns| !conns.is_empty())
            };

            if !has_connections {
                log::info!(
                    "🛑 No active connections for game {}, stopping bot simulation",
                    game_id
                );
                break;
            }

            // Check for cancellation
            if cancel_rx.try_recv().is_ok() {
                log::info!("🛑 Bot simulation cancelled for game {}", game_id);
                break;
            }

            // Try to process a bot turn
            match game_service.process_bot_turn(game_id).await {
                Ok(Some(_events)) => {
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
                        if let Err(e) = broadcaster.send((game_id.to_string(), update_msg)) {
                            log::error!(
                                "Failed to broadcast game update for game {}: {:?}",
                                game_id,
                                e
                            );
                        }
                    }
                }
                Ok(None) => {
                    // No more bot moves needed - exit the loop instead of continuous polling
                    log::debug!(
                        "🤖 No bot actions needed for game {}, ending bot simulation loop",
                        game_id
                    );
                    break;
                }
                Err(e) => {
                    log::error!("Bot processing error for game {}: {}", game_id, e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
                }
            }
        }

        log::info!("🏁 Bot simulation ended for game {}", game_id);
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
        let json = serde_json::to_string(message).map_err(axum::Error::new)?;

        sender
            .send(Message::Text(json.into()))
            .await
            .map_err(axum::Error::new)
    }

    /// Get the broadcaster for sending messages to all clients
    pub fn broadcaster(&self) -> broadcast::Sender<(GameId, WsMessage)> {
        self.broadcaster.clone()
    }
}

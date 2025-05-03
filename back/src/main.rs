mod enums;
mod game;
mod global_state;
mod map_instance;
mod map_template;
mod ordered_hashmap;
mod state;
mod state_vector;

use axum::http::Method;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use log;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex};
use tokio::time::sleep;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

use crate::game::{simulate_bot_game, start_human_vs_catanatron, Game};

// Game-related structures
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum GameMode {
    HumanVsCatanatron,
    RandomBots,
    CatanatronBots,
}

#[derive(Debug, Deserialize)]
struct GameConfig {
    mode: GameMode,
    num_players: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum GameStatus {
    Waiting,
    InProgress,
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GameState {
    id: String,
    #[serde(rename = "status")]
    status: GameStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    game: Option<Game>,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_prompt: Option<String>,
    #[serde(default)]
    bot_colors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    winning_color: Option<String>,
}

// WebSocket message types
#[derive(Debug, Serialize, Deserialize, Clone)]
struct PlayerAction {
    action: String,
    edge_id: Option<String>,
    node_id: Option<String>,
    coordinate: Option<map_template::Coordinate>,
    resource: Option<String>,
    resources: Option<Vec<String>>,
    target_color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ActionResult {
    success: bool,
    message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
enum WsMessage {
    #[serde(rename = "game_state")]
    GameState(GameState),
    #[serde(rename = "error")]
    Error(String),
    #[serde(rename = "greeting")]
    Greeting(String),
    #[serde(rename = "player_action")]
    PlayerAction(PlayerAction),
    #[serde(rename = "action_result")]
    ActionResult(ActionResult),
    #[serde(rename = "bot_action")]
    BotAction(PlayerAction),
}

// Application state
struct AppState {
    games: Mutex<HashMap<String, GameState>>,
    tx: broadcast::Sender<(String, WsMessage)>,
}

type SharedState = Arc<AppState>;

// API Routes

// Get hello world
async fn hello_world() -> &'static str {
    "Hello from Catanatron backend!"
}

// Create a new game
async fn create_game(
    State(state): State<SharedState>,
    Json(config): Json<GameConfig>,
) -> Result<Json<GameState>, StatusCode> {
    log::info!(
        "Creating game with mode: {:?}, players: {}",
        config.mode,
        config.num_players
    );

    let game_id = Uuid::new_v4().to_string();

    // Create the appropriate game based on mode
    let actual_game = match config.mode {
        GameMode::RandomBots => Some(simulate_bot_game(config.num_players)),
        GameMode::HumanVsCatanatron => Some(start_human_vs_catanatron(
            "Human Player".to_string(),
            config.num_players - 1,
        )),
        GameMode::CatanatronBots => Some(simulate_bot_game(config.num_players)),
    };

    // Convert to view for serialization
    let game_view = actual_game.clone();

    // Determine which players are bots
    let bot_colors = if let Some(game) = &game_view {
        match config.mode {
            GameMode::HumanVsCatanatron => {
                // All players except the first one are bots
                game.players
                    .iter()
                    .skip(1)
                    .map(|p| p.color.clone())
                    .collect()
            }
            GameMode::RandomBots | GameMode::CatanatronBots => {
                // All players are bots
                game.players.iter().map(|p| p.color.clone()).collect()
            }
        }
    } else {
        Vec::new()
    };

    let game_state = GameState {
        id: game_id.clone(),
        status: GameStatus::Waiting,
        game: game_view.clone(),
        current_color: game_view
            .as_ref()
            .and_then(|g| g.players.get(0).map(|p| p.color.clone())),
        current_prompt: Some("PLAY_TURN".to_string()),
        bot_colors,
        winning_color: None,
    };

    {
        let mut games = state.games.lock().await;
        games.insert(game_id.clone(), game_state.clone());
    }

    // Broadcast the game creation
    let _ = state
        .tx
        .send((game_id.clone(), WsMessage::GameState(game_state.clone())));

    // If it's a bot game, start a background task to simulate the game
    if config.mode == GameMode::RandomBots || config.mode == GameMode::CatanatronBots {
        let state_clone = state.clone();
        let game_id_clone = game_id.clone();

        // Create player names for the simulation
        let num_players = config.num_players;
        let colors = vec!["red", "blue", "white", "orange"];
        let players: Vec<game::Player> = (0..num_players)
            .map(|i| game::Player {
                id: format!("player_{}", i),
                name: format!("Bot {}", i + 1),
                color: colors[i as usize % colors.len()].to_string(),
                resources: HashMap::new(),
                dev_cards: Vec::new(),
                knights_played: 0,
                victory_points: 0,
                longest_road: false,
                largest_army: false,
            })
            .collect();

        tokio::spawn(async move {
            // Wait a moment before starting the game
            sleep(Duration::from_secs(1)).await;

            // Create a direct GameView for simulation
            let mut sim_view = simulate_bot_game(0); // placeholder: create a basic Game for initial view
                                                     // TODO: populate sim_view fields properly

            // Update game status to in progress
            {
                let mut games = state_clone.games.lock().await;
                if let Some(game_state) = games.get_mut(&game_id_clone) {
                    game_state.status = GameStatus::InProgress;
                    game_state.game = Some(sim_view.clone());

                    // Broadcast the updated state
                    let _ = state_clone.tx.send((
                        game_id_clone.clone(),
                        WsMessage::GameState(game_state.clone()),
                    ));
                }
            }

            // Simulate game moves - simple simulation
            for _ in 0..10 {
                sleep(Duration::from_millis(500)).await;

                // Simple simulation: increment turns
                sim_view.turns += 1;
                sim_view.current_player_index =
                    (sim_view.current_player_index + 1) % sim_view.players.len();

                // Broadcast update
                let update_msg = WsMessage::GameState(GameState {
                    id: game_id_clone.clone(),
                    status: GameStatus::InProgress,
                    game: Some(sim_view.clone()),
                    current_color: sim_view
                        .players
                        .get(sim_view.current_player_index)
                        .map(|p| p.color.clone()),
                    current_prompt: Some("PLAY_TURN".to_string()),
                    bot_colors: sim_view.players.iter().map(|p| p.color.clone()).collect(),
                    winning_color: None,
                });
                let _ = state_clone.tx.send((game_id_clone.clone(), update_msg));

                // Random chance of winning
                if rand::random::<u8>() % 20 == 0 {
                    // Use the game_state enum instead of a direct winner field
                    sim_view.game_state = game::GameState::Finished {
                        winner: sim_view.players[0].name.clone(),
                    };
                    break;
                }
            }

            // Update the game state with the final game state
            let mut games = state_clone.games.lock().await;
            if let Some(game_state) = games.get_mut(&game_id_clone) {
                game_state.game = Some(sim_view.clone());
                game_state.status = if let game::GameState::Finished { .. } = sim_view.game_state {
                    GameStatus::Finished
                } else {
                    GameStatus::InProgress
                };

                // Broadcast final update if game finished
                if let game::GameState::Finished { .. } = sim_view.game_state {
                    let _ = state_clone.tx.send((
                        game_id_clone.clone(),
                        WsMessage::GameState(game_state.clone()),
                    ));
                }
            }
        });
    }

    // Create a response without the game field to reduce data size
    let response = GameState {
        id: game_state.id.clone(),
        status: game_state.status.clone(),
        game: None,
        current_color: game_state.current_color.clone(),
        current_prompt: game_state.current_prompt.clone(),
        bot_colors: game_state.bot_colors.clone(),
        winning_color: game_state.winning_color.clone(),
    };

    Ok(Json(response))
}

// Get a game state
async fn get_game(
    State(state): State<SharedState>,
    Path(game_id): Path<String>,
) -> Result<Json<GameState>, StatusCode> {
    log::info!("Getting game with ID: {}", game_id);

    let games = state.games.lock().await;

    if let Some(game_state) = games.get(&game_id) {
        Ok(Json(game_state.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// WebSocket handler for game updates
async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(game_id): Path<String>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    // Clone state and game_id here so the closure is 'static and Send
    let state_clone = state.clone();
    let game_id_clone = game_id.clone();

    ws.on_upgrade(move |socket| async move {
        ws_game_connection(socket, game_id_clone, state_clone).await
    })
}

// Handle WebSocket connection for a specific game
async fn ws_game_connection(socket: WebSocket, game_id: String, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();

    // Send an immediate greeting
    let greeting = WsMessage::Greeting("Hello from the Catanatron backend!".to_string());
    if let Ok(json) = serde_json::to_string(&greeting) {
        let _ = sender.send(Message::Text(json.into())).await;
    }

    // Check if the game exists and get initial game state
    let initial_state = {
        let games = state.games.lock().await;
        if !games.contains_key(&game_id) {
            let error_msg = WsMessage::Error(format!("Game with ID {} not found", game_id));
            let _ = sender
                .send(Message::Text(
                    serde_json::to_string(&error_msg).unwrap().into(),
                ))
                .await;
            return;
        }

        // Get initial state if game exists
        games
            .get(&game_id)
            .map(|game_state| WsMessage::GameState(game_state.clone()))
    };

    // Send the initial game state
    if let Some(state_msg) = initial_state {
        if let Ok(json) = serde_json::to_string(&state_msg) {
            if sender.send(Message::Text(json.into())).await.is_err() {
                return; // Client disconnected
            }
        }
    }

    // Subscribe to the broadcast channel
    let mut rx = state.tx.subscribe();

    // Create a task to listen for broadcasts
    let game_id_clone = game_id.clone();
    let mut send_task = tokio::spawn(async move {
        while let Ok((msg_game_id, msg)) = rx.recv().await {
            // Only forward messages for this game
            if msg_game_id == game_id_clone {
                match serde_json::to_string(&msg) {
                    Ok(json) => {
                        if sender.send(Message::Text(json.into())).await.is_err() {
                            break; // Client disconnected
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to serialize WebSocket message: {}", e);
                    }
                }
            }
        }
    });

    // Listen for messages from the client
    let state_for_recv = state.clone();
    let game_id_for_recv = game_id.clone();
    
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(message)) = receiver.next().await {
            if let Message::Close(_) = message {
                break;
            }

            if let Message::Text(text) = message {
                // Try to parse the message as a PlayerAction
                if let Ok(player_action_msg) = serde_json::from_str::<WsMessage>(&text) {
                    if let WsMessage::PlayerAction(action) = player_action_msg {
                        log::info!("Received player action: {:?}", action);
                        
                        // Process the action and update game state
                        let result = process_player_action(&action, &game_id_for_recv, &state_for_recv).await;
                        
                        // Broadcast the result
                        let _ = state_for_recv.tx.send((
                            game_id_for_recv.clone(),
                            WsMessage::ActionResult(result.clone()),
                        ));

                        // If action was successful, check if it's a bot's turn and make a move
                        if result.success {
                            process_bot_turn(&game_id_for_recv, &state_for_recv).await;
                        }
                    }
                } else {
                    log::warn!("Received invalid message: {}", text);
                    let error_msg = WsMessage::Error("Invalid message format".to_string());
                    let _ = state_for_recv.tx.send((game_id_for_recv.clone(), error_msg));
                }
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
        },
        _ = (&mut recv_task) => {
            send_task.abort();
        },
    }

    log::info!("WebSocket connection closed for game {}", game_id);
}

// Process player actions and update game state
async fn process_player_action(
    action: &PlayerAction,
    game_id: &str,
    state: &SharedState,
) -> ActionResult {
    // Get the game state
    let mut games = state.games.lock().await;
    let game_state = match games.get_mut(game_id) {
        Some(game_state) => game_state,
        None => {
            return ActionResult {
                success: false,
                message: format!("Game with ID {} not found", game_id),
            }
        }
    };

    // Check if the game is still in progress
    if game_state.status != GameStatus::InProgress {
        return ActionResult {
            success: false,
            message: "Game is not in progress".to_string(),
        };
    }

    let game = match &mut game_state.game {
        Some(game) => game,
        None => {
            return ActionResult {
                success: false,
                message: "Game state not found".to_string(),
            }
        }
    };

    // Ensure it's a human player's turn
    let current_player_color = game.players[game.current_player_index].color.clone();
    if game_state.bot_colors.contains(&current_player_color) {
        return ActionResult {
            success: false,
            message: "It's not your turn".to_string(),
        };
    }

    // Process the action based on type
    match action.action.as_str() {
        "build_road" => {
            if let Some(edge_id) = &action.edge_id {
                if let Some(edge) = game.board.edges.get_mut(edge_id) {
                    if edge.color.is_none() {
                        // Check resources (simplified for demo)
                        let player = &mut game.players[game.current_player_index];
                        
                        // In a real implementation, check player's resources
                        // For now, just set the edge color
                        edge.color = Some(player.color.clone());
                        
                        // Update game state and broadcast update
                        game_state.current_prompt = Some("PLAY_TURN".to_string());
                        
                        return ActionResult {
                            success: true,
                            message: "Road built successfully".to_string(),
                        };
                    } else {
                        return ActionResult {
                            success: false,
                            message: "Edge already has a road".to_string(),
                        };
                    }
                }
            }
            
            ActionResult {
                success: false,
                message: "Invalid edge ID".to_string(),
            }
        },
        "build_settlement" => {
            if let Some(node_id) = &action.node_id {
                if let Some(node) = game.board.nodes.get_mut(node_id) {
                    if node.building.is_none() {
                        // Check resources (simplified for demo)
                        let player = &mut game.players[game.current_player_index];
                        
                        // In a real implementation, check player's resources
                        // For now, just set the node building
                        node.building = Some("Settlement".to_string());
                        node.color = Some(player.color.clone());
                        
                        // Update game state and broadcast update
                        game_state.current_prompt = Some("PLAY_TURN".to_string());
                        
                        return ActionResult {
                            success: true,
                            message: "Settlement built successfully".to_string(),
                        };
                    } else {
                        return ActionResult {
                            success: false,
                            message: "Node already has a building".to_string(),
                        };
                    }
                }
            }
            
            ActionResult {
                success: false,
                message: "Invalid node ID".to_string(),
            }
        },
        "build_city" => {
            if let Some(node_id) = &action.node_id {
                if let Some(node) = game.board.nodes.get_mut(node_id) {
                    if let Some(building) = &node.building {
                        if building == "Settlement" && node.color.as_ref() == Some(&current_player_color) {
                            // Check resources (simplified for demo)
                            
                            // Update the node
                            node.building = Some("City".to_string());
                            
                            // Update game state and broadcast update
                            game_state.current_prompt = Some("PLAY_TURN".to_string());
                            
                            return ActionResult {
                                success: true,
                                message: "City built successfully".to_string(),
                            };
                        } else {
                            return ActionResult {
                                success: false,
                                message: "Node does not have your settlement".to_string(),
                            };
                        }
                    } else {
                        return ActionResult {
                            success: false,
                            message: "Node does not have a settlement".to_string(),
                        };
                    }
                }
            }
            
            ActionResult {
                success: false,
                message: "Invalid node ID".to_string(),
            }
        },
        "roll_dice" => {
            if game.dice_rolled {
                return ActionResult {
                    success: false,
                    message: "Dice already rolled this turn".to_string(),
                };
            }
            
            // Roll dice
            let d1 = rand::thread_rng().gen_range(1..=6);
            let d2 = rand::thread_rng().gen_range(1..=6);
            let roll_value = d1 + d2;
            
            // Set dice roll in game state
            game.dice_rolled = true;
            game.current_dice_roll = Some((d1, d2));
            
            // TODO: Handle resource distribution based on roll
            
            // Check for robber (7)
            if roll_value == 7 {
                game_state.current_prompt = Some("MOVE_ROBBER".to_string());
            }
            
            return ActionResult {
                success: true,
                message: format!("Rolled {} and {}, total {}", d1, d2, roll_value),
            };
        },
        "end_turn" => {
            // Move to the next player
            game.current_player_index = (game.current_player_index + 1) % game.players.len();
            game.dice_rolled = false;
            game.turns += 1;
            
            // Update current color
            game_state.current_color = Some(game.players[game.current_player_index].color.clone());
            game_state.current_prompt = Some("PLAY_TURN".to_string());
            
            return ActionResult {
                success: true,
                message: "Turn ended".to_string(),
            };
        },
        "move_robber" => {
            if let Some(coordinate) = &action.coordinate {
                // Move robber to new location
                game.board.robber_coordinate = coordinate.clone();
                
                // If there's a target player, steal a resource (simplified)
                if let Some(target_color) = &action.target_color {
                    // In a real implementation, steal a random resource
                    // For this demo, we'll just acknowledge the theft
                    log::info!("Player stole from {}", target_color);
                }
                
                // Update prompt
                game_state.current_prompt = Some("PLAY_TURN".to_string());
                
                return ActionResult {
                    success: true,
                    message: "Robber moved successfully".to_string(),
                };
            }
            
            ActionResult {
                success: false,
                message: "Invalid coordinate for robber".to_string(),
            }
        },
        _ => ActionResult {
            success: false,
            message: format!("Unsupported action: {}", action.action),
        },
    }
}

// Process bot turns
async fn process_bot_turn(game_id: &str, state: &SharedState) {
    // Get the game state
    let mut games = state.games.lock().await;
    let game_state = match games.get_mut(game_id) {
        Some(game_state) => game_state,
        None => return,
    };

    // Check if the game is still in progress
    if game_state.status != GameStatus::InProgress {
        return;
    }

    let game = match &mut game_state.game {
        Some(game) => game,
        None => return,
    };

    // Check if it's a bot's turn
    let current_player_color = game.players[game.current_player_index].color.clone();
    if !game_state.bot_colors.contains(&current_player_color) {
        return; // Not a bot's turn
    }

    // Add a small delay to simulate "thinking"
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // Broadcast that the bot is thinking
    let bot_thinking_msg = WsMessage::BotAction(PlayerAction {
        action: "thinking".to_string(),
        edge_id: None,
        node_id: None,
        coordinate: None,
        resource: None,
        resources: None,
        target_color: None,
    });
    let _ = state.tx.send((game_id.to_string(), bot_thinking_msg));

    // Get a copy of the current game state for the bot to analyze
    // In a real implementation, we would use the AlphaBetaPlayer to make decisions
    
    // Simulate a bot action (simplified - just roll and end turn)
    // Roll dice if not rolled yet
    if !game.dice_rolled {
        let d1 = rand::thread_rng().gen_range(1..=6);
        let d2 = rand::thread_rng().gen_range(1..=6);
        let roll_value = d1 + d2;
        
        // Set dice roll in game state
        game.dice_rolled = true;
        game.current_dice_roll = Some((d1, d2));
        
        // Broadcast the bot's roll action
        let bot_roll_action = WsMessage::BotAction(PlayerAction {
            action: "roll_dice".to_string(),
            edge_id: None,
            node_id: None,
            coordinate: None,
            resource: None,
            resources: None,
            target_color: None,
        });
        let _ = state.tx.send((game_id.to_string(), bot_roll_action));
        
        // If we rolled a 7, move the robber to a random coordinate
        if roll_value == 7 {
            // For simplicity, just move to a fixed coordinate
            let robber_coord = Coordinate { x: 0, y: 0, z: 0 };
            game.board.robber_coordinate = robber_coord.clone();
            
            // Broadcast the robber move
            let bot_robber_action = WsMessage::BotAction(PlayerAction {
                action: "move_robber".to_string(),
                edge_id: None,
                node_id: None,
                coordinate: Some(robber_coord),
                resource: None,
                resources: None,
                target_color: None,
            });
            let _ = state.tx.send((game_id.to_string(), bot_robber_action));
        }
        
        // Wait a bit more after rolling
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
    
    // Try to build something based on available resources
    // In this simplified version, we'll randomly decide to build a road, settlement, or city
    let build_action = rand::thread_rng().gen_range(0..4);
    
    if build_action == 0 {
        // Try to build a road
        // Find an empty edge
        let empty_edges: Vec<String> = game.board.edges.iter()
            .filter(|(_, edge)| edge.color.is_none())
            .map(|(id, _)| id.clone())
            .collect();
        
        if !empty_edges.is_empty() {
            // Choose a random edge
            let edge_id = &empty_edges[rand::thread_rng().gen_range(0..empty_edges.len())];
            
            // Build the road
            if let Some(edge) = game.board.edges.get_mut(edge_id) {
                edge.color = Some(current_player_color.clone());
                
                // Broadcast the bot's build action
                let bot_build_action = WsMessage::BotAction(PlayerAction {
                    action: "build_road".to_string(),
                    edge_id: Some(edge_id.clone()),
                    node_id: None,
                    coordinate: None,
                    resource: None,
                    resources: None,
                    target_color: None,
                });
                let _ = state.tx.send((game_id.to_string(), bot_build_action));
                
                // Wait a bit after building
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }
    } else if build_action == 1 {
        // Try to build a settlement
        // Find an empty node
        let empty_nodes: Vec<String> = game.board.nodes.iter()
            .filter(|(_, node)| node.building.is_none())
            .map(|(id, _)| id.clone())
            .collect();
        
        if !empty_nodes.is_empty() {
            // Choose a random node
            let node_id = &empty_nodes[rand::thread_rng().gen_range(0..empty_nodes.len())];
            
            // Build the settlement
            if let Some(node) = game.board.nodes.get_mut(node_id) {
                node.building = Some("Settlement".to_string());
                node.color = Some(current_player_color.clone());
                
                // Broadcast the bot's build action
                let bot_build_action = WsMessage::BotAction(PlayerAction {
                    action: "build_settlement".to_string(),
                    edge_id: None,
                    node_id: Some(node_id.clone()),
                    coordinate: None,
                    resource: None,
                    resources: None,
                    target_color: None,
                });
                let _ = state.tx.send((game_id.to_string(), bot_build_action));
                
                // Wait a bit after building
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }
    } else if build_action == 2 {
        // Try to upgrade a settlement to a city
        // Find own settlements
        let own_settlements: Vec<String> = game.board.nodes.iter()
            .filter(|(_, node)| {
                node.building.as_ref().map_or(false, |b| b == "Settlement") &&
                node.color.as_ref().map_or(false, |c| c == &current_player_color)
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        if !own_settlements.is_empty() {
            // Choose a random settlement
            let node_id = &own_settlements[rand::thread_rng().gen_range(0..own_settlements.len())];
            
            // Upgrade to city
            if let Some(node) = game.board.nodes.get_mut(node_id) {
                node.building = Some("City".to_string());
                
                // Broadcast the bot's build action
                let bot_build_action = WsMessage::BotAction(PlayerAction {
                    action: "build_city".to_string(),
                    edge_id: None,
                    node_id: Some(node_id.clone()),
                    coordinate: None,
                    resource: None,
                    resources: None,
                    target_color: None,
                });
                let _ = state.tx.send((game_id.to_string(), bot_build_action));
                
                // Wait a bit after building
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }
    }

    // End turn
    game.current_player_index = (game.current_player_index + 1) % game.players.len();
    game.dice_rolled = false;
    game.turns += 1;
    
    // Update current color and prompt
    game_state.current_color = Some(game.players[game.current_player_index].color.clone());
    game_state.current_prompt = Some("PLAY_TURN".to_string());
    
    // Broadcast the bot's end turn action
    let bot_end_turn_action = WsMessage::BotAction(PlayerAction {
        action: "end_turn".to_string(),
        edge_id: None,
        node_id: None,
        coordinate: None,
        resource: None,
        resources: None,
        target_color: None,
    });
    let _ = state.tx.send((game_id.to_string(), bot_end_turn_action));
    
    // Broadcast the updated game state
    let _ = state.tx.send((
        game_id.to_string(),
        WsMessage::GameState(game_state.clone()),
    ));
    
    // If it's still a bot's turn, process the next bot
    let next_player_color = game.players[game.current_player_index].color.clone();
    if game_state.bot_colors.contains(&next_player_color) {
        // Drop the lock before recursively calling
        drop(games);
        process_bot_turn(game_id, state).await;
    }
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    // Initialize logger
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    // Create shared state with broadcast channel
    let (tx, _rx) = broadcast::channel(100);
    let state = Arc::new(AppState {
        games: Mutex::new(HashMap::new()),
        tx,
    });

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any)
        .allow_origin(Any);

    // Create router with routes
    let app = Router::new()
        .route("/", get(hello_world))
        .route("/games", post(create_game))
        .route("/games/{game_id}", get(get_game))
        .route("/ws/games/{game_id}", get(ws_handler))
        .with_state(state)
        .layer(cors);

    log::info!("Starting Catanatron backend server");

    Ok(app.into())
}

mod enums;
mod game;
mod global_state;
mod map_instance;
mod map_template;
mod ordered_hashmap;
mod state_vector;
mod state;

use axum::http::{HeaderValue, Method};
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

use crate::game::{simulate_bot_game, start_human_vs_catanatron, Game, GameAction, GameBoard, Player};

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
    status: GameStatus,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    game: Option<Game>,
}

// WebSocket message types
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
enum WsMessage {
    GameState(GameState),
    Error(String),
    Greeting(String),
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

    let game_state = GameState {
        id: game_id.clone(),
        status: GameStatus::Waiting,
        game: game_view.clone(),
    };

    {
        let mut games = state.games.lock().await;
        games.insert(game_id.clone(), game_state.clone());
    }

    // Broadcast the game creation
    let _ = state.tx.send((
        game_id.clone(),
        WsMessage::GameState(GameState {
            id: game_id.clone(),
            status: GameStatus::Waiting,
            game: game_view,
        }),
    ));

    // If it's a bot game, start a background task to simulate the game
    if config.mode == GameMode::RandomBots || config.mode == GameMode::CatanatronBots {
        let state_clone = state.clone();
        let game_id_clone = game_id.clone();
        
        // Create player names for the simulation
        let num_players = config.num_players;
        let colors = vec!["red", "blue", "white", "orange"];
        let players: Vec<game::Player> = (0..num_players)
            .map(|i| {
                game::Player {
                    id: format!("player_{}", i),
                    name: format!("Bot {}", i + 1),
                    color: colors[i as usize % colors.len()].to_string(),
                    resources: HashMap::new(),
                    dev_cards: Vec::new(),
                    knights_played: 0,
                    victory_points: 0,
                    longest_road: false,
                    largest_army: false,
                }
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
                sim_view.current_player_index = (sim_view.current_player_index + 1) % sim_view.players.len();

                // Broadcast update
                let update_msg = WsMessage::GameState(GameState {
                    id: game_id_clone.clone(),
                    status: GameStatus::InProgress,
                    game: Some(sim_view.clone()),
                });
                let _ = state_clone.tx.send((game_id_clone.clone(), update_msg));

                // Random chance of winning
                if rand::random::<u8>() % 20 == 0 {
                    // Use the game_state enum instead of a direct winner field
                    sim_view.game_state = game::GameState::Finished { 
                        winner: sim_view.players[0].name.clone() 
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
        id: game_state.id,
        status: game_state.status,
        game: None,
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
        games.get(&game_id).map(|game_state| {
            WsMessage::GameState(game_state.clone())
        })
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
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(message)) = receiver.next().await {
            if let Message::Close(_) = message {
                break;
            }

            // For now, ignore client messages
            // In the future, you could implement client commands here
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

// Import from lib.rs instead of individual modules
use catan::{
    WebSocketManager, GameConfiguration, 
    CatanError, CatanResult, GameState
};

use axum::http::Method;
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use log;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

// Game configuration
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

// Application state
struct AppState {
    websocket_manager: Arc<WebSocketManager>,
}

// API Routes

// Get hello world
async fn hello_world() -> &'static str {
    "Hello from Catan backend!"
}

// Create a new game
async fn create_game(
    State(state): State<Arc<AppState>>,
    Json(config): Json<GameConfig>,
) -> Result<Json<GameState>, StatusCode> {
    log::info!(
        "Creating game with mode: {:?}, players: {}",
        config.mode,
        config.num_players
    );

    // Delegate game creation to WebSocketManager
    match state.websocket_manager.create_game(config.num_players, "random").await {
        Ok(game_id) => {
            // Get the game state
            match state.websocket_manager.get_game_state(&game_id).await {
                Ok(game_state) => Ok(Json(game_state)),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Get a game state
async fn get_game(
    State(state): State<Arc<AppState>>,
    Path(game_id): Path<String>,
) -> Result<Json<GameState>, StatusCode> {
    log::info!("Getting game with ID: {}", game_id);

    // Get the game from WebSocketManager
    match state.websocket_manager.get_game_state(&game_id).await {
        Ok(game_state) => Ok(Json(game_state)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

// WebSocket handler for game updates
async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(game_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Let the WebSocketManager handle the connection
    ws.on_upgrade(move |socket| async move {
        state.websocket_manager.handle_connection(socket, game_id).await
    })
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    // Initialize logger
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    // Create WebSocketManager
    let websocket_manager = Arc::new(WebSocketManager::new());

    // Create shared application state
    let state = Arc::new(AppState {
        websocket_manager: websocket_manager.clone(),
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
        .route("/games/:game_id", get(get_game))
        .route("/ws/games/:game_id", get(ws_handler))
        .with_state(state)
        .layer(cors);

    log::info!("Starting Catan backend server");

    Ok(app.into())
}

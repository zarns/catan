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

use catan::application::GameService;
use catan::websocket_service::WebSocketService;
use catan::game::Game;

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

// Clean application state - single dependency injection point
struct AppState {
    game_service: Arc<GameService>,
    websocket_service: Arc<WebSocketService>,
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
) -> Result<Json<Game>, StatusCode> {
    log::info!(
        "Creating game with mode: {:?}, players: {}",
        config.mode,
        config.num_players
    );

    // Determine bot type from config
    let bot_type = match config.mode {
        GameMode::RandomBots => "random",
        GameMode::HumanVsCatanatron => "human", // First player human, rest bots
        GameMode::CatanatronBots => "random", // All bots for now
    };

    // Delegate to game service (clean separation)
    match state.game_service.create_game(config.num_players, bot_type).await {
        Ok(game_id) => {
            // Return the full game object
            match state.game_service.get_game(&game_id).await {
                Ok(game) => Ok(Json(game)),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Get a game
async fn get_game(
    State(state): State<Arc<AppState>>,
    Path(game_id): Path<String>,
) -> Result<Json<Game>, StatusCode> {
    log::info!("Getting game with ID: {}", game_id);

    // Delegate to game service
    match state.game_service.get_game(&game_id).await {
        Ok(game) => Ok(Json(game)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

// WebSocket handler for game updates
async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(game_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Delegate to WebSocket service (clean separation)
    ws.on_upgrade(move |socket| async move {
        state.websocket_service.handle_connection(socket, game_id).await
    })
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    // Initialize logger (only if not already initialized by shuttle)
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    let _ = env_logger::try_init(); // Use try_init to avoid double initialization

    // Create clean service layer architecture
    let game_service = Arc::new(GameService::new());
    let websocket_service = Arc::new(WebSocketService::new(game_service.clone()));

    // Create shared application state with dependency injection
    let state = Arc::new(AppState {
        game_service,
        websocket_service,
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

    log::info!("Starting Catan backend server");

    Ok(app.into())
}

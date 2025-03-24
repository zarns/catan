mod game;

use axum::{
    routing::{get, post},
    http::StatusCode,
    Router,
    Json,
    extract::{Path, State},
};
use axum::http::{Method, HeaderValue};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tower_http::cors::{CorsLayer, Any};
use uuid::Uuid;
use log;

use crate::game::{Game, simulate_bot_game, start_human_vs_catanatron};

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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
enum GameStatus {
    Waiting,
    InProgress,
    Finished,
}

#[derive(Debug, Clone, Serialize)]
struct GameState {
    id: String,
    status: GameStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    game: Option<Game>,
}

// Application state
type GameStore = Arc<Mutex<HashMap<String, GameState>>>;

// API Routes

// Get hello world
async fn hello_world() -> &'static str {
    "Hello from Catanatron backend!"
}

// Create a new game
async fn create_game(
    State(games): State<GameStore>,
    Json(config): Json<GameConfig>,
) -> Result<Json<GameState>, StatusCode> {
    log::info!("Creating game with mode: {:?}, players: {}", config.mode, config.num_players);
    
    let game_id = Uuid::new_v4().to_string();
    
    // Create the appropriate game based on mode
    let actual_game = match config.mode {
        GameMode::RandomBots => {
            Some(simulate_bot_game(config.num_players))
        },
        GameMode::HumanVsCatanatron => {
            Some(start_human_vs_catanatron("Human Player".to_string(), config.num_players - 1))
        },
        GameMode::CatanatronBots => {
            Some(simulate_bot_game(config.num_players))
        }
    };
    
    let game_state = GameState {
        id: game_id.clone(),
        status: GameStatus::Waiting,
        game: actual_game,
    };
    
    let mut games = games.lock().unwrap();
    games.insert(game_id, game_state.clone());
    
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
    State(games): State<GameStore>,
    Path(game_id): Path<String>,
) -> Result<Json<GameState>, StatusCode> {
    log::info!("Getting game with ID: {}", game_id);
    
    let games = games.lock().unwrap();
    
    if let Some(game) = games.get(&game_id) {
        Ok(Json(game.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    // Initialize logger
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    
    // Create shared state
    let games: GameStore = Arc::new(Mutex::new(HashMap::new()));
    
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any)
        .allow_origin("http://localhost:4200".parse::<HeaderValue>().unwrap());
    
    // Create router with routes
    let app = Router::new()
        .route("/", get(hello_world))
        .route("/games", post(create_game))
        .route("/games/{game_id}", get(get_game))
        .with_state(games)
        .layer(cors);

    log::info!("Starting Catanatron backend server");
    
    Ok(app.into())
}

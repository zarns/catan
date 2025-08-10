use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::actions::{resource_to_u8, GameEvent, GameId, PlayerAction};
use crate::errors::{CatanError, CatanResult, GameError, PlayerError};
use crate::game::{Game, GameState};
use crate::player_system::{Player, PlayerFactory};

/// Core application service for game management
/// This is the main orchestration layer that coordinates between domain and infrastructure
///
/// TODO: Known issues and improvements needed:
/// 1. Human player strategy needs external action injection mechanism  
/// 2. Bot decision-making could be enhanced with difficulty levels
/// 3. Game cleanup and resource management needs improvement
/// 4. Error recovery for network disconnections not implemented
/// 5. Action validation could be more granular
/// 6. Game state synchronization between players needs optimization
#[derive(Clone)]
pub struct GameService {
    games: Arc<RwLock<HashMap<GameId, Arc<RwLock<Game>>>>>,
    players: Arc<RwLock<HashMap<GameId, Vec<Player>>>>,
    bot_modes: Arc<RwLock<HashMap<GameId, String>>>,
}

impl GameService {
    pub fn new() -> Self {
        Self {
            games: Arc::new(RwLock::new(HashMap::new())),
            players: Arc::new(RwLock::new(HashMap::new())),
            bot_modes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new game with the specified configuration
    pub async fn create_game(&self, num_players: u8, bot_type: &str) -> CatanResult<GameId> {
        log::info!("🏭 DEBUG GameService::create_game:");
        log::info!("  - num_players: {num_players}");
        log::info!("  - bot_type: '{bot_type}'");

        let game_id = Uuid::new_v4().to_string();
        log::info!("  - Generated game_id: {game_id}");

        // Create the game instance using the appropriate function
        let game = match bot_type {
            "human_alphabeta" | "human_random" => {
                log::info!("  - Creating human vs bots game");
                // For human vs bots mode, use the specialized function
                crate::game::start_human_vs_catanatron("Human".to_string(), num_players - 1)
            }
            _ => {
                log::info!("  - Creating all-bot game");
                // For other modes, use the regular Game::new
                let player_names: Vec<String> =
                    (0..num_players).map(|i| format!("Bot {}", i + 1)).collect();
                let mut game = Game::new(game_id.clone(), player_names);

                // For all-bot games, all players are bots
                if bot_type == "random" {
                    game.bot_colors = game.players.iter().map(|p| p.color.clone()).collect();
                }

                game
            }
        };

        // Override the game ID with our generated one
        let mut game = game;
        game.id = game_id.clone();

        log::info!("  - Game created with {} players", game.players.len());
        log::info!("  - Current color: {:?}", game.current_color);
        log::info!("  - Current prompt: {:?}", game.current_prompt);
        log::info!(
            "  - Available actions: {}",
            game.current_playable_actions.len()
        );

        // Create player instances using the simple player system
        let mut players = Vec::new();
        let colors = ["red", "blue", "white", "orange"];

        for (i, player) in game.players.iter().enumerate() {
            let player_id = format!("player_{i}");
            let color = colors[i % colors.len()].to_string();

            let player_obj = if (bot_type == "human_alphabeta" || bot_type == "human_random")
                && i == 0
            {
                // First player is human in human vs bots mode
                log::info!("  - Creating human player: {}", player.name);
                PlayerFactory::create_human(player_id, player.name.clone(), color)
            } else {
                // All other players are bots
                log::info!("  - Creating bot player: {}", player.name);
                match bot_type {
                    // For human_alphabeta, the real AlphaBeta is invoked directly in process_bot_turn.
                    // Here we can use a placeholder (random strategy) for player metadata.
                    "human_alphabeta" => {
                        PlayerFactory::create_random_bot(player_id, player.name.clone(), color)
                    }
                    "alphabeta" => {
                        // Watch Catanatron: all bots use AlphaBeta (decided in process_bot_turn)
                        PlayerFactory::create_random_bot(player_id, player.name.clone(), color)
                    }
                    "human_random" => {
                        PlayerFactory::create_random_bot(player_id, player.name.clone(), color)
                    }
                    _ => PlayerFactory::create_random_bot(player_id, player.name.clone(), color),
                }
            };

            players.push(player_obj);
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

        // Store bot mode for this game for later decision routing
        {
            let mut modes = self.bot_modes.write().await;
            modes.insert(game_id.clone(), bot_type.to_string());
        }

        log::info!("🏭 END GameService::create_game debug\n");

        Ok(game_id)
    }

    /// Get a game by ID
    pub async fn get_game(&self, game_id: &str) -> CatanResult<Game> {
        log::info!("📖 DEBUG GameService::get_game for game_id: {game_id}");

        let games = self.games.read().await;

        if let Some(game_arc) = games.get(game_id) {
            let mut game = game_arc.write().await;
            log::debug!("  - Found game, updating metadata...");
            // Update metadata before returning
            game.update_metadata_from_state();

            log::debug!("  - Final game state:");
            log::debug!("    - Current color: {:?}", game.current_color);
            log::debug!("    - Current prompt: {:?}", game.current_prompt);
            log::debug!(
                "    - Available actions: {}",
                game.current_playable_actions.len()
            );
            log::debug!("    - Bot colors: {:?}", game.bot_colors);
            log::debug!("📖 END GameService::get_game debug\n");

            Ok(game.clone())
        } else {
            log::warn!("❌ Game not found: {game_id}");
            Err(CatanError::Game(GameError::GameNotFound {
                game_id: game_id.to_string(),
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
        action: PlayerAction,
    ) -> CatanResult<Vec<GameEvent>> {
        let games = self.games.read().await;

        let game_arc = games.get(game_id).ok_or_else(|| {
            CatanError::Game(GameError::GameNotFound {
                game_id: game_id.to_string(),
            })
        })?;

        let mut game = game_arc.write().await;

        // Find the player's color index for proper action conversion
        let player_color_index = game
            .players
            .iter()
            .position(|p| p.id == player_id)
            .ok_or_else(|| {
                CatanError::Player(PlayerError::PlayerNotInGame {
                    player_id: player_id.to_string(),
                    game_id: game_id.to_string(),
                })
            })? as u8;

        // Convert PlayerAction to the internal Action type with correct color
        let internal_action =
            Self::convert_player_action_to_internal(action.clone(), player_color_index);

        // Process the action
        match game.process_action(player_id, internal_action) {
            Ok(()) => {
                // Generate events based on the action
                let events = vec![GameEvent::ActionExecuted {
                    game_id: game_id.to_string(),
                    player_id: player_id.to_string(),
                    action,
                    success: true,
                    message: "Action processed successfully".to_string(),
                }];

                Ok(events)
            }
            Err(error) => {
                let events = vec![GameEvent::ActionExecuted {
                    game_id: game_id.to_string(),
                    player_id: player_id.to_string(),
                    action,
                    success: false,
                    message: error,
                }];

                Ok(events)
            }
        }
    }

    /// Convert PlayerAction to internal Action with correct color
    fn convert_player_action_to_internal(action: PlayerAction, color: u8) -> crate::enums::Action {
        use crate::enums::Action as EnumAction;

        match action {
            PlayerAction::Roll => EnumAction::Roll {
                color,
                dice_opt: None,
            },
            PlayerAction::BuildRoad { edge_id } => EnumAction::BuildRoad { color, edge_id },
            PlayerAction::BuildSettlement { node_id } => {
                EnumAction::BuildSettlement { color, node_id }
            }
            PlayerAction::BuildCity { node_id } => EnumAction::BuildCity { color, node_id },
            PlayerAction::BuyDevelopmentCard => EnumAction::BuyDevelopmentCard { color },
            PlayerAction::PlayKnight => EnumAction::PlayKnight { color },
            PlayerAction::PlayYearOfPlenty { resources } => EnumAction::PlayYearOfPlenty {
                color,
                resources: (resource_to_u8(resources.0), resources.1.map(resource_to_u8)),
            },
            PlayerAction::PlayMonopoly { resource } => EnumAction::PlayMonopoly {
                color,
                resource: resource_to_u8(resource),
            },
            PlayerAction::PlayRoadBuilding => EnumAction::PlayRoadBuilding { color },
            PlayerAction::MaritimeTrade { give, take, ratio } => EnumAction::MaritimeTrade {
                color,
                give: resource_to_u8(give),
                take: resource_to_u8(take),
                ratio,
            },
            PlayerAction::EndTurn => EnumAction::EndTurn { color },
            PlayerAction::MoveRobber { coordinate, victim } => {
                let victim_opt = victim.and_then(|v| {
                    // Extract color index from "player_X" format
                    v.strip_prefix("player_").and_then(|s| s.parse::<u8>().ok())
                });
                EnumAction::MoveRobber {
                    color,
                    coordinate,
                    victim_opt,
                }
            }
            PlayerAction::Discard { .. } => EnumAction::Discard { color },
            _ => EnumAction::EndTurn { color }, // Default for unhandled actions
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
                game_id: game_id.to_string(),
            }))
        }
    }

    /// Process bot turn if it's a bot's turn
    ///
    /// TODO: Current limitations and improvements needed:
    /// 1. Bot decision-making is purely random - needs strategy implementation
    /// 2. Error handling could be more granular (distinguish different failure types)
    /// 3. Action validation happens after bot decides - should validate available actions first
    /// 4. No timeout mechanism for bot decisions (could hang indefinitely)
    /// 5. Bot difficulty levels not supported
    /// 6. No bot state persistence between turns
    /// 7. Game state consistency checks missing before bot processing
    pub async fn process_bot_turn(&self, game_id: &str) -> CatanResult<Option<Vec<GameEvent>>> {
        // Get the game to check current player
        let game = self.get_game(game_id).await?;
        let players = self.get_players(game_id).await?;

        // Validate game state before processing
        if !matches!(game.game_state, GameState::Active | GameState::Setup) {
            return Ok(None);
        }

        if game.current_player_index >= players.len() {
            log::warn!(
                "Invalid current_player_index {} >= {} for game {}",
                game.current_player_index,
                players.len(),
                game_id
            );
            return Ok(None);
        }

        let current_player = &players[game.current_player_index];

        // Check if current player is a bot
        if !current_player.info.is_bot {
            return Ok(None);
        }

        // Determine bot mode for this game
        let bot_mode = {
            let modes = self.bot_modes.read().await;
            modes
                .get(game_id)
                .cloned()
                .unwrap_or_else(|| "random".to_string())
        };

        // Get available actions with proper validation and error handling
        let available_actions: Vec<PlayerAction> = if let Some(ref state) = game.state {
            let state_actions = state.generate_playable_actions();
            if state_actions.is_empty() {
                vec![PlayerAction::EndTurn]
            } else {
                state_actions.iter().map(|a| (*a).into()).collect()
            }
        } else {
            log::error!(
                "No game state available for bot {} in game {}",
                current_player.info.name,
                game_id
            );
            vec![PlayerAction::EndTurn]
        };

        // Add specific logging for initial build phase state tracking
        if let Some(ref state) = game.state {
            let action_prompt = state.get_action_prompt();
            let is_initial_phase = state.is_initial_build_phase();
            let current_color = state.get_current_color();

            // Get CURRENT PLAYER's building counts, not total
            let player_settlements = state.get_settlements(current_color).len();
            let player_roads = state.get_roads_by_color()[current_color as usize];
            let player_vp = state.get_actual_victory_points(current_color);
            let cities = state.get_cities(current_color).len();

            log::info!("🎮 Bot {} turn | Phase: {:?} | Initial: {} | VP: {} | Settlements: {} | Cities: {} | Roads: {} | Actions: {}", 
                      current_player.info.name, action_prompt, is_initial_phase, player_vp, player_settlements, cities, player_roads, available_actions.len());

            // Log ALL PLAYERS' building counts for debugging UI vs backend mismatch
            log::info!("🏗️  Building Summary:");
            let num_players = if let Some(ref state) = game.state {
                state.get_num_players()
            } else {
                game.players.len() as u8
            };

            for player_color in 0..num_players {
                let settlements = state.get_settlements(player_color).len();
                let cities = state.get_cities(player_color).len();
                let vp = state.get_actual_victory_points(player_color);
                let player_name = format!("Player {}", player_color + 1);
                log::info!(
                    "   {} (color {}): {} settlements, {} cities, {} VP",
                    player_name,
                    player_color,
                    settlements,
                    cities,
                    vp
                );
            }
        }

        // Let the bot decide what action to take with timeout protection
        let decision_result = if bot_mode == "human_alphabeta" || bot_mode == "alphabeta" {
            // Use backend AlphaBetaPlayer on the internal state for bots
            use crate::enums::Action as EnumAction;
            use crate::players::{minimax::AlphaBetaPlayer, BotPlayer as _};

            if let Some(ref state) = game.state {
                let state_actions: Vec<EnumAction> = state.generate_playable_actions();
                let ab = AlphaBetaPlayer::new(
                    "alphabeta_bot".to_string(),
                    "AlphaBeta Bot".to_string(),
                    "gray".to_string(),
                );
                // Run synchronously within timeout wrapper
                let decided_internal = ab.decide(state, &state_actions);
                let decided_player_action: PlayerAction = decided_internal.into();
                Ok(Ok(decided_player_action))
            } else {
                Ok(Ok(PlayerAction::EndTurn))
            }
        } else {
            tokio::time::timeout(
                tokio::time::Duration::from_secs(5),
                current_player.decide_action(&game.game_state, &available_actions),
            )
            .await
        };

        match decision_result {
            Ok(Ok(action)) => {
                log::info!("🤖 {} action: {:?}", current_player.info.name, action);

                // Validate the action is actually in available actions
                if !available_actions.contains(&action) {
                    log::warn!(
                        "Bot {} chose invalid action {:?}. Using EndTurn instead.",
                        current_player.info.name,
                        action
                    );
                    self.process_action(game_id, &current_player.info.id, PlayerAction::EndTurn)
                        .await
                        .map(Some)
                } else {
                    // Process the bot's action
                    self.process_action(game_id, &current_player.info.id, action)
                        .await
                        .map(Some)
                }
            }
            Ok(Err(e)) => {
                log::warn!(
                    "Bot {} decision failed: {:?}, ending turn",
                    current_player.info.name,
                    e
                );
                self.process_action(game_id, &current_player.info.id, PlayerAction::EndTurn)
                    .await
                    .map(Some)
            }
            Err(_timeout) => {
                log::error!(
                    "Bot {} decision timeout, ending turn",
                    current_player.info.name
                );
                self.process_action(game_id, &current_player.info.id, PlayerAction::EndTurn)
                    .await
                    .map(Some)
            }
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

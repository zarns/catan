use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use rand::prelude::*;

use super::types::*;
use super::board::BoardGenerator;
use super::ai::RandomAIPlayer;

#[derive(Clone)]
pub struct Game {
    pub id: String,
    pub state: GameState,
    pub max_players: usize,
    pub ai_players: Vec<RandomAIPlayer>,
}

pub struct GameManager {
    pub games: HashMap<String, Game>,
}

impl Game {
    pub fn new(id: String, max_players: usize) -> Self {
        Self {
            id,
            state: GameState {
                players: HashMap::new(),
                board: BoardGenerator::generate_board(),
                current_turn: String::new(),
                dice_value: None,
                phase: GamePhase::Lobby,
                robber_position: Position { x: 0, y: 0 },
            },
            max_players,
            ai_players: Vec::new(),
        }
    }
}

impl Default for GameManager {
    fn default() -> Self {
        Self {
            games: HashMap::new(),
        }
    }
}

impl GameManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_game(&mut self, max_players: usize) -> String {
        let game_id = Uuid::new_v4().to_string();
        let game = Game::new(game_id.clone(), max_players);
        self.games.insert(game_id.clone(), game);
        game_id
    }

    pub fn create_game_with_ai(&mut self, human_player_id: String, ai_count: usize) -> Result<String, String> {
        if ai_count == 0 || ai_count > 3 {
            return Err("AI count must be between 1 and 3".to_string());
        }

        let game_id = Uuid::new_v4().to_string();
        let mut game = Game::new(game_id.clone(), ai_count + 1);

        // Add human player
        let human_player = Player {
            id: human_player_id.clone(),
            name: format!("Player {}", human_player_id[..6].to_string()),
            resources: HashMap::new(),
            development_cards: Vec::new(),
            roads: HashSet::new(),
            settlements: HashSet::new(),
            cities: HashSet::new(),
            knights_played: 0,
        };
        game.state.players.insert(human_player_id.clone(), human_player);

        // Add AI players
        for _ in 0..ai_count {
            let ai_id = Uuid::new_v4().to_string();
            let ai = RandomAIPlayer::new(ai_id.clone());
            
            let player = Player {
                id: ai.id.clone(),
                name: ai.name.clone(),
                resources: HashMap::new(),
                development_cards: Vec::new(),
                roads: HashSet::new(),
                settlements: HashSet::new(),
                cities: HashSet::new(),
                knights_played: 0,
            };

            game.state.players.insert(ai.id.clone(), player);
            game.ai_players.push(ai);
        }

        // Set the human player as the first player
        game.state.current_turn = human_player_id;
        game.state.phase = GamePhase::Setup(SetupPhase::PlacingFirstSettlement);

        self.games.insert(game_id.clone(), game);
        Ok(game_id)
    }

    pub fn get_game_state(&self, game_id: &str) -> Option<&GameState> {
        self.games.get(game_id).map(|game| &game.state)
    }

    fn count_settlements(&self, game: &Game) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        
        // Initialize counts for all players
        for player_id in game.state.players.keys() {
            counts.insert(player_id.clone(), 0);
        }

        // Count settlements
        for (player_id, building_type) in game.state.board.settlements.values() {
            if *building_type == BuildingType::Settlement {
                *counts.entry(player_id.clone()).or_insert(0) += 1;
            }
        }

        counts
    }

    pub fn place_settlement(
        &mut self,
        game_id: &str,
        player_id: &str,
        position: Position,
    ) -> Result<(), String> {
        let game = self.games.get_mut(game_id)
            .ok_or_else(|| "Game not found".to_string())?;

        if game.state.current_turn != player_id {
            return Err("Not your turn".to_string());
        }

        // Validate based on game phase
        match game.state.phase {
            GamePhase::Setup(SetupPhase::PlacingFirstSettlement) |
            GamePhase::Setup(SetupPhase::PlacingSecondSettlement) => {
                // Validate position
                let valid_positions = BoardGenerator::get_valid_settlement_positions(&game.state.board, false);
                if !valid_positions.contains(&position) {
                    return Err("Invalid settlement position".to_string());
                }

                // Place settlement
                game.state.board.settlements.insert(position, (player_id.to_string(), BuildingType::Settlement));

                // Update phase
                game.state.phase = match game.state.phase {
                    GamePhase::Setup(SetupPhase::PlacingFirstSettlement) => 
                        GamePhase::Setup(SetupPhase::PlacingFirstRoad),
                    GamePhase::Setup(SetupPhase::PlacingSecondSettlement) => 
                        GamePhase::Setup(SetupPhase::PlacingSecondRoad),
                    _ => unreachable!(),
                };

                Ok(())
            },
            _ => Err("Cannot place settlement in current phase".to_string()),
        }
    }

    pub fn place_road(
        &mut self,
        game_id: &str,
        player_id: &str,
        start: Position,
        end: Position,
    ) -> Result<(), String> {
        // First get all the information we need with an immutable borrow
        let (next_phase, next_turn) = {
            let game = self.games.get(game_id)
                .ok_or_else(|| "Game not found".to_string())?;

            if game.state.current_turn != player_id {
                return Err("Not your turn".to_string());
            }

            let counts = self.count_settlements(game);
            
            match game.state.phase {
                GamePhase::Setup(SetupPhase::PlacingFirstRoad) => {
                    let all_placed_first = counts.iter().all(|(_, &count)| count >= 1);
                    let next_phase = if all_placed_first {
                        GamePhase::Setup(SetupPhase::PlacingSecondSettlement)
                    } else {
                        GamePhase::Setup(SetupPhase::PlacingFirstSettlement)
                    };

                    let player_ids: Vec<String> = game.state.players.keys().cloned().collect();
                    let current_index = player_ids.iter()
                        .position(|id| *id == game.state.current_turn)
                        .ok_or_else(|| "Current player not found".to_string())?;
                    let next_index = (current_index + 1) % player_ids.len();
                    let next_turn = player_ids[next_index].clone();

                    (next_phase, next_turn)
                },
                GamePhase::Setup(SetupPhase::PlacingSecondRoad) => {
                    let all_placed_second = counts.iter().all(|(_, &count)| count >= 2);
                    let next_phase = if all_placed_second {
                        GamePhase::MainGame(MainGamePhase::Rolling)
                    } else {
                        GamePhase::Setup(SetupPhase::PlacingSecondSettlement)
                    };

                    let player_ids: Vec<String> = game.state.players.keys().cloned().collect();
                    let current_index = player_ids.iter()
                        .position(|id| *id == game.state.current_turn)
                        .ok_or_else(|| "Current player not found".to_string())?;
                    let next_index = (current_index + 1) % player_ids.len();
                    let next_turn = player_ids[next_index].clone();

                    (next_phase, next_turn)
                },
                _ => return Err("Cannot place road in current phase".to_string()),
            }
        };

        // Now get mutable access to make changes
        let game = self.games.get_mut(game_id)
            .ok_or_else(|| "Game not found".to_string())?;

        // Place road
        game.state.board.roads.insert((start, end), player_id.to_string());
        
        // Update game state
        game.state.phase = next_phase;
        game.state.current_turn = next_turn;

        Ok(())
    }

    pub fn roll_dice(&mut self, game_id: &str, player_id: &str) -> Result<(u8, u8), String> {
        let game = self.games.get_mut(game_id)
            .ok_or_else(|| "Game not found".to_string())?;

        match game.state.phase {
            GamePhase::MainGame(MainGamePhase::Rolling) => {
                if game.state.current_turn != player_id {
                    return Err("Not your turn".to_string());
                }

                let mut rng = rand::thread_rng();
                let dice1 = rng.gen_range(1..=6);
                let dice2 = rng.gen_range(1..=6);

                game.state.dice_value = Some((dice1, dice2));
                
                // Update game phase based on roll
                if dice1 + dice2 == 7 {
                    game.state.phase = GamePhase::MainGame(MainGamePhase::MovingRobber);
                } else {
                    game.state.phase = GamePhase::MainGame(MainGamePhase::Building);
                    // TODO: Distribute resources based on roll
                }

                Ok((dice1, dice2))
            },
            _ => Err("Can only roll dice in Rolling phase".to_string()),
        }
    }

    pub fn process_ai_turns(&mut self, game_id: &str) -> Result<Vec<GameResponse>, String> {
        let mut responses = Vec::new();
        loop {
            // Check if it's an AI's turn
            let ai_move = {
                let game = self.games.get(game_id)
                    .ok_or_else(|| "Game not found".to_string())?;

                if game.state.phase == GamePhase::Ended {
                    break;
                }

                // Check if current player is AI
                let current_ai = game.ai_players.iter()
                    .find(|ai| ai.id == game.state.current_turn);

                match current_ai {
                    Some(ai) => ai.get_move(&game.state),
                    None => break, // Human player's turn
                }
            };

            // Process AI move if we got one
            if let Some(cmd) = ai_move {
                let current_ai_id = {
                    let game = self.games.get(game_id)
                        .ok_or_else(|| "Game not found".to_string())?;
                    game.state.current_turn.clone()
                };

                // Process the AI's move
                match cmd {
                    GameCommand::BuildSettlement { position } => {
                        self.place_settlement(game_id, &current_ai_id, position)?;
                    },
                    GameCommand::BuildRoad { start, end } => {
                        self.place_road(game_id, &current_ai_id, start, end)?;
                    },
                    GameCommand::RollDice => {
                        let roll = self.roll_dice(game_id, &current_ai_id)?;
                        responses.push(GameResponse::DiceRolled {
                            player_id: current_ai_id,
                            values: roll,
                        });
                    },
                    GameCommand::EndTurn => {
                        // TODO: Implement end turn
                    },
                    _ => {}
                }

                // Add state update to responses
                if let Some(state) = self.get_game_state(game_id) {
                    responses.push(GameResponse::GameStateUpdate {
                        game_state: state.clone(),
                    });
                }
            }
        }

        Ok(responses)
    }
}

// Create a thread-safe wrapper for GameManager
pub type SharedGameManager = Arc<Mutex<GameManager>>;

pub fn create_shared_game_manager() -> SharedGameManager {
    Arc::new(Mutex::new(GameManager::new()))
}
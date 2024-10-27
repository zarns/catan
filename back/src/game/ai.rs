use rand::prelude::*;
use crate::game::types::*;
use crate::game::board::BoardGenerator;

#[derive(Clone)]
pub struct RandomAIPlayer {
    pub id: String,
    pub name: String,
}

impl RandomAIPlayer {
    pub fn new(id: String) -> Self {
        Self {
            id: id.clone(),
            name: format!("AI-{}", &id[..6]),
        }
    }

    pub fn get_move(&self, game_state: &GameState) -> Option<GameCommand> {
        if self.id != game_state.current_turn {
            return None;
        }

        match game_state.phase {
            GamePhase::Setup(SetupPhase::PlacingFirstSettlement) |
            GamePhase::Setup(SetupPhase::PlacingSecondSettlement) => {
                let valid_positions = BoardGenerator::get_valid_settlement_positions(&game_state.board, false);
                let position = valid_positions.iter().choose(&mut thread_rng())?;
                
                Some(GameCommand::BuildSettlement {
                    position: (*position).clone(),
                })
            },
            
            GamePhase::Setup(SetupPhase::PlacingFirstRoad) |
            GamePhase::Setup(SetupPhase::PlacingSecondRoad) => {
                // Find the last settlement placed by this AI
                let my_settlements: Vec<_> = game_state.board.settlements.iter()
                    .filter(|(_, (player_id, _))| player_id == &self.id)
                    .map(|(pos, _)| pos)
                    .collect();
                
                if let Some(settlement_pos) = my_settlements.last() {
                    // Get valid adjacent positions for road placement
                    let adjacent_positions = BoardGenerator::get_adjacent_vertices(settlement_pos);
                    if let Some(end_pos) = adjacent_positions.iter().choose(&mut thread_rng()) {
                        Some(GameCommand::BuildRoad {
                            start: (*settlement_pos).clone(),
                            end: (*end_pos).clone(),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            },

            GamePhase::MainGame(MainGamePhase::Rolling) => {
                Some(GameCommand::RollDice)
            },

            GamePhase::MainGame(MainGamePhase::Building) => {
                // For now, just end turn
                Some(GameCommand::EndTurn)
            },

            _ => None,
        }
    }
}
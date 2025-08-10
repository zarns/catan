use log::LevelFilter;
use std::f64;

use crate::enums::Action;
use crate::state::State;

const DEFAULT_DEPTH: i32 = 6;
const MAX_ORDERED_ACTIONS: usize = 12; // beam size for ordering

use super::BotPlayer;

/// Alpha-Beta Minimax Player
/// Uses the minimax algorithm to choose actions
pub struct AlphaBetaPlayer {
    pub id: String,
    pub name: String,
    pub color: String,
    depth: i32,
}

impl AlphaBetaPlayer {
    pub fn new(id: String, name: String, color: String) -> Self {
        AlphaBetaPlayer {
            id,
            name,
            color,
            depth: DEFAULT_DEPTH,
        }
    }

    pub fn with_depth(id: String, name: String, color: String, depth: i32) -> Self {
        AlphaBetaPlayer {
            id,
            name,
            color,
            depth,
        }
    }

    /// Evaluate the game state from the perspective of the given player
    fn evaluate_state(&self, state: &State, player_color: u8) -> f64 {
        // Simple evaluation based on victory points and buildings
        let vp = state.get_actual_victory_points(player_color) as f64;
        let settlements = state.get_settlements(player_color).len() as f64;
        let cities = state.get_cities(player_color).len() as f64;
        let resources = state.get_player_hand(player_color).iter().sum::<u8>() as f64;
        let dev_cards = state.get_player_devhand(player_color).iter().sum::<u8>() as f64;

        // Weighted evaluation
        vp * 100.0 + settlements * 10.0 + cities * 20.0 + resources * 1.0 + dev_cards * 3.0
    }

    /// Get the relative evaluation (my score - average opponent score)
    fn evaluate_relative(&self, state: &State, my_color: u8) -> f64 {
        let my_score = self.evaluate_state(state, my_color);

        let mut opponent_scores = 0.0;
        let num_players = 4; // Assume 4 players for now

        for color in 0..num_players {
            if color != my_color {
                opponent_scores += self.evaluate_state(state, color);
            }
        }

        let avg_opponent_score = if num_players > 1 {
            opponent_scores / (num_players - 1) as f64
        } else {
            0.0
        };

        my_score - avg_opponent_score
    }

    /// Heuristic move ordering: prioritize building and high-impact actions
    fn order_actions(&self, state: &State, actions: &[Action]) -> Vec<Action> {
        let mut scored: Vec<(i32, Action)> = actions
            .iter()
            .copied()
            .map(|a| (self.score_action(state, a), a))
            .collect();
        scored.sort_by(|(sa, _), (sb, _)| sb.cmp(sa));
        scored
            .into_iter()
            .take(MAX_ORDERED_ACTIONS)
            .map(|(_, a)| a)
            .collect()
    }

    fn score_action(&self, _state: &State, action: Action) -> i32 {
        use crate::enums::Action as A;
        match action {
            A::BuildCity { .. } => 1000,
            A::BuildSettlement { .. } => 800,
            A::BuildRoad { .. } => 300,
            A::BuyDevelopmentCard { .. } => 250,
            A::PlayKnight { .. } => 220,
            A::PlayRoadBuilding { .. } => 220,
            A::PlayYearOfPlenty { .. } => 200,
            A::PlayMonopoly { .. } => 180,
            A::MaritimeTrade { .. } => 120,
            A::OfferTrade { .. } => 60,
            A::AcceptTrade { .. } => 60,
            A::ConfirmTrade { .. } => 60,
            A::RejectTrade { .. } => 40,
            A::CancelTrade { .. } => 30,
            A::MoveRobber { .. } => 20,
            A::Roll { .. } => 10,
            A::Discard { .. } => 5,
            A::EndTurn { .. } => 0,
        }
    }

    /// Expectiminimax handling for rolling dice: average over dice outcomes
    fn roll_expectation(
        &self,
        state: &State,
        depth: i32,
        alpha: f64,
        beta: f64,
        my_color: u8,
        color_to_roll: u8,
    ) -> f64 {
        // Probabilities for sums 2..12 over two fair dice
        const SUM_PROBS: [(u8, f64, (u8, u8)); 11] = [
            (2, 1.0 / 36.0, (1, 1)),
            (3, 2.0 / 36.0, (1, 2)),
            (4, 3.0 / 36.0, (1, 3)),
            (5, 4.0 / 36.0, (2, 3)),
            (6, 5.0 / 36.0, (3, 3)),
            (7, 6.0 / 36.0, (1, 6)),
            (8, 5.0 / 36.0, (2, 6)),
            (9, 4.0 / 36.0, (3, 6)),
            (10, 3.0 / 36.0, (4, 6)),
            (11, 2.0 / 36.0, (5, 6)),
            (12, 1.0 / 36.0, (6, 6)),
        ];

        let mut expected = 0.0;
        for &(_sum, prob, pair) in &SUM_PROBS {
            let mut next_state = state.clone();
            next_state.apply_action(Action::Roll {
                color: color_to_roll,
                dice_opt: Some(pair),
            });
            // After rolling, it is still the same player's turn; do not negate here
            let v = self.minimax(&next_state, depth - 1, alpha, beta, my_color);
            expected += prob * v;
        }
        expected
    }

    /// Alpha-Beta minimax with simple beam-ordered moves
    fn minimax(&self, state: &State, depth: i32, mut alpha: f64, beta: f64, my_color: u8) -> f64 {
        // Base case: terminal depth or game over
        if depth == 0 || state.winner().is_some() {
            return self.evaluate_relative(state, my_color);
        }

        let actions = state.generate_playable_actions();
        if actions.is_empty() {
            return self.evaluate_relative(state, my_color);
        }

        // Order actions to improve pruning
        let ordered_actions = self.order_actions(state, &actions);

        let mut best_value = f64::NEG_INFINITY;
        for action in ordered_actions {
            let value = match action {
                Action::Roll {
                    color,
                    dice_opt: None,
                } => {
                    // Handle chance explicitly via expectation over dice outcomes
                    self.roll_expectation(state, depth, alpha, beta, my_color, color)
                }
                _ => {
                    let mut new_state = state.clone();
                    new_state.apply_action(action);
                    -self.minimax(&new_state, depth - 1, -beta, -alpha, my_color)
                }
            };
            if value > best_value {
                best_value = value;
            }
            if value > alpha {
                alpha = value;
            }
            if alpha >= beta {
                break; // beta cut-off
            }
        }

        best_value
    }
}

impl BotPlayer for AlphaBetaPlayer {
    fn decide(&self, state: &State, playable_actions: &[Action]) -> Action {
        if playable_actions.len() == 1 {
            return playable_actions[0];
        }

        let my_color = state.get_current_color();

        // Suppress all logs globally during search
        let prev_level = log::max_level();
        log::set_max_level(LevelFilter::Off);

        let mut best_action = playable_actions[0];
        let mut best_value = f64::NEG_INFINITY;

        // Order root actions
        let ordered = self.order_actions(state, playable_actions);
        for action in ordered {
            let mut new_state = state.clone();
            new_state.apply_action(action);
            let value = self.minimax(
                &new_state,
                self.depth - 1,
                f64::NEG_INFINITY,
                f64::INFINITY,
                my_color,
            );
            if value > best_value {
                best_value = value;
                best_action = action;
            }
        }

        // Restore previous logging level after search
        log::set_max_level(prev_level);
        // Do not log timing by default to keep simulations quiet

        best_action
    }
}

impl Default for AlphaBetaPlayer {
    fn default() -> Self {
        Self::new(
            "default".to_string(),
            "AlphaBeta Player".to_string(),
            "red".to_string(),
        )
    }
}

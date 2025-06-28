use std::time::Instant;
use std::f64;

use crate::enums::Action;
use crate::state::State;

const DEFAULT_DEPTH: i32 = 2;

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

    /// Simple minimax without alpha-beta pruning for now
    fn minimax(&self, state: &State, depth: i32, my_color: u8) -> f64 {
        // Base case: terminal depth or game over
        if depth == 0 || state.winner().is_some() {
            return self.evaluate_relative(state, my_color);
        }

        let actions = state.generate_playable_actions();
        if actions.is_empty() {
            return self.evaluate_relative(state, my_color);
        }

        // For simplicity, just evaluate immediate outcomes
        let mut total_value = 0.0;
        let actions_len = actions.len();
        for action in actions {
            let mut new_state = state.clone();
            new_state.apply_action(action);
            total_value += self.minimax(&new_state, depth - 1, my_color);
        }
        
        total_value / actions_len as f64
    }
}

impl BotPlayer for AlphaBetaPlayer {
    fn decide(&self, state: &State, playable_actions: &[Action]) -> Action {
        if playable_actions.len() == 1 {
            return playable_actions[0].clone();
        }

        let start = Instant::now();
        let my_color = state.get_current_color();
        
        let mut best_action = playable_actions[0].clone();
        let mut best_value = f64::NEG_INFINITY;

        for action in playable_actions {
            let mut new_state = state.clone();
            new_state.apply_action(action.clone());
            
            let value = self.minimax(&new_state, self.depth - 1, my_color);
            
            if value > best_value {
                best_value = value;
                best_action = action.clone();
            }
        }

        let duration = start.elapsed();
        println!(
            "AlphaBeta (depth {}) took {:?} to evaluate {} actions (best value: {:.2})",
            self.depth,
            duration,
            playable_actions.len(),
            best_value
        );

        best_action
    }
}

impl Default for AlphaBetaPlayer {
    fn default() -> Self {
        Self::new("default".to_string(), "AlphaBeta Player".to_string(), "red".to_string())
    }
}

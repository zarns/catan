use std::collections::HashMap;
use std::time::Instant;
use rand::seq::SliceRandom;
use rand::thread_rng;

use super::Player;
use crate::enums::Action;
use crate::state::State;

const SIMULATIONS_PER_ACTION: usize = 3;

/// Greedy Monte Carlo Player
/// Evaluates each action by running random playouts and choosing the one with the highest win rate
pub struct GreedyPlayer {
    num_simulations_per_action: usize,
}

impl GreedyPlayer {
    pub fn new() -> Self {
        GreedyPlayer {
            num_simulations_per_action: SIMULATIONS_PER_ACTION,
        }
    }

    pub fn with_simulations(num_simulations_per_action: usize) -> Self {
        GreedyPlayer {
            num_simulations_per_action,
        }
    }

    /// Run a random playout from the given state
    fn playout(mut state: State) -> Option<u8> {
        let mut rng = thread_rng();

        // Limit the number of moves to prevent infinite games
        for _ in 0..1000 {
            if let Some(winner) = state.winner() {
                return Some(winner);
            }

            let actions = state.generate_playable_actions();
            if actions.is_empty() {
                break;
            }

            // Choose a random action
            let action = actions.choose(&mut rng).unwrap().clone();
            state.apply_action(action);
        }

        state.winner()
    }
}

impl Player for GreedyPlayer {
    fn decide(&self, state: &State, playable_actions: &[Action]) -> Action {
        if playable_actions.len() == 1 {
            return playable_actions[0].clone();
        }

        let start = Instant::now();
        let my_color = state.get_current_color();

        // Track wins for each action
        let mut action_wins: HashMap<Action, usize> = HashMap::new();
        let mut action_plays: HashMap<Action, usize> = HashMap::new();

        // For each action, run several playouts
        for action in playable_actions {
            for _ in 0..self.num_simulations_per_action {
                // Create a new state with the action applied
                let mut state_copy = state.clone();
                state_copy.apply_action(action.clone());

                // Run a random playout from this state
                if let Some(winner) = Self::playout(state_copy) {
                    // Count the win
                    let win_count = action_wins.entry(action.clone()).or_insert(0);
                    if winner == my_color {
                        *win_count += 1;
                    }
                }

                // Count the play
                let play_count = action_plays.entry(action.clone()).or_insert(0);
                *play_count += 1;
            }
        }

        // Choose the action with the highest win rate
        let mut best_action = playable_actions[0].clone();
        let mut best_win_rate = 0.0;

        for action in playable_actions {
            let wins = *action_wins.get(action).unwrap_or(&0);
            let plays = *action_plays.get(action).unwrap_or(&1); // Avoid division by zero

            let win_rate = (wins as f64) / (plays as f64);
            if win_rate > best_win_rate {
                best_win_rate = win_rate;
                best_action = action.clone();
            }
        }

        let duration = start.elapsed();
        println!(
            "Greedy took {:?} to make a decision among {} actions with win rate {:.2}%",
            duration,
            playable_actions.len(),
            best_win_rate * 100.0
        );

        best_action
    }
}

impl Default for GreedyPlayer {
    fn default() -> Self {
        Self::new()
    }
}

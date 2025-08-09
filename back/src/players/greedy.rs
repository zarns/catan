use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::prelude::*;
use std::collections::HashMap;
use std::time::Instant;

use crate::enums::Action;
use crate::state::State;

const SIMULATIONS_PER_ACTION: usize = 3;

use super::BotPlayer;

/// Greedy Monte Carlo Player
/// Evaluates each action by running random playouts and choosing the one with the highest win rate
pub struct GreedyPlayer {
    pub id: String,
    pub name: String,
    pub color: String,
    num_simulations_per_action: usize,
    use_parallel: bool,
}

impl GreedyPlayer {
    pub fn new(id: String, name: String, color: String) -> Self {
        GreedyPlayer {
            id,
            name,
            color,
            num_simulations_per_action: SIMULATIONS_PER_ACTION,
            use_parallel: true,
        }
    }

    pub fn with_simulations(
        id: String,
        name: String,
        color: String,
        num_simulations_per_action: usize,
    ) -> Self {
        GreedyPlayer {
            id,
            name,
            color,
            num_simulations_per_action,
            use_parallel: true,
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
            let action = *actions.choose(&mut rng).unwrap();
            state.apply_action(action);
        }

        state.winner()
    }

    /// Sequential (original) implementation
    fn decide_sequential(&self, state: &State, playable_actions: &[Action]) -> Action {
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
                state_copy.apply_action(*action);

                // Run a random playout from this state
                if let Some(winner) = Self::playout(state_copy) {
                    // Count the win
                    let win_count = action_wins.entry(*action).or_insert(0);
                    if winner == my_color {
                        *win_count += 1;
                    }
                }

                // Count the play
                let play_count = action_plays.entry(*action).or_insert(0);
                *play_count += 1;
            }
        }

        // Choose the action with the highest win rate
        let mut best_action = playable_actions[0];
        let mut best_win_rate = 0.0;

        for action in playable_actions {
            let wins = *action_wins.get(action).unwrap_or(&0);
            let plays = *action_plays.get(action).unwrap_or(&1); // Avoid division by zero

            let win_rate = (wins as f64) / (plays as f64);
            if win_rate > best_win_rate {
                best_win_rate = win_rate;
                best_action = *action;
            }
        }

        let duration = start.elapsed();
        println!(
            "Greedy took {:?} to make a decision among {} actions with win rate {:.2}% (sequential)",
            duration,
            playable_actions.len(),
            best_win_rate * 100.0
        );

        best_action
    }

    /// Parallel implementation
    fn decide_parallel(&self, state: &State, playable_actions: &[Action]) -> Action {
        let start = Instant::now();
        let my_color = state.get_current_color();

        // Use parallel iterator to evaluate actions
        let results: Vec<(Action, f64)> = playable_actions
            .par_iter()
            .map(|action| {
                let mut wins = 0;

                // Run simulations for this action
                for _ in 0..self.num_simulations_per_action {
                    let mut state_copy = state.clone();
                    state_copy.apply_action(*action);

                    if let Some(winner) = Self::playout(state_copy) {
                        if winner == my_color {
                            wins += 1;
                        }
                    }
                }

                let win_rate = wins as f64 / self.num_simulations_per_action as f64;
                (*action, win_rate)
            })
            .collect();

        // Find the action with the highest win rate
        let mut best_action = playable_actions[0];
        let mut best_win_rate = 0.0;

        for (action, win_rate) in results {
            if win_rate > best_win_rate {
                best_win_rate = win_rate;
                best_action = action;
            }
        }

        let duration = start.elapsed();
        println!(
            "Greedy took {:?} to make a decision among {} actions with win rate {:.2}% (parallel)",
            duration,
            playable_actions.len(),
            best_win_rate * 100.0
        );

        best_action
    }
}

impl BotPlayer for GreedyPlayer {
    fn decide(&self, state: &State, playable_actions: &[Action]) -> Action {
        if playable_actions.len() == 1 {
            return playable_actions[0];
        }

        if self.use_parallel {
            self.decide_parallel(state, playable_actions)
        } else {
            self.decide_sequential(state, playable_actions)
        }
    }
}

impl Default for GreedyPlayer {
    fn default() -> Self {
        Self::new(
            "default".to_string(),
            "Greedy Player".to_string(),
            "red".to_string(),
        )
    }
}

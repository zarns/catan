use crate::enums::Action;
use crate::players::zero::AlphaZeroPlayer;
use crate::players::BotPlayer;
use crate::state::State;
use rayon::prelude::*;

const TRAINING_MCTS_SIMULATIONS: usize = 100;
const TRAINING_EXPLORATION: f64 = 1.5;
const TRAINING_DECIDE_BUDGET_MS: u64 = 120;
const TRAINING_PLAYOUT_STEPS: usize = 80;

#[derive(Clone)]
pub struct Experience {
    pub state: State,
    pub policy_target: Vec<(Action, f32)>,
    pub value_target: f32,
}

/// Placeholder for a self-play generator. Will be fleshed out once training loop lands.
pub fn generate_self_play_game() -> Vec<Experience> {
    // Use a single AlphaZeroPlayer for the current player decisions; for other prompts
    // its decide() already short-circuits.
    let az = AlphaZeroPlayer::with_parameters_full(
        "z".to_string(),
        "AlphaZero".to_string(),
        "Z".to_string(),
        TRAINING_MCTS_SIMULATIONS,
        TRAINING_EXPLORATION,
        TRAINING_DECIDE_BUDGET_MS,
        TRAINING_PLAYOUT_STEPS,
        1.0, // root temperature for exploration during training
    );
    let mut experiences: Vec<Experience> = Vec::new();
    let mut state = State::new_base();

    let mut steps: u32 = 0;
    let max_steps: u32 = 20_000; // safety cap
    while state.winner().is_none() && steps < max_steps {
        steps += 1;
        let playable_actions = state.generate_playable_actions();
        if playable_actions.is_empty() {
            break;
        }
        // Record state before decision for policy target
        let snapshot = state.clone();
        let action = az.decide(&state, &playable_actions);
        // Capture visit-count distribution (may be empty for non-PlayTurn prompts)
        let policy = az.take_last_policy();
        if !policy.is_empty() {
            experiences.push(Experience {
                state: snapshot,
                policy_target: policy,
                value_target: 0.0,
            });
        }
        state.apply_action(action);
    }

    // Assign outcome
    if let Some(winner) = state.winner() {
        for exp in experiences.iter_mut() {
            let me = exp.state.get_current_color();
            exp.value_target = if winner == me { 1.0 } else { -1.0 };
        }
    }

    experiences
}

/// Generate N self-play games and return a flat list of experiences.
pub fn generate_self_play_games(num_games: usize) -> Vec<Experience> {
    let mut out = Vec::new();
    for _ in 0..num_games {
        out.extend(generate_self_play_game());
    }
    out
}

/// Parallel self-play generation using rayon (game-level parallelism).
pub fn generate_self_play_games_parallel(num_games: usize) -> Vec<Experience> {
    (0..num_games)
        .into_par_iter()
        .flat_map(|_| generate_self_play_game())
        .collect()
}

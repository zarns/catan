use crate::enums::Action;
use crate::players::zero::AlphaZeroPlayer;
use crate::state::State;
use crate::players::BotPlayer;

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
    let az = AlphaZeroPlayer::new("z".to_string(), "AlphaZero".to_string(), "Z".to_string());
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
            experiences.push(Experience { state: snapshot, policy_target: policy, value_target: 0.0 });
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



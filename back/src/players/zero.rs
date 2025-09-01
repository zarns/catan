use rand::seq::SliceRandom;
use rand::Rng;
use std::f64;
use std::time::{Duration, Instant};

use crate::enums::Action;
use crate::state::State;

use super::BotPlayer;

// Hyperparameters
const ALPHAZERO_DEFAULT_SIMULATIONS: usize = 50;
const ALPHAZERO_EXPLORATION_CONSTANT: f64 = 1.5;
const MCTS_DECIDE_TIME_BUDGET_MS: u64 = 50;
const MCTS_PLAYOUT_MAX_STEPS: usize = 200;
const SMART_ROLLOUT_EXPLOIT_PCT: f64 = 0.60;
const SMART_ROLLOUT_EXPLORE_PCT: f64 = 0.30; // remainder is strategic

/// Minimal AlphaZero-style player with pure MCTS (no NN yet)
/// Single-threaded, from-scratch search per move for correctness and simplicity
pub struct AlphaZeroPlayer {
    pub id: String,
    pub name: String,
    pub color: String,
    simulations: usize,
    exploration_constant: f64,
}

impl AlphaZeroPlayer {
    pub fn new(id: String, name: String, color: String) -> Self {
        Self {
            id,
            name,
            color,
            simulations: ALPHAZERO_DEFAULT_SIMULATIONS,
            exploration_constant: ALPHAZERO_EXPLORATION_CONSTANT,
        }
    }

    pub fn with_parameters(
        id: String,
        name: String,
        color: String,
        simulations: usize,
        exploration_constant: f64,
    ) -> Self {
        Self {
            id,
            name,
            color,
            simulations,
            exploration_constant,
        }
    }
}

struct MctsNode {
    state: State,
    parent: Option<usize>,
    action_from_parent: Option<Action>,
    children: Vec<usize>,
    untried_actions: Vec<Action>,
    visits: u32,
    value_sum: f64, // accumulated reward from root player's perspective
}

impl MctsNode {
    fn new(state: State, parent: Option<usize>, action_from_parent: Option<Action>) -> Self {
        let untried_actions = state.generate_playable_actions();
        Self {
            state,
            parent,
            action_from_parent,
            children: Vec::new(),
            untried_actions,
            visits: 0,
            value_sum: 0.0,
        }
    }

    fn is_terminal(&self) -> bool {
        self.state.winner().is_some()
    }

    fn is_fully_expanded(&self) -> bool {
        self.untried_actions.is_empty()
    }
}

impl AlphaZeroPlayer {
    fn run_mcts(&self, root_state: &State) -> Action {
        let root_player = root_state.get_current_color();

        let mut nodes: Vec<MctsNode> = Vec::new();
        nodes.push(MctsNode::new(root_state.clone(), None, None));

        let deadline = Instant::now() + Duration::from_millis(MCTS_DECIDE_TIME_BUDGET_MS);
        for sim in 0..self.simulations {
            if Instant::now() >= deadline {
                break;
            }
            if sim % 100 == 0 {
                log::debug!(
                    "MCTS sim {}: nodes={}",
                    sim,
                    nodes.len()
                );
            }

            // Selection
            let mut node_index = 0usize;
            loop {
                let is_terminal = nodes[node_index].is_terminal();
                let is_fully_expanded = nodes[node_index].is_fully_expanded();
                if is_terminal || !is_fully_expanded {
                    break;
                }

                let parent_visits = nodes[node_index].visits.max(1) as f64;
                let mut best_child = None;
                let mut best_score = f64::NEG_INFINITY;
                for &child_idx in &nodes[node_index].children {
                    let child = &nodes[child_idx];
                    let avg_value = if child.visits == 0 {
                        0.0
                    } else {
                        child.value_sum / child.visits as f64
                    };
                    let exploration = self.exploration_constant
                        * ((parent_visits.ln() / (child.visits as f64 + 1e-9)).sqrt());
                    let score = avg_value + exploration;
                    if score > best_score {
                        best_score = score;
                        best_child = Some(child_idx);
                    }
                }
                if let Some(idx) = best_child {
                    node_index = idx;
                } else {
                    break;
                }
            }

            // Expansion
            let expanded_index = if !nodes[node_index].is_terminal() {
                if let Some(action) = pop_random_action(&mut nodes[node_index].untried_actions) {
                    let mut new_state = nodes[node_index].state.clone();
                    new_state.apply_action(action);
                    let new_index = nodes.len();
                    nodes.push(MctsNode::new(new_state, Some(node_index), Some(action)));
                    nodes[node_index].children.push(new_index);
                    new_index
                } else {
                    node_index
                }
            } else {
                node_index
            };

            // Simulation
            let reward = simulate_random_playout(&nodes[expanded_index].state, root_player);

            // Backpropagation
            let mut current = Some(expanded_index);
            while let Some(idx) = current {
                let node = &mut nodes[idx];
                node.visits += 1;
                node.value_sum += reward;
                current = node.parent;
            }
        }

        // Choose child with highest visits
        if nodes[0].children.is_empty() {
            // No expansion happened; fall back to the first legal action
            return *root_state
                .generate_playable_actions()
                .first()
                .expect("There should be at least one legal action");
        }

        let mut best_child = nodes[0].children[0];
        let mut best_visits = nodes[best_child].visits;
        for &child_idx in &nodes[0].children {
            let v = nodes[child_idx].visits;
            if v > best_visits {
                best_visits = v;
                best_child = child_idx;
            }
        }

        nodes[best_child]
            .action_from_parent
            .expect("Child should have an action")
    }
}

fn pop_random_action(actions: &mut Vec<Action>) -> Option<Action> {
    if actions.is_empty() {
        return None;
    }
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..actions.len());
    Some(actions.swap_remove(idx))
}

fn simulate_random_playout(start: &State, root_player: u8) -> f64 {
    let mut state = start.clone();
    let mut rng = rand::thread_rng();
    for _ in 0..MCTS_PLAYOUT_MAX_STEPS {
        if let Some(winner) = state.winner() {
            return if winner == root_player { 1.0 } else { 0.0 };
        }
        let actions = state.generate_playable_actions();
        if actions.is_empty() {
            break;
        }
        let action = select_smart_rollout_action(&state, &actions, &mut rng);
        state.apply_action(action);
    }
    0.0
}

fn select_smart_rollout_action<R: Rng + ?Sized>(state: &State, actions: &[Action], rng: &mut R) -> Action {
    if actions.len() == 1 {
        return actions[0];
    }

    let coin: f64 = rng.gen();
    if coin < SMART_ROLLOUT_EXPLOIT_PCT {
        // Exploit: pick action with best lightweight heuristic
        let mut best = actions[0];
        let mut best_score = f64::NEG_INFINITY;
        for &a in actions {
            let score = heuristic_action_score(state, a);
            if score > best_score {
                best_score = score;
                best = a;
            }
        }
        return best;
    } else if coin < SMART_ROLLOUT_EXPLOIT_PCT + SMART_ROLLOUT_EXPLORE_PCT {
        // Explore: uniform random
        return *actions
            .choose(rng)
            .expect("Non-empty actions for explore");
    } else {
        // Strategic nudge: prioritize growth actions slightly
        let mut best = actions[0];
        let mut best_bias = f64::NEG_INFINITY;
        for &a in actions {
            let bias = strategic_bias(state, a);
            if bias > best_bias {
                best_bias = bias;
                best = a;
            }
        }
        return best;
    }
}

fn heuristic_action_score(state: &State, action: Action) -> f64 {
    // Lightweight, domain-informed scoring similar to ValueFunctionPlayer but cheap
    // Evaluate the delta in immediate victory points and production potential
    let mut next = state.clone();
    next.apply_action(action);

    let me = state.get_current_color();
    let vp_now = state.get_actual_victory_points(me) as f64;
    let vp_next = next.get_actual_victory_points(me) as f64;
    let vp_delta = vp_next - vp_now;

    // Production potential: sum of effective production
    let prod = next.get_effective_production(me);
    let prod_sum: f64 = prod.iter().copied().sum();

    // Hand size penalty if above 7 pre-robber
    let hand: u8 = next.get_player_hand(me).iter().copied().sum();
    let discard_penalty = if hand > 7 { -0.5 } else { 0.0 };

    // Slight preference for building roads/settlements/cities vs. no-op
    let build_bias = match action {
        Action::BuildRoad { .. } => 0.2,
        Action::BuildSettlement { .. } => 0.6,
        Action::BuildCity { .. } => 0.8,
        Action::BuyDevelopmentCard { .. } => 0.3,
        _ => 0.0,
    };

    // Combine
    (vp_delta * 2.0) + (prod_sum * 0.1) + build_bias + discard_penalty
}

fn strategic_bias(_state: &State, action: Action) -> f64 {
    match action {
        Action::BuildRoad { .. } => 0.1,
        Action::BuildSettlement { .. } => 0.2,
        Action::BuildCity { .. } => 0.25,
        Action::BuyDevelopmentCard { .. } => 0.05,
        _ => 0.0,
    }
}

impl BotPlayer for AlphaZeroPlayer {
    fn decide(&self, state: &State, playable_actions: &[Action]) -> Action {
        if playable_actions.len() == 1 {
            return playable_actions[0];
        }
        self.run_mcts(state)
    }
}



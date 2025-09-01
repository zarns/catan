use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use std::f64;
use std::time::{Duration, Instant};
use std::{cell::RefCell, collections::HashMap};

use crate::enums::{Action, ActionPrompt};
use crate::players::nn::{candle_impl::CandleNet, noop_impl::NoopNet, types::PolicyValueNet};
use crate::state::State;

use super::BotPlayer;

// Hyperparameters
const ALPHAZERO_DEFAULT_SIMULATIONS: usize = 50;
const ALPHAZERO_EXPLORATION_CONSTANT: f64 = 1.5;
const MCTS_DECIDE_TIME_BUDGET_MS: u64 = 50;
const MCTS_PLAYOUT_MAX_STEPS: usize = 200;
const SMART_ROLLOUT_EXPLOIT_PCT: f64 = 0.60;
const SMART_ROLLOUT_EXPLORE_PCT: f64 = 0.30; // remainder is strategic
const DEFAULT_BASE_SEED: u64 = 0x5EED5EED5EED_u64;
// Progressive widening
const PW_ENABLED: bool = true;
const PW_FACTOR: f64 = 2.0; // max_children = max(1, floor(PW_FACTOR * sqrt(visits)))
                            // Root noise
const ROOT_DIRICHLET_EPS: f64 = 0.25;
const ROOT_DIRICHLET_ALPHA_BASE: f64 = 10.0; // alpha ≈ 10/|A|

/// Minimal AlphaZero-style player with pure MCTS (no NN yet)
/// Single-threaded, from-scratch search per move for correctness and simplicity
pub struct AlphaZeroPlayer {
    pub id: String,
    pub name: String,
    pub color: String,
    simulations: usize,
    exploration_constant: f64,
    base_seed: u64,
    // Simple transposition table: state_hash -> (visits, value_sum)
    tt: RefCell<HashMap<u64, (u32, f64)>>,
    net: Box<dyn PolicyValueNet>,
    // Persistent search tree reused between moves
    tree: RefCell<Option<SearchTree>>,
}

impl AlphaZeroPlayer {
    pub fn new(id: String, name: String, color: String) -> Self {
        let net: Box<dyn PolicyValueNet> = match CandleNet::new_default_device() {
            Ok(n) => Box::new(n),
            Err(_) => Box::new(NoopNet),
        };
        Self {
            id,
            name,
            color,
            simulations: ALPHAZERO_DEFAULT_SIMULATIONS,
            exploration_constant: ALPHAZERO_EXPLORATION_CONSTANT,
            base_seed: DEFAULT_BASE_SEED,
            tt: RefCell::new(HashMap::new()),
            net,
            tree: RefCell::new(None),
        }
    }

    pub fn with_parameters(
        id: String,
        name: String,
        color: String,
        simulations: usize,
        exploration_constant: f64,
    ) -> Self {
        let net: Box<dyn PolicyValueNet> = match CandleNet::new_default_device() {
            Ok(n) => Box::new(n),
            Err(_) => Box::new(NoopNet),
        };
        Self {
            id,
            name,
            color,
            simulations,
            exploration_constant,
            base_seed: DEFAULT_BASE_SEED,
            tt: RefCell::new(HashMap::new()),
            net,
            tree: RefCell::new(None),
        }
    }

    pub fn set_seed(&mut self, seed: u64) {
        self.base_seed = seed;
    }
}

struct MctsNode {
    state: State,
    parent: Option<usize>,
    action_from_parent: Option<Action>,
    children: Vec<usize>,
    untried_actions: Vec<Action>,
    visits: u32,
    value_sum: f64,               // accumulated reward from root player's perspective
    priors: HashMap<Action, f64>, // P(a|s) used for PUCT at this node
    node_hash: u64,
}

impl MctsNode {
    fn new(state: State, parent: Option<usize>, action_from_parent: Option<Action>) -> Self {
        let untried_actions = state.generate_playable_actions();
        let node_hash = state.clone().compute_hash64();
        Self {
            state,
            parent,
            action_from_parent,
            children: Vec::new(),
            untried_actions,
            visits: 0,
            value_sum: 0.0,
            priors: HashMap::new(),
            node_hash,
        }
    }

    fn is_terminal(&self) -> bool {
        self.state.winner().is_some()
    }
}

struct SearchTree {
    nodes: Vec<MctsNode>,
    root: usize,
    index_by_hash: HashMap<u64, usize>,
}

impl SearchTree {
    fn new_with_root(state: State) -> Self {
        let root_node = MctsNode::new(state, None, None);
        let mut index_by_hash = HashMap::new();
        index_by_hash.insert(root_node.node_hash, 0);
        Self {
            nodes: vec![root_node],
            root: 0,
            index_by_hash,
        }
    }

    fn add_node(&mut self, node: MctsNode) -> usize {
        let idx = self.nodes.len();
        let h = node.node_hash;
        self.nodes.push(node);
        self.index_by_hash.entry(h).or_insert(idx);
        idx
    }
}

impl AlphaZeroPlayer {
    fn softmax_normalize(scores: &mut [f64]) {
        if scores.is_empty() {
            return;
        }
        let max = scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let mut sum = 0.0;
        for s in scores.iter_mut() {
            *s = (*s - max).exp();
            sum += *s;
        }
        if sum > 0.0 {
            for s in scores.iter_mut() {
                *s /= sum;
            }
        } else {
            let inv = 1.0 / (scores.len() as f64);
            for s in scores.iter_mut() {
                *s = inv;
            }
        }
    }

    fn predict_priors_heuristic(&self, state: &State, actions: &[Action]) -> HashMap<Action, f64> {
        let scores: Vec<(Action, f64)> = actions
            .iter()
            .copied()
            .map(|a| (a, heuristic_action_score(state, a)))
            .collect();
        let mut vals: Vec<f64> = scores.iter().map(|(_, s)| *s).collect();
        Self::softmax_normalize(&mut vals);
        let mut out = HashMap::with_capacity(actions.len());
        for (i, (a, _)) in scores.into_iter().enumerate() {
            out.insert(a, vals[i]);
        }
        out
    }

    fn ensure_priors(&self, node: &mut MctsNode) {
        if !node.priors.is_empty() {
            return;
        }
        if node.untried_actions.is_empty() {
            // No untried actions; leave priors empty and fall back to uniform in selection
            return;
        }
        // Prefer network priors; fallback to heuristic softmax if needed
        let pv = self
            .net
            .infer_policy_value(&node.state, &node.untried_actions);
        if !pv.priors.is_empty() {
            node.priors = pv.priors.into_iter().map(|(a, p)| (a, p as f64)).collect();
            return;
        }
        // Predict priors for unexpanded actions via heuristic
        let priors = self.predict_priors_heuristic(&node.state, &node.untried_actions);
        node.priors = priors;
    }

    fn puct_select_child(&self, nodes: &[MctsNode], node_index: usize) -> Option<usize> {
        let parent = &nodes[node_index];
        if parent.children.is_empty() {
            return None;
        }
        let parent_visits = parent.visits.max(1) as f64;
        let mut best: Option<(usize, f64)> = None;
        for &child_idx in &parent.children {
            let child = &nodes[child_idx];
            let q = if child.visits == 0 {
                0.0
            } else {
                child.value_sum / (child.visits as f64)
            };
            let a = child.action_from_parent.expect("child must have action");
            let p = parent
                .priors
                .get(&a)
                .copied()
                .unwrap_or(1.0_f64 / (parent.children.len().max(1) as f64));
            let u = self.exploration_constant * p * (parent_visits.sqrt())
                / (1.0 + child.visits as f64);
            let score = q + u;
            match best {
                None => best = Some((child_idx, score)),
                Some((_, best_score)) if score > best_score => best = Some((child_idx, score)),
                _ => {}
            }
        }
        best.map(|(idx, _)| idx)
    }

    fn new_rng_for_state(&self, root_state: &State, sim_counter: u64) -> StdRng {
        let salt = root_state.compute_hash64() ^ self.base_seed ^ sim_counter;
        StdRng::seed_from_u64(salt)
    }

    fn can_expand(&self, node: &MctsNode) -> bool {
        if node.untried_actions.is_empty() {
            return false;
        }
        if !PW_ENABLED {
            return true;
        }
        let allowed = ((PW_FACTOR * (node.visits as f64).sqrt()) as usize).max(1);
        node.children.len() < allowed
    }

    fn apply_root_noise<R: Rng + ?Sized>(&self, node: &mut MctsNode, rng: &mut R) {
        use rand_distr::Distribution;
        let count = node.priors.len();
        if count == 0 {
            return;
        }
        let alpha = (ROOT_DIRICHLET_ALPHA_BASE / (count as f64)).clamp(0.03, 0.5);
        let gamma = rand_distr::Gamma::new(alpha as f32, 1.0).unwrap();
        let noise: Vec<(Action, f64)> = node
            .priors
            .keys()
            .copied()
            .map(|a| (a, gamma.sample(rng) as f64))
            .collect();
        let sum: f64 = noise.iter().map(|(_, v)| *v).sum::<f64>().max(1e-12);
        let inv_sum = 1.0 / sum;
        for (a, n) in noise.into_iter() {
            let p = node.priors.get_mut(&a).unwrap();
            let dir = n * inv_sum;
            *p = (1.0 - ROOT_DIRICHLET_EPS) * (*p) + ROOT_DIRICHLET_EPS * dir;
        }
    }

    fn run_mcts(&self, root_state: &State) -> Action {
        let root_player = root_state.get_current_color();

        // Align or create the persistent tree
        let current_hash = root_state.compute_hash64();
        {
            let mut opt = self.tree.borrow_mut();
            if let Some(tree) = opt.as_mut() {
                if let Some(&idx) = tree.index_by_hash.get(&current_hash) {
                    tree.root = idx;
                } else {
                    *tree = SearchTree::new_with_root(root_state.clone());
                }
            } else {
                *opt = Some(SearchTree::new_with_root(root_state.clone()));
            }
            let t = opt.as_mut().unwrap();
            self.ensure_priors(t.nodes.get_mut(t.root).expect("root exists"));
        }

        let deadline = Instant::now() + Duration::from_millis(MCTS_DECIDE_TIME_BUDGET_MS);
        for sim in 0..self.simulations {
            let mut rng = self.new_rng_for_state(root_state, sim as u64);
            if Instant::now() >= deadline {
                break;
            }
            if sim == 0 {
                // Add Dirichlet noise at root once per search
                let mut tref = self.tree.borrow_mut();
                let tree_mut = tref.as_mut().unwrap();
                let root_node = tree_mut.nodes.get_mut(tree_mut.root).expect("root exists");
                self.apply_root_noise(root_node, &mut rng);
            }
            if sim % 100 == 0 {
                log::debug!(
                    "MCTS sim {}: nodes= {}",
                    sim,
                    self.tree
                        .borrow()
                        .as_ref()
                        .map(|t| t.nodes.len())
                        .unwrap_or(0)
                );
            }

            // Selection
            let mut node_index = self.tree.borrow().as_ref().unwrap().root;
            loop {
                let is_terminal =
                    self.tree.borrow().as_ref().unwrap().nodes[node_index].is_terminal();
                let can_expand = {
                    let t = self.tree.borrow();
                    let node = &t.as_ref().unwrap().nodes[node_index];
                    self.can_expand(node)
                };
                if is_terminal || can_expand {
                    break;
                }
                // Ensure priors computed for parent
                {
                    let mut tref = self.tree.borrow_mut();
                    let tree_mut = tref.as_mut().unwrap();
                    let parent_mut = tree_mut.nodes.get_mut(node_index).expect("node exists");
                    self.ensure_priors(parent_mut);
                }

                if let Some(idx) =
                    self.puct_select_child(&self.tree.borrow().as_ref().unwrap().nodes, node_index)
                {
                    node_index = idx;
                } else {
                    break;
                }
            }

            // Expansion (only if allowed by progressive widening)
            let expanded_index = {
                let can_expand_now = {
                    let t = self.tree.borrow();
                    let node = &t.as_ref().unwrap().nodes[node_index];
                    self.can_expand(node)
                };
                if can_expand_now {
                    let action_opt = {
                        let mut b = self.tree.borrow_mut();
                        let t = b.as_mut().unwrap();
                        pop_random_action(&mut t.nodes[node_index].untried_actions, &mut rng)
                    };
                    if let Some(action) = action_opt {
                        let mut new_state = self.tree.borrow().as_ref().unwrap().nodes[node_index]
                            .state
                            .clone();
                        new_state.apply_action(action);
                        let new_index = {
                            let mut b = self.tree.borrow_mut();
                            let t = b.as_mut().unwrap();
                            let mut new_node =
                                MctsNode::new(new_state, Some(node_index), Some(action));
                            // TT bootstrapping
                            if let Some((v, sum)) =
                                self.tt.borrow().get(&new_node.node_hash).copied()
                            {
                                new_node.visits = v;
                                new_node.value_sum = sum;
                            }
                            let idx = t.add_node(new_node);
                            t.nodes[node_index].children.push(idx);
                            idx
                        };
                        // Initialize priors for the new node too
                        {
                            let mut b = self.tree.borrow_mut();
                            let t = b.as_mut().unwrap();
                            self.ensure_priors(
                                t.nodes.get_mut(new_index).expect("new node exists"),
                            );
                        }
                        new_index
                    } else {
                        node_index
                    }
                } else {
                    node_index
                }
            };

            // Simulation
            let reward = {
                let st = self.tree.borrow();
                let t = st.as_ref().unwrap();
                simulate_random_playout(&t.nodes[expanded_index].state, root_player, &mut rng)
            };

            // Backpropagation
            let mut current = Some(expanded_index);
            while let Some(idx) = current {
                {
                    let mut b = self.tree.borrow_mut();
                    let t = b.as_mut().unwrap();
                    let node = &mut t.nodes[idx];
                    node.visits += 1;
                    node.value_sum += reward;
                    // Update TT
                    let h = node.node_hash;
                    if let Ok(mut tt) = self.tt.try_borrow_mut() {
                        let entry = tt.entry(h).or_insert((0u32, 0.0));
                        entry.0 = entry.0.saturating_add(1);
                        entry.1 += reward;
                    }
                    current = node.parent;
                }
            }
        }

        // Choose child with highest visits
        if self.tree.borrow().as_ref().unwrap().nodes[self.tree.borrow().as_ref().unwrap().root]
            .children
            .is_empty()
        {
            // No expansion happened; fall back to the first legal action
            return *root_state
                .generate_playable_actions()
                .first()
                .expect("There should be at least one legal action");
        }

        let (best_child, _best_visits) = {
            let st = self.tree.borrow();
            let t = st.as_ref().unwrap();
            let root_idx = t.root;
            let mut best_child = t.nodes[root_idx].children[0];
            let mut best_visits = t.nodes[best_child].visits;
            for &child_idx in &t.nodes[root_idx].children {
                let v = t.nodes[child_idx].visits;
                if v > best_visits {
                    best_visits = v;
                    best_child = child_idx;
                }
            }
            (best_child, best_visits)
        };

        // Re-root for next move
        {
            let mut b = self.tree.borrow_mut();
            let t = b.as_mut().unwrap();
            t.root = best_child;
        }

        self.tree.borrow().as_ref().unwrap().nodes[best_child]
            .action_from_parent
            .expect("Child should have an action")
    }
}

fn pop_random_action<R: Rng + ?Sized>(actions: &mut Vec<Action>, rng: &mut R) -> Option<Action> {
    if actions.is_empty() {
        return None;
    }
    let idx = rng.gen_range(0..actions.len());
    Some(actions.swap_remove(idx))
}

fn simulate_random_playout<R: Rng + ?Sized>(start: &State, root_player: u8, rng: &mut R) -> f64 {
    let mut state = start.clone();
    for _ in 0..MCTS_PLAYOUT_MAX_STEPS {
        if let Some(winner) = state.winner() {
            return if winner == root_player { 1.0 } else { 0.0 };
        }
        let actions = state.generate_playable_actions();
        if actions.is_empty() {
            break;
        }
        let action = select_smart_rollout_action(&state, &actions, rng);
        state.apply_action(action);
    }
    0.0
}

fn select_smart_rollout_action<R: Rng + ?Sized>(
    state: &State,
    actions: &[Action],
    rng: &mut R,
) -> Action {
    if actions.len() == 1 {
        return actions[0];
    }

    let coin: f64 = rng.gen::<f64>();
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
        best
    } else if coin < SMART_ROLLOUT_EXPLOIT_PCT + SMART_ROLLOUT_EXPLORE_PCT {
        // Explore: uniform random
        return *actions.choose(rng).expect("Non-empty actions for explore");
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
        best
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
        match state.get_action_prompt() {
            ActionPrompt::PlayTurn => self.run_mcts(state),
            // Fast-path other prompts deterministically
            _ => {
                let mut rng = self.new_rng_for_state(state, 0);
                *playable_actions
                    .choose(&mut rng)
                    .expect("There should be at least one playable action")
            }
        }
    }
}

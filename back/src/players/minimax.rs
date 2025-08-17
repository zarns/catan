use log::LevelFilter;
use std::f64;

use super::value::ValueWeights;
use crate::enums::Action;
use crate::map_instance::{EdgeId, NodeId};
use crate::state::State;
use rand::Rng;
use std::collections::HashMap;
use std::time::Instant;

const DEFAULT_DEPTH: i32 = 5; // increased default depth
const MAX_ORDERED_ACTIONS: usize = 20; // wider beam for better coverage below root
const ROOT_MAX_ORDERED_ACTIONS: usize = 24; // root-only wider beam
const PVS_EPS: f64 = 1e-6;
const LMR_MIN_DEPTH: i32 = 3; // enable reductions at shallower depths
const LMR_LATE_INDEX: usize = 4; // reduce depth for moves after this index
const SHALLOW_EVAL_WEIGHT: f64 = 0.10; // higher to improve ordering
const HISTORY_WEIGHT: f64 = 0.001; // make history matter
const KILLER_BONUS: f64 = 100.0; // scale to static scores
const ASPIRATION_MIN_WINDOW: f64 = 15.0; // tighter initial window

use super::BotPlayer;

/// Alpha-Beta Minimax Player
/// Uses the minimax algorithm to choose actions
// Type aliases to simplify complex types
type TtKey = (u64, i32);
type TtEntry = (f64, i8, Option<Action>);
type TtMap = HashMap<TtKey, TtEntry>;
type KillerPair = (Option<Action>, Option<Action>);
type KillerMap = HashMap<i32, KillerPair>;
type HistoryMap = HashMap<Action, i64>;

#[derive(Clone, Copy)]
struct SearchCtx {
    depth: i32,
    alpha: f64,
    beta: f64,
    my_color: u8,
    deadline: Option<Instant>,
}

pub struct AlphaBetaPlayer {
    pub id: String,
    pub name: String,
    pub color: String,
    depth: i32,
    time_profile: SearchTimeProfile,
    weights: ValueWeights,
    tt: std::cell::RefCell<TtMap>, // (hash, depth) -> (value, flag: -1=alpha, 0=exact, 1=beta, best_move)
    epsilon: Option<f64>,
    killer_moves: std::cell::RefCell<KillerMap>, // depth -> (killer1, killer2)
    history_scores: std::cell::RefCell<HistoryMap>, // action -> score
    node_production_cache: std::cell::RefCell<HashMap<NodeId, f64>>,
}

#[derive(Clone, Copy)]
pub struct SearchTimeProfile {
    pub fast_ms: u64,
    pub slow_ms: u64,
    pub slow_branch_threshold: usize,
}

impl SearchTimeProfile {
    pub const FAST: Self = Self {
        fast_ms: 60,
        slow_ms: 100,
        slow_branch_threshold: 12,
    };
    pub const BALANCED: Self = Self {
        fast_ms: 80,
        slow_ms: 130,
        slow_branch_threshold: 14,
    };
    pub const DEEP: Self = Self {
        fast_ms: 100,
        slow_ms: 180,
        slow_branch_threshold: 16,
    };
    pub const ULTRA: Self = Self {
        fast_ms: 10000,
        slow_ms: 10000,
        slow_branch_threshold: 12,
    };
}

impl AlphaBetaPlayer {
    pub fn new(id: String, name: String, color: String) -> Self {
        AlphaBetaPlayer {
            id,
            name,
            color,
            depth: DEFAULT_DEPTH,
            time_profile: SearchTimeProfile::DEEP,
            weights: ValueWeights::contender(),
            tt: std::cell::RefCell::new(HashMap::with_capacity(1 << 18)),
            epsilon: None,
            killer_moves: std::cell::RefCell::new(HashMap::with_capacity(512)),
            history_scores: std::cell::RefCell::new(HashMap::with_capacity(1024)),
            node_production_cache: std::cell::RefCell::new(HashMap::with_capacity(1024)),
        }
    }

    pub fn with_depth(id: String, name: String, color: String, depth: i32) -> Self {
        AlphaBetaPlayer {
            id,
            name,
            color,
            depth,
            time_profile: SearchTimeProfile::DEEP,
            weights: ValueWeights::contender(),
            tt: std::cell::RefCell::new(HashMap::with_capacity(1 << 18)),
            epsilon: None,
            killer_moves: std::cell::RefCell::new(HashMap::with_capacity(512)),
            history_scores: std::cell::RefCell::new(HashMap::with_capacity(1024)),
            node_production_cache: std::cell::RefCell::new(HashMap::with_capacity(1024)),
        }
    }

    /// Construct with explicit weights, time budget, and optional epsilon exploration
    pub fn with_config(
        id: String,
        name: String,
        color: String,
        depth: i32,
        weights: ValueWeights,
        epsilon: Option<f64>,
    ) -> Self {
        AlphaBetaPlayer {
            id,
            name,
            color,
            depth,
            time_profile: SearchTimeProfile {
                fast_ms: 100,
                slow_ms: 150,
                slow_branch_threshold: 14,
            },
            weights,
            tt: std::cell::RefCell::new(HashMap::with_capacity(1 << 18)),
            epsilon,
            killer_moves: std::cell::RefCell::new(HashMap::with_capacity(512)),
            history_scores: std::cell::RefCell::new(HashMap::with_capacity(1024)),
            node_production_cache: std::cell::RefCell::new(HashMap::with_capacity(1024)),
        }
    }

    pub fn set_weights(&mut self, weights: ValueWeights) {
        self.weights = weights;
    }
    pub fn set_epsilon(&mut self, epsilon: Option<f64>) {
        self.epsilon = epsilon;
    }

    /// Configure a dual time profile: use `slow_ms` when branching is large, otherwise `fast_ms`.
    pub fn set_time_profile(&mut self, fast_ms: u64, slow_ms: u64, slow_branch_threshold: usize) {
        self.time_profile = SearchTimeProfile {
            fast_ms,
            slow_ms,
            slow_branch_threshold,
        };
    }

    /// Evaluate the game state from the perspective of the given player using ValueWeights
    fn evaluate_state(&self, state: &State, p0_color: u8) -> f64 {
        let w = &self.weights;

        // Victory points
        let vps = state.get_actual_victory_points(p0_color) as f64;

        // Production (effective, considering robber)
        let my_prod = state.get_effective_production(p0_color);
        let my_prod_value = self.value_production(&my_prod, true);

        // Enemy production (average over opponents)
        let mut enemy_acc = 0.0;
        let mut enemy_cnt = 0.0;
        for color in 0..state.get_num_players() {
            if color == p0_color {
                continue;
            }
            let p = state.get_effective_production(color);
            enemy_acc += self.value_production(&p, false);
            enemy_cnt += 1.0;
        }
        let enemy_prod_value = if enemy_cnt > 0.0 {
            enemy_acc / enemy_cnt
        } else {
            0.0
        };

        // Reachability placeholders (0 until implemented)
        let reachable_production_at_zero = 0.0;
        let reachable_production_at_one = 0.0;

        // Hand features
        let hand = state.get_player_hand(p0_color);
        let num_in_hand: u8 = hand.iter().copied().sum();
        let discard_penalty = if num_in_hand > 7 {
            w.discard_penalty
        } else {
            0.0
        };
        let hand_devs = state
            .get_player_devhand(p0_color)
            .iter()
            .map(|&x| x as f64)
            .sum::<f64>();
        let army_size = state
            .get_played_dev_card_count(p0_color, crate::enums::DevCard::Knight as usize)
            as f64;
        let hand_synergy = self.hand_synergy(state, p0_color);

        // Board features
        let num_buildable_nodes = state.buildable_node_ids(p0_color).len() as f64;
        let num_tiles = self.count_my_owned_tiles(state, p0_color) as f64;

        // Longest road factor placeholder
        let longest_road_factor = if num_buildable_nodes == 0.0 {
            w.longest_road
        } else {
            0.1
        };
        let longest_road_length = 0.0;

        vps * w.public_vps
            + my_prod_value * w.production
            + enemy_prod_value * w.enemy_production
            + reachable_production_at_zero * w.reachable_production_0
            + reachable_production_at_one * w.reachable_production_1
            + hand_synergy * w.hand_synergy
            + num_buildable_nodes * w.buildable_nodes
            + num_tiles * w.num_tiles
            + (num_in_hand as f64) * w.hand_resources
            + discard_penalty
            + longest_road_length * longest_road_factor
            + hand_devs * w.hand_devs
            + army_size * w.army_size
    }

    fn value_production(&self, production: &[f64], include_variety: bool) -> f64 {
        const TRANSLATE_VARIETY: f64 = 4.0;
        const PROBA_POINT: f64 = 2.778 / 100.0;
        let sum: f64 = production.iter().copied().sum();
        let variety_count = production.iter().filter(|&&p| p > 0.0).count() as f64;
        let variety_bonus = if include_variety {
            variety_count * TRANSLATE_VARIETY * PROBA_POINT
        } else {
            0.0
        };
        sum + variety_bonus
    }

    fn hand_synergy(&self, state: &State, color: u8) -> f64 {
        let hand = state.get_player_hand(color);
        let wheat = hand.get(3).copied().unwrap_or(0) as i32;
        let ore = hand.get(4).copied().unwrap_or(0) as i32;
        let sheep = hand.get(2).copied().unwrap_or(0) as i32;
        let brick = hand.get(1).copied().unwrap_or(0) as i32;
        let wood = hand.first().copied().unwrap_or(0) as i32;

        let distance_to_city = ((2 - wheat).max(0) + (3 - ore).max(0)) as f64 / 5.0;
        let distance_to_settlement =
            ((1 - wheat).max(0) + (1 - sheep).max(0) + (1 - brick).max(0) + (1 - wood).max(0))
                as f64
                / 4.0;
        (2.0 - distance_to_city - distance_to_settlement) / 2.0
    }

    fn count_my_owned_tiles(&self, state: &State, color: u8) -> usize {
        use std::collections::HashSet;
        let mut tiles: HashSet<u8> = HashSet::new();
        let map = state.get_map_instance();
        for b in state.get_settlements(color) {
            if let crate::state::Building::Settlement(_, node) = b {
                if let Some(adj) = map.get_adjacent_tiles(node) {
                    for t in adj {
                        tiles.insert(t.id);
                    }
                }
            }
        }
        for b in state.get_cities(color) {
            if let crate::state::Building::City(_, node) = b {
                if let Some(adj) = map.get_adjacent_tiles(node) {
                    for t in adj {
                        tiles.insert(t.id);
                    }
                }
            }
        }
        tiles.len()
    }

    /// Get the relative evaluation (my score - average opponent score)
    fn evaluate_relative(&self, state: &State, my_color: u8) -> f64 {
        let my_score = self.evaluate_state(state, my_color);

        let mut opponent_scores = 0.0;
        let num_players = state.get_num_players();

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

    /// Heuristic move ordering combining static scores with a shallow evaluation.
    fn order_actions(&self, state: &State, actions: &[Action], depth: i32) -> Vec<Action> {
        let my_color = state.get_current_color();
        let mut scored: Vec<(f64, Action)> = Vec::with_capacity(actions.len());
        for &a in actions {
            let mut static_score = self.score_action(state, a) as f64;
            // Small ordering boost if this blocks opponent expansion
            if let Action::BuildSettlement { node_id, .. } = a {
                if self.blocks_opponent_expansion(state, node_id) {
                    static_score += 100.0;
                }
            }
            // Quick 0-ply evaluation to improve ordering (skip expensive chance nodes)
            let quick_eval = match a {
                Action::Roll { dice_opt: None, .. } => 0.0,
                _ => {
                    let mut ns = state.clone();
                    ns.apply_action(a);
                    self.evaluate_relative(&ns, my_color)
                }
            };
            // History weight
            let hist = *self.history_scores.borrow().get(&a).unwrap_or(&0) as f64;
            // Killer bonus if this action is a killer at this depth
            let mut killer_bonus = 0.0;
            if let Some(&(k1, k2)) = self.killer_moves.borrow().get(&depth) {
                if Some(a) == k1 || Some(a) == k2 {
                    killer_bonus = KILLER_BONUS;
                }
            }
            // Weighted sum to bias ordering
            let combined = static_score * 1.0
                + quick_eval * SHALLOW_EVAL_WEIGHT
                + hist * HISTORY_WEIGHT
                + killer_bonus;
            scored.push((combined, a));
        }
        scored.sort_by(|(sa, _), (sb, _)| sb.partial_cmp(sa).unwrap_or(std::cmp::Ordering::Equal));
        scored
            .into_iter()
            .take(MAX_ORDERED_ACTIONS)
            .map(|(_, a)| a)
            .collect()
    }

    #[inline]
    fn is_quiet_move(&self, action: Action) -> bool {
        use crate::enums::Action as A;
        matches!(
            action,
            A::BuildRoad { .. }
                | A::MaritimeTrade { .. }
                | A::OfferTrade { .. }
                | A::AcceptTrade { .. }
                | A::RejectTrade { .. }
                | A::ConfirmTrade { .. }
                | A::CancelTrade { .. }
                | A::EndTurn { .. }
        )
    }

    #[inline]
    fn get_lmr_reduction(&self, depth: i32, move_index: usize, is_quiet: bool) -> i32 {
        if !is_quiet || depth < LMR_MIN_DEPTH {
            return 0;
        }
        // Conservative reductions: only late moves, reduce by 1
        if depth >= 4 && move_index >= LMR_LATE_INDEX + 4 {
            return 1;
        }
        if depth >= 3 && move_index >= LMR_LATE_INDEX + 2 {
            return 1;
        }
        0
    }

    fn quiescence_eval(&self, state: &State, my_color: u8) -> f64 {
        use crate::enums::Action as A;
        // Look for tactical moves that can change evaluation significantly
        let actions = state.generate_playable_actions();
        let mut tactical: Vec<Action> = Vec::new();
        for &a in &actions {
            match a {
                A::BuildCity { .. }
                | A::BuildSettlement { .. }
                | A::BuyDevelopmentCard { .. }
                | A::PlayKnight { .. }
                | A::PlayRoadBuilding { .. }
                | A::PlayYearOfPlenty { .. }
                | A::PlayMonopoly { .. }
                | A::MoveRobber {
                    victim_opt: Some(_),
                    ..
                } => tactical.push(a),
                _ => {}
            }
            if tactical.len() >= 6 {
                break;
            }
        }
        if tactical.is_empty() {
            return self.evaluate_relative(state, my_color);
        }
        let is_max = state.get_current_color() == my_color;
        let mut best = if is_max {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
        for a in tactical {
            let mut ns = state.clone();
            ns.apply_action(a);
            let v = self.evaluate_relative(&ns, my_color);
            if is_max {
                if v > best {
                    best = v;
                }
            } else if v < best {
                best = v;
            }
        }
        best
    }

    fn score_action(&self, state: &State, action: Action) -> i32 {
        use crate::enums::Action as A;
        let my_color = state.get_current_color();
        let my_vps = state.get_actual_victory_points(my_color);
        let end_game = my_vps >= 7;
        let leader = self.get_leader(state);
        let am_leader = leader == my_color;
        let leader_vps = state.get_actual_victory_points(leader);
        match action {
            A::BuildCity { node_id, .. } => {
                let base = if end_game { 1200 } else { 900 };
                let prod = self.estimate_node_production(state, node_id);
                base + (prod * 10.0) as i32
            }
            A::BuildSettlement { node_id, .. } => {
                let base = if end_game { 700 } else { 1000 };
                let prod = self.estimate_node_production(state, node_id);
                let expansion_bonus = self.count_buildable_from_node(state, node_id) as i32 * 20;
                let blocks_opponent = self.blocks_opponent_expansion(state, node_id);
                let defensive_bonus = if blocks_opponent && !am_leader {
                    150
                } else {
                    0
                };
                base + (prod * 15.0) as i32 + expansion_bonus + defensive_bonus
            }
            A::BuildRoad { edge_id, .. } => {
                let base = 200;
                let longest_bonus = if self.in_longest_road_race(state, my_color) {
                    500
                } else {
                    0
                };
                let expansion_bonus = if self.opens_settlement_spot(state, edge_id) {
                    300
                } else {
                    0
                };
                base + longest_bonus + expansion_bonus
            }
            A::BuyDevelopmentCard { .. } => {
                let hand_size: u8 = state.get_player_hand(my_color).iter().copied().sum();
                let surplus_bonus = if hand_size > 5 { 100 } else { 0 };
                let army_bonus = if self.would_claim_largest_army(state, my_color) {
                    50
                } else {
                    0
                };
                250 + surplus_bonus + army_bonus
            }
            A::PlayKnight { .. } => {
                let army_bonus = if self.would_claim_largest_army(state, my_color) {
                    300
                } else {
                    0
                };
                220 + army_bonus
            }
            A::PlayRoadBuilding { .. } => 220,
            A::PlayYearOfPlenty { .. } => 200,
            A::PlayMonopoly { .. } => 180,
            A::MaritimeTrade { .. } => {
                if self.one_resource_from_building(state, my_color) {
                    500
                } else {
                    120
                }
            }
            A::OfferTrade { .. } => 60,
            A::AcceptTrade { .. } => 60,
            A::ConfirmTrade { .. } => 60,
            A::RejectTrade { .. } => 40,
            A::CancelTrade { .. } => 30,
            A::MoveRobber {
                coordinate,
                victim_opt: Some(victim),
                ..
            } => {
                let impact = self.estimate_robber_impact(state, coordinate, victim);
                let leader_block_bonus = if victim == leader && leader_vps >= 8 {
                    500
                } else if victim == leader {
                    200
                } else {
                    50
                };
                10 + leader_block_bonus + (impact * 100.0) as i32
            }
            A::MoveRobber { .. } => 5,
            A::Roll { .. } => 10,
            A::Discard { .. } => 0,
            A::EndTurn { .. } => 1,
        }
    }

    fn estimate_node_production(&self, state: &State, node_id: NodeId) -> f64 {
        if let Some(&cached) = self.node_production_cache.borrow().get(&node_id) {
            return cached;
        }
        let mut weighted = 0.0;
        if let Some(node_prod) = state.get_map_instance().get_node_production(node_id) {
            for (resource, prob) in node_prod.iter() {
                let res_idx: u8 = match resource {
                    crate::enums::Resource::Wood => 0,
                    crate::enums::Resource::Brick => 1,
                    crate::enums::Resource::Sheep => 2,
                    crate::enums::Resource::Wheat => 3,
                    crate::enums::Resource::Ore => 4,
                };
                let scarcity = self.get_resource_scarcity(state, res_idx);
                weighted += prob * scarcity;
            }
        }
        self.node_production_cache
            .borrow_mut()
            .insert(node_id, weighted);
        weighted
    }

    fn get_resource_scarcity(&self, _state: &State, resource: u8) -> f64 {
        match resource {
            3 | 4 => 1.3, // wheat/ore slightly more valuable
            _ => 1.0,
        }
    }

    fn count_buildable_from_node(&self, state: &State, node_id: NodeId) -> usize {
        let buildable = state.buildable_node_ids(state.get_current_color());
        if buildable.contains(&node_id) {
            1
        } else {
            0
        }
    }

    fn in_longest_road_race(&self, state: &State, color: u8) -> bool {
        let roads = state.get_roads_by_color();
        let my = *roads.get(color as usize).unwrap_or(&0) as i32;
        let mut max_opponent = 0i32;
        for (idx, &r) in roads.iter().enumerate() {
            if idx as u8 != color && (r as i32) > max_opponent {
                max_opponent = r as i32;
            }
        }
        my >= 4 && my >= max_opponent - 1
    }

    fn opens_settlement_spot(&self, state: &State, edge_id: EdgeId) -> bool {
        let buildable = state.buildable_node_ids(state.get_current_color());
        let board_spots = state.get_map_instance().land_nodes();
        let (a, b) = edge_id;
        let a_is_new = board_spots.contains(&a) && !buildable.contains(&a);
        let b_is_new = board_spots.contains(&b) && !buildable.contains(&b);
        a_is_new || b_is_new
    }

    fn would_claim_largest_army(&self, state: &State, color: u8) -> bool {
        let my_knights =
            state.get_played_dev_card_count(color, crate::enums::DevCard::Knight as usize) as i32;
        my_knights + 1 >= 3
    }

    fn one_resource_from_building(&self, state: &State, color: u8) -> bool {
        let hand = state.get_player_hand(color);
        let wood = hand.first().copied().unwrap_or(0) as i32;
        let brick = hand.get(1).copied().unwrap_or(0) as i32;
        let sheep = hand.get(2).copied().unwrap_or(0) as i32;
        let wheat = hand.get(3).copied().unwrap_or(0) as i32;
        let ore = hand.get(4).copied().unwrap_or(0) as i32;
        let missing_settlement =
            (1 - wood).max(0) + (1 - brick).max(0) + (1 - sheep).max(0) + (1 - wheat).max(0);
        let missing_city = (2 - wheat).max(0) + (3 - ore).max(0);
        missing_settlement <= 1 || missing_city <= 1
    }

    fn estimate_robber_impact(
        &self,
        state: &State,
        coordinate: crate::map_template::Coordinate,
        victim: u8,
    ) -> f64 {
        let mut ns = state.clone();
        if let Some(tile) = ns.get_map_instance().get_land_tile(coordinate) {
            let before_victim = self.value_production(&ns.get_effective_production(victim), false);
            let my_color = state.get_current_color();
            let before_me = self.value_production(&ns.get_effective_production(my_color), false);
            ns.set_robber_tile(tile.id);
            let after_victim = self.value_production(&ns.get_effective_production(victim), false);
            let after_me = self.value_production(&ns.get_effective_production(my_color), false);
            (before_victim - after_victim) - (before_me - after_me)
        } else {
            0.0
        }
    }

    fn get_leader(&self, state: &State) -> u8 {
        let mut best_color = 0u8;
        let mut best_vp = -1i32;
        for color in 0..state.get_num_players() {
            let vp = state.get_actual_victory_points(color) as i32;
            if vp > best_vp {
                best_vp = vp;
                best_color = color;
            }
        }
        best_color
    }

    fn blocks_opponent_expansion(&self, state: &State, node_id: NodeId) -> bool {
        let my_color = state.get_current_color();
        // If any opponent currently has this node buildable, or a neighbor buildable,
        // then placing here would block their expansion per distance-2 rule.
        let neighbors: Vec<NodeId> = state.get_map_instance().get_neighbor_nodes(node_id);
        for opp in 0..state.get_num_players() {
            if opp == my_color {
                continue;
            }
            let opp_buildable = state.buildable_node_ids(opp);
            if opp_buildable.contains(&node_id) {
                return true;
            }
            if neighbors.iter().any(|n| opp_buildable.contains(n)) {
                return true;
            }
        }
        false
    }

    /// Domain-aware pruning hook. Currently a passthrough; to be expanded with
    /// conservative eliminations (e.g., dominated maritime trades, low-impact robber moves).
    fn prune_actions(&self, state: &State, actions: &[Action]) -> Vec<Action> {
        use crate::enums::Action as A;
        // Initial conservative pruning: during initial placement, drop 1-tile settlement spots
        if state.is_initial_build_phase() {
            let mut pruned = Vec::with_capacity(actions.len());
            for &a in actions {
                match a {
                    A::BuildSettlement { node_id, .. } => {
                        let adj = state.get_map_instance().get_adjacent_tiles(node_id);
                        if let Some(adj_tiles) = adj {
                            if adj_tiles.len() > 1 {
                                pruned.push(a);
                            }
                        } else {
                            pruned.push(a);
                        }
                    }
                    _ => pruned.push(a),
                }
            }
            return pruned;
        }
        // Base filtered list (no maritime pruning needed; move_generation already uses best port rates)
        let filtered: Vec<Action> = actions.to_vec();

        // Robber compression: keep only the most impactful MoveRobber action (if any)
        let mut best_idx: Option<usize> = None;
        let mut best_impact = f64::NEG_INFINITY;
        let mover = state.get_current_color();
        for (idx, a) in filtered.iter().enumerate() {
            if let A::MoveRobber {
                coordinate,
                victim_opt: Some(victim),
                ..
            } = a
            {
                // Simulate setting robber on tile; skip steal effect for speed
                let mut ns = state.clone();
                if let Some(tile) = ns.get_map_instance().get_land_tile(*coordinate) {
                    ns.set_robber_tile(tile.id);
                    // Compute impact: enemy production - our production
                    let enemy_prod =
                        self.value_production(&ns.get_effective_production(*victim), false);
                    let my_prod = self.value_production(&ns.get_effective_production(mover), false);
                    let impact = enemy_prod - my_prod;
                    if impact > best_impact {
                        best_impact = impact;
                        best_idx = Some(idx);
                    }
                }
            }
        }
        if let Some(keep_idx) = best_idx {
            let mut kept: Vec<Action> = Vec::with_capacity(filtered.len());
            for (i, a) in filtered.into_iter().enumerate() {
                let keep = match a {
                    A::MoveRobber { .. } => i == keep_idx,
                    _ => true,
                };
                if keep {
                    kept.push(a);
                }
            }
            return kept;
        }
        filtered
    }

    /// Evaluate an action, expanding stochastic outcomes into an expected value when needed.
    fn evaluate_action_with_chance(&self, state: &State, action: Action, ctx: &SearchCtx) -> f64 {
        match action {
            Action::Roll {
                color,
                dice_opt: None,
            } => {
                // Expectation over dice outcomes
                self.roll_expectation(state, color, ctx)
            }
            Action::BuyDevelopmentCard { color } => {
                // Expectation over dev card identities based on remaining bank composition
                let counts = state.get_remaining_dev_counts();
                let total: u32 = counts.iter().map(|&c| c as u32).sum();
                if total == 0 {
                    let mut next_state = state.clone();
                    next_state.apply_action(action);
                    return self.minimax(
                        &next_state,
                        ctx.depth - 1,
                        ctx.alpha,
                        ctx.beta,
                        ctx.my_color,
                        ctx.deadline,
                    );
                }
                let mut expected = 0.0;
                for (card_idx, &cnt) in counts.iter().enumerate() {
                    if cnt == 0 {
                        continue;
                    }
                    let p = (cnt as f64) / (total as f64);
                    let mut next_state = state.clone();
                    // Simulate the outcome for this specific card type deterministically
                    next_state.simulate_buy_dev_card_outcome(color, card_idx);
                    let v = self.minimax(
                        &next_state,
                        ctx.depth - 1,
                        ctx.alpha,
                        ctx.beta,
                        ctx.my_color,
                        ctx.deadline,
                    );
                    expected += p * v;
                    if let Some(dl) = ctx.deadline {
                        if Instant::now() >= dl {
                            break;
                        }
                    }
                }
                expected
            }
            Action::MoveRobber {
                color,
                coordinate,
                victim_opt: Some(victim),
            } => {
                // Expect over stolen resource based on victim hand composition
                let victim_hand = state.get_player_hand(victim);
                let total_cards: u8 = victim_hand.iter().copied().sum();
                if total_cards == 0 {
                    // No steal; deterministic move robber only
                    let mut next_state = state.clone();
                    let tile_id = next_state
                        .get_map_instance()
                        .get_land_tile(coordinate)
                        .expect("valid robber coordinate")
                        .id;
                    next_state.set_robber_tile(tile_id);
                    next_state.clear_is_moving_robber();
                    return self.minimax(
                        &next_state,
                        ctx.depth - 1,
                        ctx.alpha,
                        ctx.beta,
                        ctx.my_color,
                        ctx.deadline,
                    );
                }

                let mut expected = 0.0;
                for (res_idx, &count) in victim_hand.iter().enumerate() {
                    if count == 0 {
                        continue;
                    }
                    let p = (count as f64) / (total_cards as f64);
                    let mut next_state = state.clone();
                    // Move robber to tile
                    let tile_id = next_state
                        .get_map_instance()
                        .get_land_tile(coordinate)
                        .expect("valid robber coordinate")
                        .id;
                    next_state.set_robber_tile(tile_id);
                    // Transfer one resource from victim to mover
                    next_state.from_player_to_player(victim, color, res_idx as u8, 1);
                    next_state.clear_is_moving_robber();
                    let v = self.minimax(
                        &next_state,
                        ctx.depth - 1,
                        ctx.alpha,
                        ctx.beta,
                        ctx.my_color,
                        ctx.deadline,
                    );
                    expected += p * v;
                    if let Some(dl) = ctx.deadline {
                        if Instant::now() >= dl {
                            break;
                        }
                    }
                }
                expected
            }
            _ => {
                let mut next_state = state.clone();
                next_state.apply_action(action);
                self.minimax(
                    &next_state,
                    ctx.depth - 1,
                    ctx.alpha,
                    ctx.beta,
                    ctx.my_color,
                    ctx.deadline,
                )
            }
        }
    }

    /// Expectiminimax handling for rolling dice: average over dice outcomes
    fn roll_expectation(&self, state: &State, color_to_roll: u8, ctx: &SearchCtx) -> f64 {
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
            if let Some(dl) = ctx.deadline {
                if Instant::now() >= dl {
                    break;
                }
            }
            let mut next_state = state.clone();
            next_state.apply_action(Action::Roll {
                color: color_to_roll,
                dice_opt: Some(pair),
            });
            // After rolling, it is still the same player's turn; do not negate here
            let v = self.minimax(
                &next_state,
                ctx.depth - 1,
                ctx.alpha,
                ctx.beta,
                ctx.my_color,
                ctx.deadline,
            );
            expected += prob * v;
        }
        expected
    }

    /// Alpha-Beta minimax (explicit max/min) with simple beam-ordered moves
    fn minimax(
        &self,
        state: &State,
        depth: i32,
        mut alpha: f64,
        mut beta: f64,
        my_color: u8,
        deadline: Option<std::time::Instant>,
    ) -> f64 {
        // Transposition lookup
        let state_hash = state.compute_hash64();
        let mut tt_move: Option<Action> = None;
        if let Some(&(v, flag, best_mv)) = self.tt.borrow().get(&(state_hash, depth)) {
            tt_move = best_mv;
            match flag {
                0 => return v, // exact
                -1 => {
                    if v <= alpha {
                        return v;
                    }
                } // alpha bound
                1 => {
                    if v >= beta {
                        return v;
                    }
                } // beta bound
                _ => {}
            }
        }
        // Base case: terminal depth or game over
        if depth == 0 || state.winner().is_some() {
            // Quiescence-lite: if tactical moves available, evaluate best 1-ply tactical
            if depth == 0 && state.winner().is_none() {
                return self.quiescence_eval(state, my_color);
            }
            return self.evaluate_relative(state, my_color);
        }
        if let Some(dl) = deadline {
            if std::time::Instant::now() >= dl {
                return self.evaluate_relative(state, my_color);
            }
        }

        let actions = state.generate_playable_actions();
        if actions.is_empty() {
            return self.evaluate_relative(state, my_color);
        }

        // Prune then order actions to improve pruning
        let pruned_actions = self.prune_actions(state, &actions);
        if pruned_actions.is_empty() {
            return self.evaluate_relative(state, my_color);
        }
        let mut ordered_actions = self.order_actions(state, &pruned_actions, depth);
        // Move-count pruning at shallow depths: keep only top-N quiet moves beyond a cap
        if depth <= 2 && ordered_actions.len() > 10 {
            ordered_actions.truncate(10);
        }
        // Prefer TT-best move
        if let Some(mv) = tt_move {
            if let Some(pos) = ordered_actions.iter().position(|&a| a == mv) {
                if pos != 0 {
                    let mv2 = ordered_actions.remove(pos);
                    ordered_actions.insert(0, mv2);
                }
            }
        }

        let is_maximizing = state.get_current_color() == my_color;
        let alpha_orig = alpha;
        if is_maximizing {
            let mut best_value = f64::NEG_INFINITY;
            let mut best_mv: Option<Action> = None;
            for (idx, action) in ordered_actions.iter().copied().enumerate() {
                let mut value;
                if idx == 0 {
                    value = self.evaluate_action_with_chance(
                        state,
                        action,
                        &SearchCtx {
                            depth,
                            alpha,
                            beta,
                            my_color,
                            deadline,
                        },
                    );
                } else {
                    // PVS null-window probe with graduated LMR for quiet late moves
                    let probe_beta = (alpha + PVS_EPS).min(beta);
                    let reduce_by = self.get_lmr_reduction(depth, idx, self.is_quiet_move(action));
                    // Futility pruning at shallow depths for quiet moves
                    if depth <= 2 && self.is_quiet_move(action) {
                        let eval = self.evaluate_relative(state, my_color);
                        let margin = if depth == 1 { 100.0 } else { 200.0 };
                        if eval + margin < alpha {
                            continue;
                        }
                    }
                    let probe_depth = (depth - reduce_by).max(1);
                    value = self.evaluate_action_with_chance(
                        state,
                        action,
                        &SearchCtx {
                            depth: probe_depth,
                            alpha,
                            beta: probe_beta,
                            my_color,
                            deadline,
                        },
                    );
                    if value > alpha && value < beta {
                        value = self.evaluate_action_with_chance(
                            state,
                            action,
                            &SearchCtx {
                                depth,
                                alpha,
                                beta,
                                my_color,
                                deadline,
                            },
                        );
                    }
                }
                if value > best_value {
                    best_value = value;
                    best_mv = Some(action);
                }
                if value > alpha {
                    alpha = value;
                }
                if alpha >= beta {
                    self.record_killer(depth, Some(action));
                    self.bump_history(action, depth);
                    // Singular/PV extension: if first move causes cutoff and looks singular, extend principal continuation
                    // (Handled implicitly by TT on subsequent nodes; keep cutoff here)
                    break;
                }
                if let Some(dl) = deadline {
                    if std::time::Instant::now() >= dl {
                        break;
                    }
                }
            }
            let flag = if best_value <= alpha_orig {
                -1
            } else if best_value >= beta {
                1
            } else {
                0
            };
            self.tt
                .borrow_mut()
                .insert((state_hash, depth), (best_value, flag, best_mv));
            best_value
        } else {
            let mut best_value = f64::INFINITY;
            let mut best_mv: Option<Action> = None;
            for (idx, action) in ordered_actions.iter().copied().enumerate() {
                let mut value;
                if idx == 0 {
                    value = self.evaluate_action_with_chance(
                        state,
                        action,
                        &SearchCtx {
                            depth,
                            alpha,
                            beta,
                            my_color,
                            deadline,
                        },
                    );
                } else {
                    let probe_beta = (alpha + PVS_EPS).min(beta);
                    let reduce_by = self.get_lmr_reduction(depth, idx, self.is_quiet_move(action));
                    // Futility pruning at shallow depths for quiet moves
                    if depth <= 2 && self.is_quiet_move(action) {
                        let eval = self.evaluate_relative(state, my_color);
                        let margin = if depth == 1 { 100.0 } else { 200.0 };
                        if eval + margin < alpha {
                            continue;
                        }
                    }
                    let probe_depth = (depth - reduce_by).max(1);
                    value = self.evaluate_action_with_chance(
                        state,
                        action,
                        &SearchCtx {
                            depth: probe_depth,
                            alpha,
                            beta: probe_beta,
                            my_color,
                            deadline,
                        },
                    );
                    if value > alpha && value < beta {
                        value = self.evaluate_action_with_chance(
                            state,
                            action,
                            &SearchCtx {
                                depth,
                                alpha,
                                beta,
                                my_color,
                                deadline,
                            },
                        );
                    }
                }
                if value < best_value {
                    best_value = value;
                    best_mv = Some(action);
                }
                if value < beta {
                    beta = value;
                }
                if beta <= alpha {
                    self.record_killer(depth, Some(action));
                    self.bump_history(action, depth);
                    break;
                }
                if let Some(dl) = deadline {
                    if std::time::Instant::now() >= dl {
                        break;
                    }
                }
            }
            let flag = if best_value <= alpha_orig {
                -1
            } else if best_value >= beta {
                1
            } else {
                0
            };
            self.tt
                .borrow_mut()
                .insert((state_hash, depth), (best_value, flag, best_mv));
            best_value
        }
    }

    fn record_killer(&self, depth: i32, action: Option<Action>) {
        if action.is_none() {
            return;
        }
        let mut killers = self.killer_moves.borrow_mut();
        let entry = killers.entry(depth).or_insert((None, None));
        if entry.0 != action {
            entry.1 = entry.0;
            entry.0 = action;
        }
    }

    fn bump_history(&self, action: Action, depth: i32) {
        let mut hist = self.history_scores.borrow_mut();
        let e = hist.entry(action).or_insert(0);
        *e += (depth as i64).max(1);
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

        // Clear caches to prevent cross-game or cross-position pollution
        self.node_production_cache.borrow_mut().clear();

        // Optional epsilon-greedy exploration at root
        if let Some(eps) = self.epsilon {
            let mut rng = rand::thread_rng();
            if rng.gen_range(0.0..1.0) < eps {
                let idx = rng.gen_range(0..playable_actions.len());
                log::set_max_level(prev_level);
                return playable_actions[idx];
            }
        }

        let ms = if playable_actions.len() >= self.time_profile.slow_branch_threshold {
            self.time_profile.slow_ms
        } else {
            self.time_profile.fast_ms
        };
        let deadline = Instant::now() + std::time::Duration::from_millis(ms);

        let mut best_action = playable_actions[0];
        let mut best_value = f64::NEG_INFINITY;
        // removed unused `best_candidates`

        // Iterative deepening from 1..=depth or until time runs out
        let mut stable_iterations = 0;
        let mut last_best = best_action;
        for current_depth in 1..=self.depth {
            // Prune then order root actions
            let root_pruned = self.prune_actions(state, playable_actions);
            let mut ordered = self.order_actions(state, &root_pruned, current_depth);
            // Root-only wider beam
            if ordered.len() > ROOT_MAX_ORDERED_ACTIONS {
                ordered.truncate(ROOT_MAX_ORDERED_ACTIONS);
            }

            let mut round_best_action = best_action;
            let mut round_best_value = best_value;
            let mut round_candidates: Vec<Action> = Vec::new();

            for action in ordered {
                if Instant::now() >= deadline {
                    break;
                }
                let mut new_state = state.clone();
                new_state.apply_action(action);
                // Fixed aspiration window; single search using configured width
                let (a, b) = if best_value.is_finite() && current_depth > 1 {
                    (
                        best_value - ASPIRATION_MIN_WINDOW,
                        best_value + ASPIRATION_MIN_WINDOW,
                    )
                } else {
                    (f64::NEG_INFINITY, f64::INFINITY)
                };
                let value = self.minimax(
                    &new_state,
                    current_depth - 1,
                    a,
                    b,
                    my_color,
                    Some(deadline),
                );
                let tol = 1e-6;
                if value > round_best_value + tol {
                    round_best_value = value;
                    round_best_action = action;
                    round_candidates.clear();
                    round_candidates.push(action);
                } else if (value - round_best_value).abs() <= tol {
                    round_candidates.push(action);
                }
            }

            // If we improved within this iteration, keep it
            if round_best_value > best_value {
                best_value = round_best_value;
                // Tie-break randomly if several candidates within tolerance
                if !round_candidates.is_empty() {
                    let mut rng = rand::thread_rng();
                    let idx = rng.gen_range(0..round_candidates.len());
                    best_action = round_candidates[idx];
                } else {
                    best_action = round_best_action;
                }
            }

            if Instant::now() >= deadline {
                break;
            }

            // Early stop if best move is stable
            if best_action == last_best {
                stable_iterations += 1;
                if stable_iterations >= 2 && current_depth >= 3 {
                    break;
                }
            } else {
                stable_iterations = 0;
                last_best = best_action;
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

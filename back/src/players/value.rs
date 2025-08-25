use rand::Rng;

use crate::enums::{Action, DevCard};
use crate::map_instance::NodeId;
use crate::state::{Building, State};

use super::BotPlayer;

const TRANSLATE_VARIETY: f64 = 4.0; // each new resource is like 4 production points
const PROBA_POINT: f64 = 2.778 / 100.0; // probability point used in Python value_production

#[derive(Debug, Clone)]
pub struct ValueWeights {
    pub public_vps: f64,
    pub production: f64,
    pub enemy_production: f64,
    pub num_tiles: f64,
    pub reachable_production_0: f64,
    pub reachable_production_1: f64,
    pub buildable_nodes: f64,
    pub longest_road: f64,
    pub hand_synergy: f64,
    pub hand_resources: f64,
    pub discard_penalty: f64,
    pub hand_devs: f64,
    pub army_size: f64,
}

impl Default for ValueWeights {
    fn default() -> Self {
        Self {
            public_vps: 100.0,
            production: 10.0,
            enemy_production: -5.0,
            num_tiles: 1.0,
            reachable_production_0: 0.0,
            reachable_production_1: 2.0,
            buildable_nodes: 1.0,
            longest_road: 3.0,
            hand_synergy: 2.0,
            hand_resources: 0.5,
            discard_penalty: -5.0,
            hand_devs: 1.0,
            army_size: 5.0,
        }
    }
}

impl ValueWeights {
    pub fn contender() -> Self {
        Self {
            public_vps: 120.0,
            production: 9.0,
            enemy_production: -4.5,
            num_tiles: 1.5,
            reachable_production_0: 0.0,
            reachable_production_1: 2.5,
            buildable_nodes: 1.2,
            longest_road: 4.0,
            hand_synergy: 2.5,
            hand_resources: 0.6,
            discard_penalty: -5.0,
            hand_devs: 1.2,
            army_size: 6.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValueFunctionPlayer {
    pub id: String,
    pub name: String,
    pub color: String,
    pub my_color: u8,
    pub weights: ValueWeights,
    pub epsilon: Option<f64>,
}

impl ValueFunctionPlayer {
    pub fn new(id: String, name: String, color_name: String, my_color: u8) -> Self {
        Self {
            id,
            name,
            color: color_name,
            my_color,
            weights: ValueWeights::default(),
            epsilon: None,
        }
    }

    pub fn with_weights(
        id: String,
        name: String,
        color_name: String,
        my_color: u8,
        weights: ValueWeights,
    ) -> Self {
        Self {
            id,
            name,
            color: color_name,
            my_color,
            weights,
            epsilon: None,
        }
    }

    fn value_production(&self, production: &[f64], include_variety: bool) -> f64 {
        let sum: f64 = production.iter().copied().sum();
        let variety_count = production.iter().filter(|&&p| p > 0.0).count() as f64;
        let variety_bonus = if include_variety {
            variety_count * TRANSLATE_VARIETY * PROBA_POINT
        } else {
            0.0
        };
        sum + variety_bonus
    }

    fn count_my_owned_tiles(&self, state: &State, color: u8) -> usize {
        let mut tiles = std::collections::HashSet::new();
        let mut add_adjacent = |node_id: NodeId| {
            if let Some(adj) = state.get_map_instance().get_adjacent_tiles(node_id) {
                for t in adj.iter() {
                    tiles.insert(t.id);
                }
            }
        };

        for b in state.get_settlements(color) {
            if let Building::Settlement(_, node) = b {
                add_adjacent(node);
            }
        }
        for b in state.get_cities(color) {
            if let Building::City(_, node) = b {
                add_adjacent(node);
            }
        }
        tiles.len()
    }

    fn hand_synergy(&self, state: &State, color: u8) -> f64 {
        // Estimate distance to city and settlement based on hand counts
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

    fn evaluate_state(&self, state: &State, p0_color: u8) -> f64 {
        let w = &self.weights;

        // Public/actual VPs
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

        // Reachable production at 0 and 1 roads: placeholders (0 for now)
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
        let army_size = state.get_played_dev_card_count(p0_color, DevCard::Knight as usize) as f64;
        let hand_synergy = self.hand_synergy(state, p0_color);

        // Board features
        let num_buildable_nodes = state.buildable_node_ids(p0_color).len() as f64;
        let num_tiles = self.count_my_owned_tiles(state, p0_color) as f64;

        // Longest road factor: if cannot build more, weight longest road bonus; else small
        let longest_road_factor = if num_buildable_nodes == 0.0 {
            w.longest_road
        } else {
            0.1
        };
        let longest_road_length = 0.0; // TODO: add getter or compute per components

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
}

impl BotPlayer for ValueFunctionPlayer {
    fn decide(&self, state: &State, playable_actions: &[Action]) -> Action {
        if playable_actions.len() == 1 {
            return playable_actions[0];
        }

        if let Some(eps) = self.epsilon {
            let mut rng = rand::thread_rng();
            if rng.gen_range(0.0..1.0) < eps {
                let idx = rng.gen_range(0..playable_actions.len());
                return playable_actions[idx];
            }
        }

        let mut best_action = playable_actions[0];
        let mut best_value = f64::NEG_INFINITY;
        for &action in playable_actions.iter() {
            let mut next_state = state.clone();
            next_state.apply_action(action);
            let value = self.evaluate_state(&next_state, self.my_color);
            if value > best_value {
                best_value = value;
                best_action = action;
            }
        }
        best_action
    }
}

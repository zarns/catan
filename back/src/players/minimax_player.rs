use std::time::{Duration, Instant};
use std::f64;
use rand::Rng;

use super::Player;
use crate::enums::Action;
use crate::state::State;

const DEFAULT_DEPTH: usize = 3;
const MAX_SEARCH_TIME: u64 = 10; // 10 seconds max

/// The AlphaBetaPlayer uses a minimax search with alpha-beta pruning
/// to find the optimal move in a game state. It handles probabilistic
/// outcomes (like dice rolls) by calculating expected values.
pub struct AlphaBetaPlayer {
    depth: usize,
    pruning: bool,
    epsilon: Option<f64>, // For optional exploration
}

/// Represents a state node in the search tree, used for debugging
struct StateNode {
    state: State,
    expected_value: Option<f64>,
    children: Vec<ActionNode>,
}

/// Represents an action node in the search tree, used for debugging
struct ActionNode {
    action: Action,
    expected_value: Option<f64>,
    outcomes: Vec<(StateNode, f64)>, // (node, probability)
}

/// A structure representing a weighted outcome from an action
struct Outcome {
    state: State,
    probability: f64,
}

impl AlphaBetaPlayer {
    /// Create a new AlphaBetaPlayer with default parameters
    pub fn new() -> Self {
        AlphaBetaPlayer {
            depth: DEFAULT_DEPTH,
            pruning: true,  // Enable pruning by default
            epsilon: None,
        }
    }
    
    /// Create a new AlphaBetaPlayer with custom parameters
    pub fn with_params(depth: usize, pruning: bool, epsilon: Option<f64>) -> Self {
        AlphaBetaPlayer {
            depth,
            pruning,
            epsilon,
        }
    }
    
    /// Expand a game state into all possible outcomes with probabilities
    fn expand_spectrum(&self, state: &State, action: &Action) -> Vec<Outcome> {
        let mut outcomes = Vec::new();
        let mut state_copy = state.clone();
        
        // Apply the action
        state_copy.apply_action(action.clone());
        
        // Check if this action results in a dice roll (for now, simplify and just return the state)
        // In a full implementation, we would generate all possible dice outcomes with probabilities
        outcomes.push(Outcome {
            state: state_copy,
            probability: 1.0,
        });
        
        outcomes
    }
    
    /// Count the number of roads owned by a player
    fn count_player_roads(&self, state: &State, color: u8) -> usize {
        // Since roads_by_color is private, we can't use it directly.
        // Instead, we'll just count the number of development cards and 
        // resource cards to approximate player strength
        let resources = state.get_player_hand(color);
        let resource_count = resources.iter().sum::<u8>() as usize;
        
        // Very rough approximation, not ideal but will work until we implement
        // a better solution or expose a method in State to count roads
        resource_count / 2
    }
    
    /// Count the number of settlements owned by a player
    fn count_player_settlements(&self, state: &State, color: u8) -> usize {
        let settlements = state.get_settlements(color);
        settlements.len()
    }
    
    /// Count the number of cities owned by a player
    fn count_player_cities(&self, state: &State, color: u8) -> usize {
        let cities = state.get_cities(color);
        cities.len()
    }
    
    /// Evaluate a game state's value for the current player
    fn evaluate_state(&self, state: &State, my_color: u8) -> f64 {
        // Constants for weighting different factors (based on Python's DEFAULT_WEIGHTS)
        const VP_WEIGHT: f64 = 3.0e14;           // public_vps
        const PRODUCTION_WEIGHT: f64 = 1.0e8;    // production
        const ENEMY_PRODUCTION_WEIGHT: f64 = -1.0e8; // enemy_production 
        const NUM_TILES_WEIGHT: f64 = 1.0;       // num_tiles
        const BUILDABLE_NODES_WEIGHT: f64 = 1.0e3; // buildable_nodes
        const LONGEST_ROAD_WEIGHT: f64 = 10.0;   // longest_road
        const HAND_RESOURCES_WEIGHT: f64 = 1.0;  // hand_resources
        const DISCARD_PENALTY: f64 = -5.0;       // discard_penalty
        const HAND_DEVS_WEIGHT: f64 = 10.0;      // hand_devs
        const ARMY_SIZE_WEIGHT: f64 = 10.1;      // army_size
        
        // Victory points (highest weight as winning is most important)
        let vp_value = state.get_actual_victory_points(my_color) as f64 * VP_WEIGHT;
        
        // Resource production (approximate based on owned settlements/cities)
        let settlements = state.get_settlements(my_color);
        let cities = state.get_cities(my_color);
        let production_value = (settlements.len() as f64 + 2.0 * cities.len() as f64) * PRODUCTION_WEIGHT;
        
        // Enemy production penalty (estimate based on other players' buildings)
        let mut enemy_production = 0.0;
        for color in 0..4 {
            if color != my_color {
                let enemy_settlements = state.get_settlements(color).len() as f64;
                let enemy_cities = state.get_cities(color).len() as f64;
                enemy_production += enemy_settlements + 2.0 * enemy_cities;
            }
        }
        let enemy_production_value = enemy_production * ENEMY_PRODUCTION_WEIGHT;
        
        // Resource hand value
        let resources = state.get_player_hand(my_color);
        let resource_count = resources.iter().sum::<u8>() as f64;
        let resource_value = resource_count * HAND_RESOURCES_WEIGHT;
        
        // Apply discard penalty if holding more than 7 cards
        let discard_value = if resource_count > 7.0 { DISCARD_PENALTY } else { 0.0 };
        
        // Development cards value
        let dev_cards = state.get_player_devhand(my_color);
        let dev_card_count = dev_cards.iter().sum::<u8>() as f64;
        let dev_card_value = dev_card_count * HAND_DEVS_WEIGHT;
        
        // Knight cards value (approximated based on total dev cards)
        let army_value = dev_cards[0] as f64 * ARMY_SIZE_WEIGHT; // Index 0 is often knights
        
        // Building value calculation - approximated by the number of legal actions
        // Since we don't have direct access to buildable nodes, use playable actions as proxy
        let actions = state.generate_playable_actions();
        let build_actions = actions.iter().filter(|&a| matches!(a, 
            Action::BuildRoad { .. } | 
            Action::BuildSettlement { .. } | 
            Action::BuildCity { .. }
        )).count();
        let buildable_nodes_value = build_actions as f64 * BUILDABLE_NODES_WEIGHT;
        
        // Calculate number of unique tiles player has access to (for resource variety)
        // Since we can't use HashSet with buildings, simply count settlements + cities
        // and multiply by a factor to represent average unique tiles per building
        let avg_tiles_per_building = 2.5; // Each building gives access to ~2-3 tiles on average
        let num_tiles_value = (settlements.len() + cities.len()) as f64 * avg_tiles_per_building * NUM_TILES_WEIGHT;
        
        // Longest road value - approximated by number of settlements
        // This isn't accurate, but settlements are somewhat correlated with road length
        let longest_road_approx = settlements.len().max(cities.len()) as f64;
        let longest_road_value = longest_road_approx * LONGEST_ROAD_WEIGHT;
        
        // Sum all components with their weights
        vp_value + 
        production_value + 
        enemy_production_value + 
        resource_value + 
        discard_value + 
        dev_card_value + 
        army_value + 
        buildable_nodes_value + 
        num_tiles_value + 
        longest_road_value
    }
    
    /// Optionally prune the list of actions to consider
    fn prune_actions(&self, state: &State, my_color: u8) -> Vec<Action> {
        if !self.pruning {
            return state.generate_playable_actions();
        }
        
        // Prioritize actions based on game strategy
        let all_actions = state.generate_playable_actions();
        let mut building_actions = Vec::new();
        let mut dev_card_actions = Vec::new();
        let mut trade_actions = Vec::new();
        let mut other_actions = Vec::new();
        
        for action in all_actions {
            match action {
                // Building actions are highest priority - they directly contribute to victory points
                Action::BuildSettlement { .. } | 
                Action::BuildCity { .. } => building_actions.push(action),
                
                // Road building is important for expansion
                Action::BuildRoad { .. } => building_actions.push(action),
                
                // Development cards can be powerful
                Action::BuyDevelopmentCard { .. } => dev_card_actions.push(action),
                
                // Trading actions come next
                Action::MaritimeTrade { .. } |
                Action::Discard { .. } => trade_actions.push(action),
                
                // All other actions
                _ => other_actions.push(action),
            }
        }
        
        // Calculate the max actions to consider based on game phase
        let max_actions = if state.is_initial_build_phase() {
            // During initial placement, consider all options
            building_actions.len() + dev_card_actions.len() + trade_actions.len() + other_actions.len()
        } else {
            // In normal gameplay, limit the number of actions to analyze
            20
        };
        
        // Create a prioritized list of actions
        let mut prioritized_actions = Vec::new();
        prioritized_actions.extend(building_actions);
        prioritized_actions.extend(dev_card_actions);
        prioritized_actions.extend(trade_actions);
        prioritized_actions.extend(other_actions);
        
        // If we have too many actions, keep the prioritized ones up to max_actions
        if prioritized_actions.len() > max_actions {
            prioritized_actions.truncate(max_actions);
        }
        
        prioritized_actions
    }
    
    /// The core alpha-beta minimax algorithm
    fn alphabeta(
        &self, 
        state: &State, 
        depth: usize, 
        mut alpha: f64, 
        mut beta: f64, 
        deadline: Instant,
        my_color: u8
    ) -> (Option<Action>, f64) {
        // Terminal conditions
        if depth == 0 || state.winner().is_some() || Instant::now() > deadline {
            return (None, self.evaluate_state(state, my_color));
        }
        
        let current_color = state.get_current_color();
        let maximizing_player = current_color == my_color;
        let actions = self.prune_actions(state, my_color);
        
        if actions.is_empty() {
            return (None, self.evaluate_state(state, my_color));
        }
        
        if maximizing_player {
            let mut best_action = None;
            let mut best_value = f64::NEG_INFINITY;
            
            for action in &actions {
                let outcomes = self.expand_spectrum(state, action);
                let mut expected_value = 0.0;
                
                for outcome in outcomes {
                    let (_, value) = self.alphabeta(
                        &outcome.state,
                        depth - 1,
                        alpha,
                        beta,
                        deadline,
                        my_color
                    );
                    
                    expected_value += outcome.probability * value;
                }
                
                if expected_value > best_value {
                    best_value = expected_value;
                    best_action = Some(action.clone());
                }
                
                alpha = alpha.max(best_value);
                if alpha >= beta {
                    break; // Beta cutoff
                }
            }
            
            (best_action, best_value)
        } else {
            let mut best_action = None;
            let mut best_value = f64::INFINITY;
            
            for action in &actions {
                let outcomes = self.expand_spectrum(state, action);
                let mut expected_value = 0.0;
                
                for outcome in outcomes {
                    let (_, value) = self.alphabeta(
                        &outcome.state,
                        depth - 1,
                        alpha,
                        beta,
                        deadline,
                        my_color
                    );
                    
                    expected_value += outcome.probability * value;
                }
                
                if expected_value < best_value {
                    best_value = expected_value;
                    best_action = Some(action.clone());
                }
                
                beta = beta.min(best_value);
                if beta <= alpha {
                    break; // Alpha cutoff
                }
            }
            
            (best_action, best_value)
        }
    }
}

impl Player for AlphaBetaPlayer {
    fn decide(&self, state: &State, playable_actions: &[Action]) -> Action {
        if playable_actions.len() == 1 {
            return playable_actions[0].clone();
        }
        
        // Epsilon-greedy exploration
        if let Some(eps) = self.epsilon {
            let mut rng = rand::thread_rng();
            if rng.gen::<f64>() < eps {
                let idx = rng.gen_range(0..playable_actions.len());
                return playable_actions[idx].clone();
            }
        }
        
        let start = Instant::now();
        let deadline = start + Duration::from_secs(MAX_SEARCH_TIME);
        let my_color = state.get_current_color();
        
        // Run alpha-beta search
        let result = self.alphabeta(
            state,
            self.depth,
            f64::NEG_INFINITY,
            f64::INFINITY,
            deadline,
            my_color
        );
        
        // Log the decision time
        let duration = start.elapsed();
        println!(
            "AlphaBeta took {:?} to make a decision with depth {}",
            duration,
            self.depth
        );
        
        // Return the best action or default to the first available action
        match result.0 {
            Some(action) => action,
            None => playable_actions[0].clone(),
        }
    }
}

impl Default for AlphaBetaPlayer {
    fn default() -> Self {
        Self::new()
    }
}

use std::time::Instant;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::Rng;
use std::f64;
use std::sync::{Arc, Mutex};
use rayon::prelude::*;

use super::Player;
use crate::enums::Action;
use crate::state::State;

const MCTS_SIMULATIONS: usize = 100;
const EXPLORATION_CONSTANT: f64 = 1.41; // sqrt(2)

/// Node in the MCTS tree
struct MctsNode {
    state: State,
    action: Option<Action>,        // Action that led to this state (None for root)
    parent: Option<usize>,         // Index of parent node in the nodes vector
    children: Vec<usize>,          // Indices of child nodes in the nodes vector
    visits: usize,                 // Number of times this node was visited
    wins: usize,                   // Number of wins from this node
    untried_actions: Vec<Action>,  // Actions not yet expanded into children
}

impl MctsNode {
    fn new(state: State, action: Option<Action>, parent: Option<usize>) -> Self {
        let untried_actions = state.generate_playable_actions();
        Self {
            state,
            action,
            parent,
            children: Vec::new(),
            visits: 0,
            wins: 0,
            untried_actions,
        }
    }

    fn is_fully_expanded(&self) -> bool {
        self.untried_actions.is_empty()
    }

    fn is_terminal(&self) -> bool {
        self.state.winner().is_some()
    }

    fn uct_value(&self, parent_visits: usize, exploration: f64) -> f64 {
        if self.visits == 0 {
            return f64::INFINITY;
        }
        
        let exploitation = self.wins as f64 / self.visits as f64;
        let exploration_term = exploration * ((parent_visits as f64).ln() / self.visits as f64).sqrt();
        
        exploitation + exploration_term
    }
}

/// Monte Carlo Tree Search Player
/// Implements the full MCTS algorithm with tree building and UCT selection
pub struct MctsPlayer {
    num_simulations: usize,
    exploration_constant: f64,
    use_parallel: bool,
}

impl MctsPlayer {
    pub fn new() -> Self {
        MctsPlayer {
            num_simulations: MCTS_SIMULATIONS,
            exploration_constant: EXPLORATION_CONSTANT,
            use_parallel: true,
        }
    }

    pub fn with_parameters(num_simulations: usize, exploration_constant: f64) -> Self {
        MctsPlayer {
            num_simulations,
            exploration_constant,
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
            let action = actions.choose(&mut rng).unwrap().clone();
            state.apply_action(action);
        }

        state.winner()
    }

    /// Selects a node for expansion using UCT
    fn select_node(&self, nodes: &[MctsNode], node_index: usize) -> usize {
        let current = &nodes[node_index];
        
        // If this node isn't fully expanded, return it
        if !current.is_fully_expanded() {
            return node_index;
        }
        
        // If this is a terminal node, return it
        if current.is_terminal() {
            return node_index;
        }
        
        // Find the child with the highest UCT value
        let parent_visits = current.visits;
        
        let mut best_child_index = 0;
        let mut best_value = f64::NEG_INFINITY;
        
        for &child_index in &current.children {
            let child = &nodes[child_index];
            let uct_value = child.uct_value(parent_visits, self.exploration_constant);
            
            if uct_value > best_value {
                best_value = uct_value;
                best_child_index = child_index;
            }
        }
        
        // Recursively select from the best child
        self.select_node(nodes, best_child_index)
    }

    /// Expand the node by adding a new child node for an untried action
    fn expand(&self, nodes: &mut Vec<MctsNode>, node_index: usize) -> usize {
        let mut rng = thread_rng();
        
        // Get an untried action
        let (node_action, node_state) = {
            let node = &mut nodes[node_index];
            if node.untried_actions.is_empty() {
                return node_index; // Cannot expand further
            }
            
            let action_index = rng.gen_range(0..node.untried_actions.len());
            let action = node.untried_actions.remove(action_index);
            
            // Apply the action to the state
            let mut new_state = node.state.clone();
            new_state.apply_action(action.clone());
            
            (action, new_state)
        };
        
        // Create a new node
        let new_node = MctsNode::new(node_state, Some(node_action), Some(node_index));
        let new_node_index = nodes.len();
        
        // Add the new node to the tree
        nodes.push(new_node);
        nodes[node_index].children.push(new_node_index);
        
        new_node_index
    }

    /// Simulate a random playout from the node
    fn simulate(&self, nodes: &[MctsNode], node_index: usize) -> Option<u8> {
        let state = nodes[node_index].state.clone();
        Self::playout(state)
    }

    /// Backpropagate the result up the tree
    fn backpropagate(&self, nodes: &mut Vec<MctsNode>, node_index: usize, winner: Option<u8>) {
        let mut current_index = Some(node_index);
        
        while let Some(idx) = current_index {
            let node = &mut nodes[idx];
            node.visits += 1;
            
            if let Some(win_color) = winner {
                if win_color == node.state.get_current_color() {
                    node.wins += 1;
                }
            }
            
            current_index = node.parent;
        }
    }

    /// Run a single MCTS simulation
    fn run_single_simulation(&self, nodes: &mut Vec<MctsNode>) {
        // Selection - select a node to expand
        let node_to_expand = self.select_node(nodes, 0);
        
        // Expansion - expand the selected node if possible
        let new_node_index = if nodes[node_to_expand].is_terminal() {
            node_to_expand
        } else {
            self.expand(nodes, node_to_expand)
        };
        
        // Simulation - run a random playout from the new node
        let result = self.simulate(nodes, new_node_index);
        
        // Backpropagation - update nodes with result
        self.backpropagate(nodes, new_node_index, result);
    }

    /// Run the MCTS algorithm and return the best action
    fn run_mcts(&self, state: &State, playable_actions: &[Action]) -> Action {
        let start = Instant::now();
        
        if self.use_parallel {
            self.run_mcts_parallel(state, playable_actions)
        } else {
            self.run_mcts_sequential(state, playable_actions)
        }
    }
    
    /// Run MCTS sequentially (original implementation)
    fn run_mcts_sequential(&self, state: &State, playable_actions: &[Action]) -> Action {
        let start = Instant::now();
        
        // Create root node
        let mut nodes = Vec::new();
        nodes.push(MctsNode::new(state.clone(), None, None));
        
        // Run simulations
        for _ in 0..self.num_simulations {
            self.run_single_simulation(&mut nodes);
        }
        
        // Choose the best action based on most visits
        let root = &nodes[0];
        let mut best_action = playable_actions[0].clone();
        let mut best_visits = 0;
        
        for &child_index in &root.children {
            let child = &nodes[child_index];
            
            if child.visits > best_visits {
                best_visits = child.visits;
                if let Some(action) = &child.action {
                    best_action = action.clone();
                }
            }
        }
        
        let duration = start.elapsed();
        log::debug!(
            "MCTS took {:?} to make a decision with {} simulations (sequential)",
            duration,
            self.num_simulations
        );
        
        best_action
    }
    
    /// Run MCTS with parallel simulations
    fn run_mcts_parallel(&self, state: &State, playable_actions: &[Action]) -> Action {
        let start = Instant::now();
        
        // Initialize the tree with root node
        let root_state = state.clone();
        
        // Phase 1: Create initial expansion for each action
        let initial_results: Vec<(Action, usize, usize)> = playable_actions
            .par_iter()
            .map(|action| {
                let mut action_state = root_state.clone();
                action_state.apply_action(action.clone());
                
                // Run playouts for this action
                let simulations_per_action = self.num_simulations / playable_actions.len();
                let mut wins = 0;
                let my_color = root_state.get_current_color();
                
                for _ in 0..simulations_per_action {
                    let mut sim_state = action_state.clone();
                    if let Some(winner) = Self::playout(sim_state) {
                        if winner == my_color {
                            wins += 1;
                        }
                    }
                }
                
                (action.clone(), wins, simulations_per_action)
            })
            .collect();
        
        // Find the action with the highest win ratio
        let mut best_action = playable_actions[0].clone();
        let mut best_win_ratio = 0.0;
        
        for (action, wins, sims) in initial_results {
            let win_ratio = wins as f64 / sims as f64;
            if win_ratio > best_win_ratio {
                best_win_ratio = win_ratio;
                best_action = action;
            }
        }
        
        let duration = start.elapsed();
        log::debug!(
            "MCTS took {:?} to make a decision with ~{} simulations (parallel), win rate: {:.2}%",
            duration,
            self.num_simulations,
            best_win_ratio * 100.0
        );
        
        best_action
    }
}

impl Player for MctsPlayer {
    fn decide(&self, state: &State, playable_actions: &[Action]) -> Action {
        if playable_actions.len() == 1 {
            return playable_actions[0].clone();
        }
        
        self.run_mcts(state, playable_actions)
    }
}

impl Default for MctsPlayer {
    fn default() -> Self {
        Self::new()
    }
}

use rand::Rng;
use std::collections::HashMap;

use super::Player;
use crate::enums::{Action, ActionPrompt};
use crate::state::State;

/// Player that decides randomly but gives preference to certain actions.
/// This player assigns higher weights to actions that are generally valuable:
/// - Building cities
/// - Building settlements
/// - Buying development cards
/// Other actions have a default weight of 1.
pub struct WeightedRandomPlayer {}

impl WeightedRandomPlayer {
    /// Creates action weight map similar to Python version
    fn get_action_weights() -> HashMap<&'static str, usize> {
        let mut weights = HashMap::new();
        weights.insert("BuildCity", 10000);
        weights.insert("BuildSettlement", 1000);
        weights.insert("BuyDevelopmentCard", 100);
        weights
    }
}

impl Player for WeightedRandomPlayer {
    fn decide(&self, _state: &State, playable_actions: &[Action]) -> Action {
        if playable_actions.len() == 1 {
            return playable_actions[0].clone();
        }

        let weights = Self::get_action_weights();
        let mut rng = rand::thread_rng();

        // Create a weighted list of actions
        let mut weighted_actions = Vec::new();

        for action in playable_actions {
            // Determine action type string from action enum variant
            let action_type = match action {
                Action::BuildCity { .. } => "BuildCity",
                Action::BuildSettlement { .. } => "BuildSettlement",
                Action::BuyDevelopmentCard { .. } => "BuyDevelopmentCard",
                _ => "Other",
            };

            // Get weight for this action type (default to 1 if not specified)
            let weight = *weights.get(action_type).unwrap_or(&1);

            // Add this action to the list 'weight' times
            for _ in 0..weight {
                weighted_actions.push(action.clone());
            }
        }

        // If no actions were added (shouldn't happen), return the first action
        if weighted_actions.is_empty() {
            return playable_actions[0].clone();
        }

        // Choose a random action from the weighted list
        let index = rng.gen_range(0..weighted_actions.len());
        weighted_actions[index].clone()
    }
}

impl Default for WeightedRandomPlayer {
    fn default() -> Self {
        Self {}
    }
}

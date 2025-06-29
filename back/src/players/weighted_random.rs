use rand::prelude::*;
use std::collections::HashMap;

use crate::enums::Action;
use crate::state::State;

use super::BotPlayer;

/// Player that decides randomly but gives preference to certain actions.
/// This player assigns higher weights to actions that are generally valuable:
/// - Building cities
/// - Building settlements
/// - Buying development cards
/// Other actions have a default weight of 1.
pub struct WeightedRandomPlayer {
    pub id: String,
    pub name: String,
    pub color: String,
}

impl WeightedRandomPlayer {
    pub fn new(id: String, name: String, color: String) -> Self {
        WeightedRandomPlayer { id, name, color }
    }

    /// Creates action weight map similar to Python version
    fn get_action_weights() -> HashMap<&'static str, u32> {
        let mut weights = HashMap::new();
        weights.insert("BuildCity", 10); // High priority for victory points
        weights.insert("BuildSettlement", 8); // High priority for victory points
        weights.insert("BuyDevelopmentCard", 5); // Medium priority
        weights.insert("Other", 1); // Low priority for other actions
        weights
    }
}

impl BotPlayer for WeightedRandomPlayer {
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
        Self::new(
            "default".to_string(),
            "Weighted Random Player".to_string(),
            "red".to_string(),
        )
    }
}

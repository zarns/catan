use std::collections::{HashMap, HashSet};

use rand::Rng;

// Import from parent module's imports
use super::Building;
use super::State;

// Import directly from lib scope
use crate::deck_slices::{
    freqdeck_add, freqdeck_sub, CITY_COST, DEVCARD_COST, ROAD_COST, SETTLEMENT_COST,
};

// Other imports
use crate::enums::{Action, DevCard};
use crate::map_instance::{EdgeId, NodeId};
use crate::state_vector::*;

impl State {
    pub fn apply_action(&mut self, action: Action) {
        let before_initial = self.is_initial_build_phase();
        let before_settlements = self
            .buildings_by_color
            .values()
            .map(|buildings| {
                buildings
                    .iter()
                    .filter(|b| matches!(b, Building::Settlement(_, _)))
                    .count()
            })
            .sum::<usize>();
        let before_roads = self.roads.len() / 2;

        match action {
            Action::BuildSettlement { color, node_id } => {
                let (new_owner, new_length) = self.build_settlement(color, node_id);
                self.maintain_longest_road(new_owner, new_length);
            }
            Action::BuildRoad { color, edge_id } => {
                let (new_owner, new_length) = self.build_road(color, edge_id);
                self.maintain_longest_road(new_owner, new_length);
            }
            Action::BuildCity { color, node_id } => {
                self.build_city(color, node_id);
            }
            Action::BuyDevelopmentCard { color } => {
                self.buy_development_card(color);
            }
            Action::Roll { color, dice_opt } => {
                self.roll_dice(color, dice_opt);
            }
            Action::Discard { color } => {
                self.discard(color);
            }
            Action::MoveRobber {
                color,
                coordinate,
                victim_opt,
            } => {
                self.move_robber(color, coordinate, victim_opt);
            }
            Action::PlayKnight { color } => {
                self.play_knight(color);
                self.maintain_largest_army();
            }
            Action::PlayYearOfPlenty { color, resources } => {
                self.play_year_of_plenty(color, resources);
            }
            Action::PlayMonopoly { color, resource } => {
                self.play_monopoly(color, resource);
            }
            Action::PlayRoadBuilding { color } => {
                self.play_road_building(color);
            }
            Action::MaritimeTrade {
                color,
                give,
                take,
                ratio,
            } => {
                self.maritime_trade(color, give, take, ratio);
            }
            Action::EndTurn { color } => {
                self.end_turn(color);
            }
            _ => {
                panic!("Action not implemented: {:?}", action);
            }
        }

        // Log important state changes
        let after_initial = self.is_initial_build_phase();
        let after_settlements = self
            .buildings_by_color
            .values()
            .map(|buildings| {
                buildings
                    .iter()
                    .filter(|b| matches!(b, Building::Settlement(_, _)))
                    .count()
            })
            .sum::<usize>();
        let after_roads = self.roads.len() / 2;

        // Log phase transitions and significant state changes
        if before_initial != after_initial {
            log::info!(
                "🎯 PHASE TRANSITION: Initial build phase = {} → {}",
                before_initial,
                after_initial
            );
        }

        if before_settlements != after_settlements || before_roads != after_roads {
            log::info!(
                "🏗️  Build progress: Settlements: {} → {}, Roads: {} → {}",
                before_settlements,
                after_settlements,
                before_roads,
                after_roads
            );
        }
    }

    pub fn add_victory_points(&mut self, color: u8, points: u8) {
        let n = self.get_num_players();
        self.vector[actual_victory_points_index(n, color)] += points;
        self.check_for_victory(); // Check for win condition whenever VPs change
    }

    pub fn sub_victory_points(&mut self, color: u8, points: u8) {
        let n = self.get_num_players();
        self.vector[actual_victory_points_index(n, color)] -= points;
        self.check_for_victory(); // Check for win condition whenever VPs change
    }

    pub fn advance_turn(&mut self, step_size: i8) {
        // We add an extra num_players to ensure next_index is positive (u8)
        let num_players = self.get_num_players() as i8;
        let next_index =
            ((self.get_current_tick_seat() as i8 + step_size + num_players) % num_players) as u8;

        self.vector[CURRENT_TICK_SEAT_INDEX] = next_index;
    }

    pub fn build_settlement(&mut self, placing_color: u8, node_id: u8) -> (Option<u8>, u8) {
        self.buildings
            .insert(node_id, Building::Settlement(placing_color, node_id));
        self.buildings_by_color
            .entry(placing_color)
            .or_default()
            .push(Building::Settlement(placing_color, node_id));

        let is_free = self.is_initial_build_phase();
        if !is_free {
            freqdeck_sub(self.get_mut_player_hand(placing_color), SETTLEMENT_COST);
            freqdeck_add(&mut self.vector[BANK_RESOURCE_SLICE], SETTLEMENT_COST);
        }

        self.add_victory_points(placing_color, 1);

        // Update board_buildable_ids cache - remove the settlement node and its neighbors
        self.board_buildable_ids.remove(&node_id);
        for neighbor_id in self.map_instance.get_neighbor_nodes(node_id) {
            self.board_buildable_ids.remove(&neighbor_id);
        }

        let mut road_lengths: HashMap<u8, u8> = HashMap::new();

        if is_free {
            let owned_buildings = self.buildings_by_color.get(&placing_color).unwrap();
            let owned_settlements = owned_buildings
                .iter()
                .filter(|b| matches!(b, Building::Settlement(_, _)))
                .count();

            // If second house, yield resources
            if owned_settlements == 2 {
                let adjacent_tiles = self.map_instance.get_adjacent_tiles(node_id);
                if let Some(adjacent_tiles) = adjacent_tiles {
                    let mut total_resources = [0; 5];
                    for tile in adjacent_tiles {
                        if let Some(resource) = tile.resource {
                            total_resources[resource as usize] += 1;
                        }
                    }

                    let bank = &mut self.vector[BANK_RESOURCE_SLICE];
                    freqdeck_sub(bank, total_resources);

                    let hand = self.get_mut_player_hand(placing_color);
                    freqdeck_add(hand, total_resources);

                    log::info!(
                        "🎁 Player {} received resources from second settlement: {:?}",
                        placing_color,
                        total_resources
                    );
                }
            }
            // Maintain caches and longest road =====
            //   - connected_components
            let component = HashSet::from([node_id]);
            self.connected_components
                .entry(placing_color)
                .or_default()
                .push(component);

            // During initial build phase, preserve existing longest road state
            return (self.longest_road_color, self.longest_road_length);
        } else {
            // Mantain connected_components
            // Mantain longest_road_color and longest_road_length

            let mut plowed_edges_by_color: HashMap<u8, Vec<EdgeId>> = HashMap::new();
            for edge in self.map_instance.get_neighbor_edges(node_id) {
                if let Some(&road_color) = self.roads.get(&edge) {
                    plowed_edges_by_color
                        .entry(road_color)
                        .or_default()
                        .push(edge);
                }
            }

            for (plowed_color, plowed_edges) in plowed_edges_by_color {
                if plowed_edges.len() != 2 || plowed_color == placing_color {
                    continue; // Skip if no bisection/plow
                }

                if let Some(plowed_component_idx) =
                    self.get_connected_component_index(plowed_color, node_id)
                {
                    let outer_nodes: Vec<NodeId> = plowed_edges
                        .iter()
                        .map(|&edge| if edge.0 == node_id { edge.1 } else { edge.0 })
                        .collect();

                    if outer_nodes.len() != 2 {
                        continue; // Can't bisect
                    }

                    // The component will be split
                    let original_component =
                        self.connected_components[&plowed_color][plowed_component_idx].clone();
                    self.connected_components
                        .get_mut(&plowed_color)
                        .unwrap()
                        .remove(plowed_component_idx);

                    // DFS to create new components
                    let component_a = self.dfs_walk(outer_nodes[0], plowed_color);
                    let mut remaining_nodes = original_component;
                    for node in &component_a {
                        remaining_nodes.remove(node);
                    }
                    remaining_nodes.remove(&node_id); // Remove settlement node

                    if !component_a.is_empty() {
                        self.connected_components
                            .get_mut(&plowed_color)
                            .unwrap()
                            .push(component_a);
                    }
                    if !remaining_nodes.is_empty() {
                        self.connected_components
                            .get_mut(&plowed_color)
                            .unwrap()
                            .push(remaining_nodes);
                    }
                }
            }

            // Recalculate all road lengths (sort colors for deterministic order)
            let mut colors: Vec<_> = self.connected_components.keys().cloned().collect();
            colors.sort();
            for color in colors {
                if let Some(components) = self.connected_components.get(&color) {
                    for component in components {
                        let length = self.longest_acyclic_path(component, color).len() as u8;
                        if length > *road_lengths.get(&color).unwrap_or(&0) {
                            road_lengths.insert(color, length);
                        }
                    }
                }
            }
        }

        // Return longest road information
        let new_longest_road_length = road_lengths.values().max().unwrap_or(&0);

        // Determine new longest road owner based on this implementation's rules:
        // 1. Need >= 5 roads to initially claim longest road
        // 2. Once you have it, you keep it unless someone else builds a longer road OR your road becomes < 5
        // 3. If no one currently has it, need >= 5 to claim it
        let new_longest_road_color = if let Some(current_holder) = self.longest_road_color {
            // Someone currently holds longest road
            let current_holder_length = road_lengths.get(&current_holder).unwrap_or(&0);

            // Current holder loses it if their road becomes < 5
            if *current_holder_length < 5 {
                // Check if anyone else has >= 5 to claim it
                if *new_longest_road_length >= 5 {
                    // Sort for deterministic iteration
                    let mut candidates: Vec<_> = road_lengths
                        .iter()
                        .filter(|(_, &length)| length == *new_longest_road_length)
                        .collect();
                    candidates.sort_by_key(|(&color, _)| color);
                    candidates.first().map(|(&color, _)| color)
                } else {
                    None
                }
            } else {
                // Current holder has >= 5, check if anyone else has longer
                let someone_has_longer = road_lengths.iter().any(|(&color, &length)| {
                    color != current_holder && length > *current_holder_length
                });

                if someone_has_longer {
                    // Find who has the longest road now (deterministic)
                    let max_length = road_lengths.values().max().unwrap_or(&0);
                    let mut candidates: Vec<_> = road_lengths
                        .iter()
                        .filter(|(_, &length)| length == *max_length)
                        .collect();
                    candidates.sort_by_key(|(&color, _)| color);
                    candidates.first().map(|(&color, _)| color)
                } else {
                    // Current holder keeps it
                    Some(current_holder)
                }
            }
        } else if *new_longest_road_length >= 5 {
            // No one currently has longest road, but someone has >= 5 (deterministic)
            let mut candidates: Vec<_> = road_lengths
                .iter()
                .filter(|(_, &length)| length == *new_longest_road_length)
                .collect();
            candidates.sort_by_key(|(&color, _)| color);
            candidates.first().map(|(&color, _)| color)
        } else {
            // No one has longest road and no one has >= 5
            None
        };

        (new_longest_road_color, *new_longest_road_length)
    }

    /// Helper method to get the current state of initial placement phase
    /// Returns (total_settlements, total_roads, is_phase_1_complete, is_phase_2_complete)
    pub fn get_initial_placement_progress(&self) -> (usize, usize, bool, bool) {
        let total_settlements = self
            .buildings_by_color
            .values()
            .map(|buildings| {
                buildings
                    .iter()
                    .filter(|b| matches!(b, Building::Settlement(_, _)))
                    .count()
            })
            .sum::<usize>();
        let total_roads = self.roads.len() / 2; // Each road stored twice
        let num_players = self.config.num_players as usize;

        let phase_1_complete = total_settlements >= num_players && total_roads >= num_players;
        let phase_2_complete =
            total_settlements >= 2 * num_players && total_roads >= 2 * num_players;

        (
            total_settlements,
            total_roads,
            phase_1_complete,
            phase_2_complete,
        )
    }

    fn build_road(&mut self, placing_color: u8, edge_id: EdgeId) -> (Option<u8>, u8) {
        let inverted_edge = (edge_id.1, edge_id.0);

        // DEBUG: Log road building details
        log::debug!(
            "🛣️  Building road for player {} on edge ({}, {}) and inverted ({}, {})",
            placing_color,
            edge_id.0,
            edge_id.1,
            inverted_edge.0,
            inverted_edge.1
        );

        // DEBUG: Log existing roads before insertion
        let existing_roads_count = self.roads.len() / 2; // Each road stored twice
        log::debug!(
            "📊 Before insertion: {} roads in storage, inserting for player {}",
            existing_roads_count,
            placing_color
        );

        self.roads.insert(edge_id, placing_color);
        self.roads.insert(inverted_edge, placing_color);
        self.roads_by_color[placing_color as usize] += 1;

        // DEBUG: Log after insertion
        log::debug!(
            "📊 After insertion: {} roads in storage, player {} now has {} roads",
            self.roads.len() / 2,
            placing_color,
            self.roads_by_color[placing_color as usize]
        );

        let is_initial_build_phase = self.is_initial_build_phase();
        let is_road_building = self.is_road_building();
        let is_free = is_initial_build_phase || is_road_building;
        if !is_free {
            freqdeck_sub(self.get_mut_player_hand(placing_color), ROAD_COST);
            freqdeck_add(&mut self.vector[BANK_RESOURCE_SLICE], ROAD_COST);
        }

        // If this is a free road from Road Building card, decrement the counter
        if is_road_building {
            self.vector[FREE_ROADS_AVAILABLE_INDEX] -= 1;
        }

        if is_initial_build_phase {
            // BUGFIX: Count only settlements, not all buildings
            let num_settlements = self
                .buildings_by_color
                .values()
                .map(|buildings| {
                    buildings
                        .iter()
                        .filter(|b| matches!(b, Building::Settlement(_, _)))
                        .count()
                })
                .sum::<usize>();
            let num_roads = self.roads.len() / 2; // Each road is stored twice (a,b) and (b,a)
            let num_players = self.config.num_players as usize;

            log::info!(
                "🏗️  Initial build: {} settlements, {} roads ({} players)",
                num_settlements,
                num_roads,
                num_players
            );

            // Catan initial placement rules:
            // Phase 1: Each player places 1 settlement + 1 road (forward order: 0,1,2,3)
            // Phase 2: Each player places 1 settlement + 1 road (reverse order: 3,2,1,0)

            let going_forward = num_settlements <= num_players;
            let at_phase_transition = num_settlements == num_players && num_roads == num_players;
            let initial_phase_complete =
                num_settlements == 2 * num_players && num_roads == 2 * num_players;

            if initial_phase_complete {
                // All initial placements done - start normal gameplay
                self.vector[IS_INITIAL_BUILD_PHASE_INDEX] = 0;
                log::info!("🎯 Initial build phase COMPLETE → Normal gameplay");
            } else if at_phase_transition {
                // Transition from forward to reverse order - don't advance turn
                // The last player to place in forward order places first in reverse order
                log::info!("🔄 Phase transition: forward → reverse order");
                // No turn advancement - current player continues
            } else if going_forward {
                // Phase 1: advance turn forward (0->1->2->3)
                self.advance_turn(1);
                log::info!("➡️  Phase 1: turn advanced forward");
            } else {
                // Phase 2: advance turn backward (3->2->1->0)
                self.advance_turn(-1);
                log::info!("⬅️  Phase 2: turn advanced backward");
            }
        }

        // Maintain caches and longest road =====
        // Extend or merge components
        let (a, b) = edge_id;
        let a_index = self.get_connected_component_index(placing_color, a);
        let b_index = self.get_connected_component_index(placing_color, b);

        // Make sure the connected_components for this color exists
        self.connected_components.entry(placing_color).or_default();

        // Update connected components based on the new road
        let affected_component =
            self.update_connected_components(placing_color, a, b, a_index, b_index);

        let prev_road_color = self.longest_road_color;
        let prev_road_length = self.longest_road_length;

        // Calculate length for affected component
        let path_length = self
            .longest_acyclic_path(&affected_component, placing_color)
            .len() as u8;

        let (new_road_color, new_road_length) =
            if path_length >= 5 && path_length > prev_road_length {
                (Some(placing_color), path_length)
            } else {
                (prev_road_color, prev_road_length)
            };
        (new_road_color, new_road_length)
    }

    /// Updates the road network when a new road is built
    ///
    /// This method maintains the connected components for a player's road network:
    /// - Merges components when a road connects two previously separate networks
    /// - Extends an existing component when a road connects to it
    /// - Creates a new component for isolated roads
    ///
    /// The function also handles enemy settlements that would block connections.
    ///
    /// Returns the affected component that contains the new road.
    fn update_connected_components(
        &mut self,
        placing_color: u8,
        a: NodeId,
        b: NodeId,
        a_index: Option<usize>,
        b_index: Option<usize>,
    ) -> HashSet<NodeId> {
        // Pre-compute node validity before mutable borrow
        let a_valid = !self.is_enemy_node(placing_color, a);
        let b_valid = !self.is_enemy_node(placing_color, b);

        // Get the components list for this color, creating it if it doesn't exist
        let components = self.connected_components.entry(placing_color).or_default();

        // Case 1: Both nodes are in components
        if let (Some(a_idx), Some(b_idx)) = (a_index, b_index) {
            if a_idx == b_idx {
                // Both in same component - no change needed
                return components[a_idx].clone();
            }

            // Merge components - always merge into the component with smaller index
            // to minimize shifts in the vector
            let (keep_idx, remove_idx) = if a_idx < b_idx {
                (a_idx, b_idx)
            } else {
                (b_idx, a_idx)
            };

            let removed = components.remove(remove_idx);
            components[keep_idx].extend(removed);
            return components[keep_idx].clone();
        }

        // Case 2: Only one node is in a component - extend that component
        if let Some(idx) = a_index.or(b_index) {
            let component = &mut components[idx];

            // Add the node that isn't in a component if it's valid
            let new_node = if a_index.is_some() { b } else { a };
            let is_valid = if a_index.is_some() { b_valid } else { a_valid };

            if is_valid {
                component.insert(new_node);
            }

            return component.clone();
        }

        // Case 3: Neither node is in a component - create a new one with valid nodes
        let mut new_component = HashSet::new();
        if a_valid {
            new_component.insert(a);
        }
        if b_valid {
            new_component.insert(b);
        }

        if !new_component.is_empty() {
            components.push(new_component.clone());
        }

        new_component
    }

    fn build_city(&mut self, color: u8, node_id: u8) {
        // Update the main buildings HashMap
        self.buildings
            .insert(node_id, Building::City(color, node_id));

        // Update the buildings_by_color tracking
        let buildings = self.buildings_by_color.entry(color).or_default();

        // Remove the settlement from buildings_by_color
        if let Some(pos) = buildings.iter().position(|b| {
            if let Building::Settlement(_, n) = b {
                *n == node_id
            } else {
                false
            }
        }) {
            buildings.remove(pos);
        }

        // BUGFIX: Add the new city to buildings_by_color
        buildings.push(Building::City(color, node_id));

        freqdeck_sub(self.get_mut_player_hand(color), CITY_COST);
        freqdeck_add(&mut self.vector[BANK_RESOURCE_SLICE], CITY_COST);
        self.add_victory_points(color, 1);
    }

    fn buy_development_card(&mut self, color: u8) -> Option<DevCard> {
        // Get next card from deck
        if let Some(card) = take_next_dev_card(&mut self.vector) {
            // Pay for the card
            freqdeck_sub(self.get_mut_player_hand(color), DEVCARD_COST);
            freqdeck_add(&mut self.vector[BANK_RESOURCE_SLICE], DEVCARD_COST);

            let dev_card = match card {
                0 => DevCard::Knight,
                1 => DevCard::YearOfPlenty,
                2 => DevCard::Monopoly,
                3 => DevCard::RoadBuilding,
                4 => DevCard::VictoryPoint,
                _ => panic!("Invalid dev card index"),
            };

            match dev_card {
                DevCard::VictoryPoint => {
                    self.add_victory_points(color, 1);
                }
                _ => {
                    let dev_hand =
                        &mut self.vector[player_devhand_slice(self.config.num_players, color)];
                    dev_hand[card as usize] += 1;
                }
            }

            Some(dev_card)
        } else {
            None
        }
    }

    fn roll_dice(&mut self, color: u8, dice_opt: Option<(u8, u8)>) {
        self.vector[HAS_ROLLED_INDEX] = 1;
        let (die1, die2) = dice_opt.unwrap_or_else(|| {
            let mut rng = rand::thread_rng();
            (rng.gen_range(1..=6), rng.gen_range(1..=6))
        });

        // Store the dice roll for logging purposes
        self.last_dice_roll = Some((die1, die2));

        let total = die1 + die2;

        log::info!("🎲 Player {} rolled {} + {} = {}", color, die1, die2, total);

        if total == 7 {
            log::info!("🎲 Rolling 7 → Discard/Robber phase");
            self.handle_roll_seven(color);
        } else {
            log::info!("🎲 Rolling {} → Resource distribution", total);
            self.distribute_roll_yields(total);
            self.vector[CURRENT_TICK_SEAT_INDEX] = color;
        }
    }

    fn handle_roll_seven(&mut self, color: u8) {
        // Check who needs to discard
        let discarders: Vec<bool> = (0..self.get_num_players())
            .map(|c| {
                let player_hand = self.get_player_hand(c);
                let total_cards: u8 = player_hand.iter().sum();
                total_cards > self.config.discard_limit
            })
            .collect();

        let should_enter_discard_phase = discarders.iter().any(|&x| x);
        if should_enter_discard_phase {
            self.vector[IS_DISCARDING_INDEX] = 1;
            self.vector[CURRENT_TICK_SEAT_INDEX] = color;
            log::info!(
                "🎲 Rolling 7: Entering discard phase, original roller: {}",
                color
            );
        } else {
            self.vector[IS_MOVING_ROBBER_INDEX] = 1;
            self.vector[CURRENT_TICK_SEAT_INDEX] = color;
            log::info!("🎲 Rolling 7: No discards needed, moving to robber");
        }
    }

    // Returns Vec of (color, resource_index, amount) tuples for what each player should receive
    fn collect_roll_yields(&self, roll: u8) -> Vec<(u8, usize, u8)> {
        let mut all_yields = Vec::new();
        let matching_tiles = self.map_instance.get_tiles_by_number(roll);

        for tile in matching_tiles {
            // Skip robber tile
            if self.get_robber_tile() == tile.id {
                continue;
            }

            if let Some(resource) = tile.resource {
                let resource_idx = resource as usize;
                // Collect all yields for this tile
                for &node_id in tile.hexagon.nodes.values() {
                    if let Some(building) = self.buildings.get(&node_id) {
                        match building {
                            Building::Settlement(owner_color, _) => {
                                all_yields.push((*owner_color, resource_idx, 1));
                            }
                            Building::City(owner_color, _) => {
                                all_yields.push((*owner_color, resource_idx, 2));
                            }
                        }
                    }
                }
            }
        }
        all_yields
    }

    fn distribute_roll_yields(&mut self, roll: u8) {
        let yields = self.collect_roll_yields(roll);
        if yields.is_empty() {
            log::info!("🎲 Roll {} yields NO resources", roll);
            return;
        }

        log::info!("🎲 Roll {} yields: {:?}", roll, yields);

        // Calculate total needed by resource type
        let mut resource_needs = [0u8; 5];
        for (_, resource_idx, amount) in &yields {
            resource_needs[*resource_idx] += amount;
        }

        // Check what can be allocated from bank
        let bank = &self.vector[BANK_RESOURCE_SLICE];
        log::info!(
            "🏦 Current bank: [Wood:{}, Brick:{}, Sheep:{}, Wheat:{}, Ore:{}]",
            bank[0],
            bank[1],
            bank[2],
            bank[3],
            bank[4]
        );
        log::info!("📦 Total resource needs: {:?}", resource_needs);

        // For each resource type, determine if multiple players need it
        let mut resource_recipients = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        for (color, resource_idx, _) in &yields {
            if !resource_recipients[*resource_idx].contains(color) {
                resource_recipients[*resource_idx].push(*color);
            }
        }

        // Determine which resources can be distributed
        let mut can_distribute = [true; 5];
        for i in 0..5 {
            if bank[i] < resource_needs[i] {
                // Resource is insufficient
                if resource_recipients[i].len() > 1 {
                    // Multiple players need this resource - no one gets it
                    log::info!(
                        "❌ Resource {}: insufficient for multiple recipients ({}), no one gets it",
                        i,
                        resource_recipients[i].len()
                    );
                    can_distribute[i] = false;
                } else {
                    log::info!("⚠️  Resource {}: insufficient for single recipient, will distribute what's available", i);
                    // Single player - they get what's available (handled during distribution)
                }
            }
        }

        // Make a copy of bank resources to track what's distributed
        let mut remaining = [0u8; 5];
        remaining.copy_from_slice(&bank[..5]);

        // Distribute resources according to the rules
        for (owner_color, resource_idx, amount) in yields {
            if !can_distribute[resource_idx] {
                // Skip resources that can't be distributed
                continue;
            }

            // Calculate how much to give (either full amount or what's available)
            let available = remaining[resource_idx].min(amount);
            if available > 0 {
                // Update tracking of what's left
                remaining[resource_idx] -= available;

                // Update actual game state
                self.vector[BANK_RESOURCE_SLICE][resource_idx] -= available;
                self.get_mut_player_hand(owner_color)[resource_idx] += available;

                log::info!(
                    "✅ Distributed {} of resource {} to player {}",
                    available,
                    resource_idx,
                    owner_color
                );
            }
        }

        log::info!(
            "🏦 Bank after distribution: [Wood:{}, Brick:{}, Sheep:{}, Wheat:{}, Ore:{}]",
            &self.vector[BANK_RESOURCE_SLICE][0],
            &self.vector[BANK_RESOURCE_SLICE][1],
            &self.vector[BANK_RESOURCE_SLICE][2],
            &self.vector[BANK_RESOURCE_SLICE][3],
            &self.vector[BANK_RESOURCE_SLICE][4]
        );
    }

    /*
     * TODO: For now, we're not letting players choose what to discard, to avoid
     * the combinatorial explosion of possibilities. Instead, we'll just
     * force discards in a way that maximizes resource diversity.
     */
    fn discard(&mut self, color: u8) {
        let mut remaining_hand = self.get_player_hand(color).to_vec();
        let total_cards: u8 = remaining_hand.iter().sum();
        let mut to_discard = total_cards - (total_cards / 2);
        let mut discarded = [0u8; 5];

        while to_discard > 0 {
            // Find highest frequency resources
            let max_count = *remaining_hand.iter().max().unwrap();
            let max_indices: Vec<_> = (0..5).filter(|&i| remaining_hand[i] == max_count).collect();

            // Take one card from each highest frequency resource
            for &i in &max_indices {
                if to_discard > 0 {
                    remaining_hand[i] -= 1;
                    discarded[i] += 1;
                    to_discard -= 1;
                }
            }
        }

        freqdeck_sub(self.get_mut_player_hand(color), discarded);
        freqdeck_add(&mut self.vector[BANK_RESOURCE_SLICE], discarded);

        log::info!(
            "🗑️  Player {} discarded {} cards: {:?}",
            color,
            discarded.iter().sum::<u8>(),
            discarded
        );

        // BUGFIX: Handle proper discard turn advancement and state transitions
        self.advance_discard_turn();
    }

    fn advance_discard_turn(&mut self) {
        // CRITICAL BUGFIX: Fix the discard advancement logic to handle seating order properly

        let current_tick_seat_index = self.vector[CURRENT_TICK_SEAT_INDEX] as usize;
        let seating_order = self.get_seating_order();
        let current_discarder_color = seating_order[current_tick_seat_index];
        let num_players = self.get_num_players();

        log::info!(
            "🔍 Checking for remaining discarders (current: {} at index {}, limit: {})",
            current_discarder_color,
            current_tick_seat_index,
            self.config.discard_limit
        );

        // BUGFIX: Check if the CURRENT discarder still needs to discard FIRST
        let current_hand = self.get_player_hand(current_discarder_color);
        let current_total: u8 = current_hand.iter().sum();
        log::info!(
            "🔍 Current discarder {} has {} cards",
            current_discarder_color,
            current_total
        );

        if current_total > self.config.discard_limit {
            log::info!(
                "➡️  Current discarder still needs to discard: Player {} ({} cards > {})",
                current_discarder_color,
                current_total,
                self.config.discard_limit
            );
            return; // Stay with current discarder - they still need to discard
        }

        // Current discarder is done, check remaining players by going through seating order
        for i in 1..num_players {
            let next_seating_index = (current_tick_seat_index + i as usize) % num_players as usize;
            let next_player_color = seating_order[next_seating_index];
            let player_hand = self.get_player_hand(next_player_color);
            let total_cards: u8 = player_hand.iter().sum();

            log::info!(
                "🔍 Player {} (index {}) has {} cards (limit: {})",
                next_player_color,
                next_seating_index,
                total_cards,
                self.config.discard_limit
            );

            if total_cards > self.config.discard_limit {
                // Found next discarder
                self.vector[CURRENT_TICK_SEAT_INDEX] = next_seating_index as u8;
                log::info!(
                    "➡️  Next discarder: Player {} at index {} ({} cards > {})",
                    next_player_color,
                    next_seating_index,
                    total_cards,
                    self.config.discard_limit
                );
                return;
            }
        }

        // No more discarders found - transition to robber movement
        self.vector[IS_DISCARDING_INDEX] = 0;
        self.vector[IS_MOVING_ROBBER_INDEX] = 1;
        // Return to the player who originally rolled the 7
        self.vector[CURRENT_TICK_SEAT_INDEX] = self.vector[CURRENT_TURN_SEAT_INDEX];
        log::info!(
            "🎯 All discards complete → Moving robber (Player {})",
            self.vector[CURRENT_TURN_SEAT_INDEX]
        );
    }

    fn move_robber(&mut self, color: u8, coordinate: (i8, i8, i8), victim_opt: Option<u8>) {
        self.set_robber_tile(self.map_instance.get_land_tile(coordinate).unwrap().id);

        if let Some(victim) = victim_opt {
            let total_cards: u8 = self.get_player_hand(victim).iter().sum();

            if total_cards > 0 {
                // Randomly select card to steal
                let mut rng = rand::thread_rng();
                let selected_idx = rng.gen_range(0..total_cards);

                let mut cumsum = 0;
                let mut stolen_resource_idx = 0;
                for (i, &count) in self.get_player_hand(victim).iter().enumerate() {
                    cumsum += count;
                    if selected_idx < cumsum {
                        stolen_resource_idx = i;
                        break;
                    }
                }

                let mut stolen_freqdeck = [0; 5];
                stolen_freqdeck[stolen_resource_idx] = 1;
                freqdeck_sub(self.get_mut_player_hand(victim), stolen_freqdeck);
                freqdeck_add(self.get_mut_player_hand(color), stolen_freqdeck);
            }
        }
        self.vector[IS_MOVING_ROBBER_INDEX] = 0;
    }

    fn maintain_longest_road(&mut self, new_owner: Option<u8>, new_length: u8) {
        let prev_owner = self.longest_road_color;
        self.longest_road_color = new_owner;
        self.longest_road_length = new_length;

        if new_owner == prev_owner {
            return;
        }

        if let Some(prev_owner) = prev_owner {
            self.sub_victory_points(prev_owner, 2);
        }

        if let Some(new_owner) = new_owner {
            self.add_victory_points(new_owner, 2);
        }
    }

    fn dfs_walk(&self, start_node: NodeId, color: u8) -> HashSet<NodeId> {
        let mut agenda = vec![start_node];
        let mut visited = HashSet::new();

        while let Some(node) = agenda.pop() {
            if visited.contains(&node) {
                continue;
            }
            visited.insert(node);

            if self.is_enemy_node(color, node) {
                continue;
            }

            for neighbor in self.map_instance.get_neighbor_nodes(node) {
                let edge = (node.min(neighbor), node.max(neighbor));
                if self.roads.get(&edge) == Some(&color) {
                    agenda.push(neighbor);
                }
            }
        }
        visited
    }

    fn play_knight(&mut self, color: u8) {
        // Mark card as played
        self.remove_dev_card(color, DevCard::Knight as usize);
        self.add_played_dev_card(color, DevCard::Knight as usize);
        self.set_has_played_dev_card();

        // Set state to move robber
        self.set_is_moving_robber();
    }

    fn maintain_largest_army(&mut self) {
        let prev_owner = self.largest_army_color;
        let prev_count = self.largest_army_count;

        // Find player with most knights (if any have 3 or more)
        let mut max_knights = 0;
        let mut max_knights_color = None;

        for color in 0..self.get_num_players() {
            let knights = self.get_played_dev_card_count(color, DevCard::Knight as usize);
            if knights >= 3 && knights > max_knights {
                max_knights = knights;
                max_knights_color = Some(color);
            }
        }

        // Case where playerB meets playerA's largest army -> no change
        if max_knights == prev_count {
            return;
        }

        self.largest_army_color = max_knights_color;
        self.largest_army_count = max_knights;

        // If playerA retains largest army -> no VP changes
        if max_knights_color == prev_owner {
            return;
        }

        if let Some(prev_owner) = prev_owner {
            self.sub_victory_points(prev_owner, 2);
        }

        if let Some(new_owner) = max_knights_color {
            self.add_victory_points(new_owner, 2);
        }
    }

    fn play_year_of_plenty(&mut self, color: u8, resources: (u8, Option<u8>)) {
        // Assume move_generation has already checked that player has year of plenty card
        // and that bank has enough resources
        self.remove_dev_card(color, DevCard::YearOfPlenty as usize);
        self.add_played_dev_card(color, DevCard::YearOfPlenty as usize);
        self.set_has_played_dev_card();

        // Give first resource to player
        self.from_bank_to_player(color, resources.0);

        // Give second resource if specified
        if let Some(resource2) = resources.1 {
            self.from_bank_to_player(color, resource2);
        }
    }

    fn play_monopoly(&mut self, color: u8, resource: u8) {
        // Assume move_generation has already checked that player has monopoly card.
        self.remove_dev_card(color, DevCard::Monopoly as usize);
        self.add_played_dev_card(color, DevCard::Monopoly as usize);
        self.set_has_played_dev_card();

        // Steal all resources of type from other players
        for victim_color in 0..self.get_num_players() {
            if victim_color != color {
                let amount = self.get_player_resource_count(victim_color, resource);
                if amount > 0 {
                    self.from_player_to_player(victim_color, color, resource, amount);
                }
            }
        }
    }

    fn play_road_building(&mut self, color: u8) {
        // Assume move_generation has already checked that player has road building card.
        self.remove_dev_card(color, DevCard::RoadBuilding as usize);
        self.add_played_dev_card(color, DevCard::RoadBuilding as usize);
        self.set_has_played_dev_card();

        // Set state for free roads
        self.vector[IS_BUILDING_ROAD_INDEX] = 1;
        self.vector[FREE_ROADS_AVAILABLE_INDEX] = 2;
    }

    fn maritime_trade(&mut self, color: u8, give: u8, take: u8, ratio: u8) {
        // Assume move_generation has already checked that player has enough resources
        // to give and that bank has enough resources to take
        self.from_player_to_bank(color, give, ratio);
        self.from_bank_to_player(color, take);
    }

    fn end_turn(&mut self, _color: u8) {
        // BUGFIX: Handle discard phase properly
        if self.is_discarding() {
            // During discard phase, EndTurn should advance discard turn, not regular turn
            self.advance_discard_turn();
        } else {
            // Normal turn advancement
            self.vector[HAS_PLAYED_DEV_CARD] = 0;
            self.vector[HAS_ROLLED_INDEX] = 0;
            self.advance_turn(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_settlement_initial_build_phase() {
        let mut state = State::new_base();
        let color = state.get_current_color();
        assert_eq!(state.buildings.get(&0), None);
        assert_eq!(state.board_buildable_ids.len(), 54);
        assert_eq!(state.get_actual_victory_points(color), 0);

        let node_id = 0;
        state.build_settlement(color, node_id);

        assert_eq!(
            state.buildings.get(&node_id),
            Some(&Building::Settlement(color, node_id))
        );
        assert_eq!(state.board_buildable_ids.len(), 50);
        assert_eq!(state.get_actual_victory_points(color), 1);
    }

    #[test]
    fn test_build_settlement_spends_resources() {
        let mut state = State::new_base();
        let color = state.get_current_color();
        assert_eq!(state.buildings.get(&0), None);
        assert_eq!(state.board_buildable_ids.len(), 54);
        assert_eq!(state.get_actual_victory_points(color), 0);

        // Exit initial build phase
        state.vector[IS_INITIAL_BUILD_PHASE_INDEX] = 0;

        freqdeck_add(state.get_mut_player_hand(color), SETTLEMENT_COST);
        let hand_before = state.get_player_hand(color).to_vec();

        let node_id = 0;
        state.build_settlement(color, node_id);

        assert_eq!(
            state.buildings.get(&node_id),
            Some(&Building::Settlement(color, node_id))
        );
        assert_eq!(state.board_buildable_ids.len(), 50);
        assert_eq!(state.get_actual_victory_points(color), 1);

        let hand_after = state.get_player_hand(color);
        for i in 0..5 {
            assert_eq!(hand_after[i], hand_before[i] - SETTLEMENT_COST[i]);
        }
    }

    #[test]
    fn test_roll_seven_triggers_discard() {
        let mut state = State::new_base();
        let color = state.get_current_color();

        {
            let hand = state.get_mut_player_hand(color);
            hand[0] = 8; // Give 8 wood cards
        }

        state.roll_dice(color, Some((4, 3)));

        assert_eq!(state.vector[HAS_ROLLED_INDEX], 1);
        assert_eq!(state.vector[IS_DISCARDING_INDEX], 1);
        assert_eq!(state.vector[CURRENT_TICK_SEAT_INDEX], color);
        assert_eq!(state.vector[IS_MOVING_ROBBER_INDEX], 0);
    }

    #[test]
    fn test_roll_seven_no_discard_needed() {
        let mut state = State::new_base();
        let color = state.get_current_color();

        state.roll_dice(color, Some((4, 3)));

        assert_eq!(state.vector[HAS_ROLLED_INDEX], 1);
        assert_eq!(state.vector[IS_DISCARDING_INDEX], 0);
        assert_eq!(state.vector[CURRENT_TICK_SEAT_INDEX], color);
        assert_eq!(state.vector[IS_MOVING_ROBBER_INDEX], 1);
    }

    #[test]
    fn test_roll_tracks_has_rolled() {
        let mut state = State::new_base();
        let color = state.get_current_color();

        assert_eq!(state.vector[HAS_ROLLED_INDEX], 0);
        state.roll_dice(color, Some((2, 3)));
        assert_eq!(state.vector[HAS_ROLLED_INDEX], 1);
    }

    #[test]
    fn test_second_settlement_yields_resources() {
        let mut state = State::new_base();
        let color = state.get_current_color();
        let first_node = 0;
        let bank_before = state.vector[BANK_RESOURCE_SLICE].to_vec();
        let hand_before = state.get_player_hand(color).to_vec();

        state.build_settlement(color, first_node);

        assert_eq!(state.get_player_hand(color), hand_before);
        assert_eq!(state.vector[BANK_RESOURCE_SLICE], bank_before);

        let second_node = 3;
        let bank_before = state.vector[BANK_RESOURCE_SLICE].to_vec();
        let hand_before = state.get_player_hand(color).to_vec();

        state.build_settlement(color, second_node);

        assert_ne!(state.get_player_hand(color), hand_before);
        assert_ne!(state.vector[BANK_RESOURCE_SLICE], bank_before);

        for i in 0..5 {
            let bank_diff = bank_before[i] - state.vector[BANK_RESOURCE_SLICE][i];
            let hand_diff = state.get_player_hand(color)[i] - hand_before[i];
            assert_eq!(bank_diff, hand_diff);
        }
    }

    #[test]
    fn test_settlement_cuts_longest_road() {
        let mut state = State::new_base();
        let color1 = 1;
        let color2 = 2;

        // give color1 6 consecutive roads
        state.apply_action(Action::BuildSettlement {
            color: color1,
            node_id: 0,
        });
        for edge in [(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 16)] {
            state.apply_action(Action::BuildRoad {
                color: color1,
                edge_id: edge,
            });
        }

        assert_eq!(state.longest_road_color, Some(color1));
        assert_eq!(state.get_actual_victory_points(color1), 3);
        assert_eq!(state.get_actual_victory_points(color2), 0);

        // Give color2 a settlement at node 4 to bisect color1's Longest Road
        state.vector[IS_INITIAL_BUILD_PHASE_INDEX] = 0;
        freqdeck_add(state.get_mut_player_hand(color2), SETTLEMENT_COST);
        state.apply_action(Action::BuildSettlement {
            color: color2,
            node_id: 4,
        });

        assert_eq!(state.longest_road_color, None);
        assert_eq!(state.get_actual_victory_points(color1), 1);
        assert_eq!(state.get_actual_victory_points(color2), 1);
    }

    #[test]
    fn test_build_road_maintains_connected_components() {
        let mut state = State::new_base();
        let color1 = 1;

        state.build_settlement(color1, 0);
        state.build_road(color1, (0, 1));

        let components = state.connected_components.get(&color1).unwrap();
        assert_eq!(components.len(), 1);
        assert_eq!(components[0], HashSet::from([0, 1]));

        state.build_road(color1, (1, 2));

        let components = state.connected_components.get(&color1).unwrap();
        assert_eq!(components.len(), 1);
        assert_eq!(components[0], HashSet::from([0, 1, 2]));

        state.build_settlement(color1, 4);
        state.build_road(color1, (3, 4));

        let components = state.connected_components.get(&color1).unwrap();
        assert_eq!(components.len(), 2);
        assert_eq!(components[0], HashSet::from([0, 1, 2]));
        assert_eq!(components[1], HashSet::from([3, 4]));

        state.build_road(color1, (2, 3));

        let components = state.connected_components.get(&color1).unwrap();
        assert_eq!(components.len(), 1);
        assert_eq!(components[0], HashSet::from([0, 1, 2, 3, 4]));
    }

    #[test]
    fn test_settlement_cuts_longest_road_and_transfers() {
        let mut state = State::new_base();
        let color1 = 1;
        let color2 = 2;

        // give color1 6 consecutive roads
        state.apply_action(Action::BuildSettlement {
            color: color1,
            node_id: 0,
        });
        for edge in [(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 16)] {
            state.apply_action(Action::BuildRoad {
                color: color1,
                edge_id: edge,
            });
        }
        // Give color2 5 consecutive roads with potential to bisect/plow color1's road
        state.apply_action(Action::BuildSettlement {
            color: color2,
            node_id: 11,
        });
        for edge in [(11, 12), (12, 13), (13, 14), (14, 15), (4, 15)] {
            state.apply_action(Action::BuildRoad {
                color: color2,
                edge_id: edge,
            });
        }

        assert_eq!(state.longest_road_color, Some(color1));
        assert_eq!(state.get_actual_victory_points(color1), 3);
        assert_eq!(state.get_actual_victory_points(color2), 1);

        // Give color2 a settlement at node 4 to bisect color1's Longest Road
        state.vector[IS_INITIAL_BUILD_PHASE_INDEX] = 0;
        freqdeck_add(state.get_mut_player_hand(color2), SETTLEMENT_COST);
        state.apply_action(Action::BuildSettlement {
            color: color2,
            node_id: 4,
        });

        assert_eq!(state.longest_road_color, Some(color2));
        assert_eq!(state.get_actual_victory_points(color1), 1);
        assert_eq!(state.get_actual_victory_points(color2), 4);
    }

    #[test]
    fn test_extend_own_longest_road() {
        let mut state = State::new_base();
        let color1 = 1;

        state.apply_action(Action::BuildSettlement {
            color: color1,
            node_id: 0,
        });
        for edge in [(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)] {
            state.apply_action(Action::BuildRoad {
                color: color1,
                edge_id: edge,
            });
        }

        assert_eq!(state.longest_road_color, Some(color1));
        assert_eq!(state.longest_road_length, 5);
        assert_eq!(state.get_actual_victory_points(color1), 3);

        state.apply_action(Action::BuildRoad {
            color: color1,
            edge_id: (5, 16),
        });

        assert_eq!(state.longest_road_color, Some(color1));
        assert_eq!(state.longest_road_length, 6);
        assert_eq!(state.get_actual_victory_points(color1), 3);
    }

    #[test]
    fn test_bisection_counts_remaining_components() {
        let mut state = State::new_base();
        let color1 = 1;
        let color2 = 2;

        state.apply_action(Action::BuildSettlement {
            color: color1,
            node_id: 0,
        });
        for edge in [(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 16)] {
            state.apply_action(Action::BuildRoad {
                color: color1,
                edge_id: edge,
            });
        }

        assert_eq!(state.longest_road_color, Some(color1));
        assert_eq!(state.longest_road_length, 6);
        assert_eq!(state.get_actual_victory_points(color1), 3);

        state.vector[IS_INITIAL_BUILD_PHASE_INDEX] = 0;
        freqdeck_add(state.get_mut_player_hand(color2), SETTLEMENT_COST);
        state.apply_action(Action::BuildSettlement {
            color: color2,
            node_id: 5,
        });

        assert_eq!(state.longest_road_color, Some(color1));
        assert_eq!(state.longest_road_length, 5);
        assert_eq!(state.connected_components.get(&color1).unwrap().len(), 2);
        assert_eq!(state.get_actual_victory_points(color1), 3);
        assert_eq!(state.get_actual_victory_points(color2), 1);
    }

    #[test]
    fn test_buy_development_cards() {
        let mut state = State::new_base();
        let color = state.get_current_color();
        let mut cards_drawn = 0;

        while cards_drawn < 26 {
            freqdeck_add(state.get_mut_player_hand(color), DEVCARD_COST);
            let initial_hand: [u8; 5] = state.get_player_hand(color).try_into().unwrap();
            let initial_devhand = state.get_player_devhand(color).to_vec();
            let initial_bank = state.vector[BANK_RESOURCE_SLICE].to_vec();
            let initial_vps = state.get_actual_victory_points(color);

            let drawn_card = state.buy_development_card(color);
            cards_drawn += 1;

            log::debug!("Cards Drawn: {}, Drawn card: {:?}", cards_drawn, drawn_card);

            if cards_drawn < 26 {
                let hand_after = state.get_player_hand(color);
                let bank_after = &state.vector[BANK_RESOURCE_SLICE];
                for i in 0..5 {
                    assert_eq!(hand_after[i], initial_hand[i] - DEVCARD_COST[i]);
                    assert_eq!(bank_after[i], initial_bank[i] + DEVCARD_COST[i]);
                }
                let devhand_after = state.get_player_devhand(color);

                if drawn_card == Some(DevCard::VictoryPoint) {
                    // VP added, devhand not incremented
                    assert_eq!(state.get_actual_victory_points(color), initial_vps + 1);
                    assert_eq!(
                        devhand_after[drawn_card.unwrap() as usize],
                        initial_devhand[drawn_card.unwrap() as usize]
                    );
                } else {
                    // VP not added, devhand incremented
                    assert_eq!(state.get_actual_victory_points(color), initial_vps);
                    assert_eq!(
                        devhand_after[drawn_card.unwrap() as usize],
                        initial_devhand[drawn_card.unwrap() as usize] + 1
                    );
                }
            } else {
                // 26th card should not be drawn
                assert!(drawn_card.is_none());
                assert_eq!(state.get_player_hand(color), initial_hand);
                assert_eq!(&state.vector[BANK_RESOURCE_SLICE], initial_bank);
            }
        }
    }

    #[test]
    fn test_roll_yields_resources() {
        let mut state = State::new_base();
        let color = state.get_current_color();

        state.build_settlement(color, 0);

        let adjacent_tiles = state.map_instance.get_adjacent_tiles(0).unwrap();

        let mut chosen_roll = None;
        let mut expected_resource_yields = [0; 5];

        for tile in adjacent_tiles.iter() {
            if let (Some(number), Some(resource)) = (tile.number, tile.resource) {
                // First valid number we find will be our roll
                // Don't pick robber tile
                if tile.id != state.get_robber_tile() {
                    if chosen_roll.is_none() {
                        chosen_roll = Some(number);
                    }

                    if Some(number) == chosen_roll {
                        expected_resource_yields[resource as usize] += 1;
                    }
                }
            }
        }

        let initial_bank = state.vector[BANK_RESOURCE_SLICE].to_vec();
        let initial_hand = state.get_player_hand(color).to_vec();
        // Roll numbers should sum to chosen_roll
        let roll_numbers = (chosen_roll.unwrap() / 2, (chosen_roll.unwrap() + 1) / 2);

        state.apply_action(Action::Roll {
            color,
            dice_opt: Some(roll_numbers),
        });

        for resource_idx in 0..5 {
            assert_eq!(
                state.vector[BANK_RESOURCE_SLICE][resource_idx],
                initial_bank[resource_idx] - expected_resource_yields[resource_idx],
                "Bank should have {} fewer resource of {:?}",
                expected_resource_yields[resource_idx],
                resource_idx
            );
            assert_eq!(
                state.get_player_hand(color)[resource_idx],
                initial_hand[resource_idx] + expected_resource_yields[resource_idx],
                "Player should have {} more resource of {:?}",
                expected_resource_yields[resource_idx],
                resource_idx
            )
        }
    }

    #[test]
    fn test_roll_city_yields_double() {
        let mut state = State::new_base();
        let color = state.get_current_color();

        freqdeck_add(state.get_mut_player_hand(color), CITY_COST);
        state.build_settlement(color, 0);
        state.build_city(color, 0);

        let adjacent_tiles = state.map_instance.get_adjacent_tiles(0).unwrap();

        let mut chosen_roll = None;
        let mut expected_resource_yields = [0; 5];

        for tile in adjacent_tiles.iter() {
            if let (Some(number), Some(resource)) = (tile.number, tile.resource) {
                // Don't pick robber tile
                if tile.id != state.get_robber_tile() {
                    if chosen_roll.is_none() {
                        chosen_roll = Some(number);
                    }

                    if Some(number) == chosen_roll {
                        expected_resource_yields[resource as usize] += 2;
                    }
                }
            }
        }

        let initial_bank = state.vector[BANK_RESOURCE_SLICE].to_vec();
        let initial_hand = state.get_player_hand(color).to_vec();
        // Roll numbers should sum to chosen_roll
        let roll_numbers = (chosen_roll.unwrap() / 2, (chosen_roll.unwrap() + 1) / 2);

        state.apply_action(Action::Roll {
            color,
            dice_opt: Some(roll_numbers),
        });

        for resource_idx in 0..5 {
            assert_eq!(
                state.vector[BANK_RESOURCE_SLICE][resource_idx],
                initial_bank[resource_idx] - expected_resource_yields[resource_idx],
                "Bank should have {} fewer resource of {:?}",
                expected_resource_yields[resource_idx],
                resource_idx
            );
            assert_eq!(
                state.get_player_hand(color)[resource_idx],
                initial_hand[resource_idx] + expected_resource_yields[resource_idx],
                "Player should have {} more resource of {:?}",
                expected_resource_yields[resource_idx],
                resource_idx
            );
        }
    }

    #[test]
    fn test_roll_single_player_partial_payment_when_insufficient_bank() {
        let mut state = State::new_base();
        let color = state.get_current_color();

        let node_id = 0;
        state.build_settlement(color, node_id);
        freqdeck_add(state.get_mut_player_hand(color), CITY_COST);
        state.build_city(color, node_id);

        let adjacent_tiles = state.map_instance.get_adjacent_tiles(node_id).unwrap();

        let mut chosen_roll = None;
        let mut chosen_resource = None;

        for tile in adjacent_tiles.iter() {
            if let (Some(number), Some(resource)) = (tile.number, tile.resource) {
                if tile.id != state.get_robber_tile() && chosen_roll.is_none() {
                    chosen_roll = Some(number);
                    chosen_resource = Some(resource);
                }
            }
        }
        assert!(chosen_roll.is_some(), "Should find at least one valid tile");

        for i in 0..5 {
            state.vector[BANK_RESOURCE_SLICE][i] = 1;
        }
        let hand_before = state.get_player_hand(color).to_vec();

        let roll = chosen_roll.unwrap();
        let roll_numbers = (roll / 2, (roll + 1) / 2);
        state.roll_dice(color, Some(roll_numbers));

        let chosen_resource_idx = chosen_resource.unwrap() as usize;
        assert_eq!(state.vector[BANK_RESOURCE_SLICE][chosen_resource_idx], 0);

        assert_eq!(
            state.get_player_hand(color)[chosen_resource_idx],
            hand_before[chosen_resource_idx] + 1
        );
        assert_eq!(state.vector[BANK_RESOURCE_SLICE][chosen_resource_idx], 0)
    }

    #[test]
    fn test_roll_multiple_player_no_payment_when_insufficient_bank() {
        let mut state = State::new_base();
        let color1 = 1;
        let color2 = 2;

        let (resource, number, node1, node2) = {
            let tile = state
                .map_instance
                .get_land_tiles()
                .values()
                .find(|tile| {
                    tile.resource.is_some() && // Not a desert
                    tile.id != state.get_robber_tile() // Not under robber
                })
                .expect("Should be at least one valid tile");

            let node_ids: Vec<_> = tile.hexagon.nodes.values().take(2).copied().collect();

            (
                tile.resource.unwrap(),
                tile.number.unwrap(),
                node_ids[0],
                node_ids[1],
            )
        };

        // Place two opposing cities on a shared tile with expected yields
        state.build_settlement(color1, node1);
        state.build_settlement(color2, node2);
        freqdeck_add(state.get_mut_player_hand(color1), CITY_COST);
        freqdeck_add(state.get_mut_player_hand(color2), CITY_COST);
        state.build_city(color1, node1);
        state.build_city(color2, node2);

        // Set bank to have only 1 of the needed resource
        let resource_idx = resource as usize;
        state.vector[BANK_RESOURCE_SLICE][resource_idx] = 1;

        let bank_before = state.vector[BANK_RESOURCE_SLICE][resource_idx];
        let hand1_before = state.get_player_hand(color1)[resource_idx];
        let hand2_before = state.get_player_hand(color2)[resource_idx];

        // Roll the shared tile's number
        let roll_numbers = (number / 2, (number + 1) / 2);
        state.roll_dice(color1, Some(roll_numbers));

        assert_eq!(
            state.vector[BANK_RESOURCE_SLICE][resource_idx], bank_before,
            "Bank should be unchanged"
        );
        // Neither player should get any resources
        assert_eq!(
            state.get_player_hand(color1)[resource_idx],
            hand1_before,
            "Player 1 should not receive resources"
        );
        assert_eq!(
            state.get_player_hand(color2)[resource_idx],
            hand2_before,
            "Player 2 should not receive resources"
        );
    }

    #[test]
    fn test_discard() {
        let mut state = State::new_base();
        let color = state.get_current_color();

        // Give the player a known distribution of 17 cards
        freqdeck_add(state.get_mut_player_hand(color), [3, 9, 1, 3, 1]);

        let bank_before = state.vector[BANK_RESOURCE_SLICE].to_vec();

        state.discard(color);

        // After discarding, the player should have half => 17 / 2 = 8.
        let total_after: u8 = state.get_player_hand(color).iter().sum();
        assert_eq!(total_after, 8, "Player should have exactly 8 cards left.");

        // Verify discard phase ended
        assert_eq!(
            state.vector[IS_DISCARDING_INDEX], 0,
            "Discard phase should end."
        );

        // The bank should have received exactly 6 more cards in total
        let bank_after = &state.vector[BANK_RESOURCE_SLICE];
        let mut total_discarded = 0;
        for i in 0..5 {
            total_discarded += bank_after[i] - bank_before[i];
        }
        assert_eq!(
            total_discarded, 9,
            "Exactly 9 cards should have been added to the bank."
        );

        // Check the specific distribution after discard
        let final_player_hand = state.get_player_hand(color);
        assert_eq!(
            final_player_hand,
            &[2, 2, 1, 2, 1],
            "Discard logic should spread discards across highest-frequency resources first."
        );
    }

    #[test]
    fn test_play_knight() {
        let mut state = State::new_base();
        let color = state.get_current_color();

        state.add_dev_card(color, DevCard::Knight as usize);
        assert_eq!(state.get_dev_card_count(color, DevCard::Knight as usize), 1);
        assert_eq!(
            state.get_played_dev_card_count(color, DevCard::Knight as usize),
            0
        );
        assert_eq!(state.vector[HAS_PLAYED_DEV_CARD], 0);
        assert_eq!(state.vector[IS_MOVING_ROBBER_INDEX], 0);

        state.play_knight(color);

        assert_eq!(state.get_dev_card_count(color, DevCard::Knight as usize), 0);
        assert_eq!(
            state.get_played_dev_card_count(color, DevCard::Knight as usize),
            1
        );
        assert_eq!(state.vector[HAS_PLAYED_DEV_CARD], 1);
        assert_eq!(state.vector[IS_MOVING_ROBBER_INDEX], 1);
    }

    #[test]
    fn test_play_knight_largest_army() {
        let mut state = State::new_base();
        let color1 = 1;
        let color2 = 2;

        // Give first player 3 knight cards
        for _ in 0..3 {
            state.add_dev_card(color1, DevCard::Knight as usize);
        }

        // Play knights and verify largest army
        for i in 0..3 {
            state.vector[HAS_PLAYED_DEV_CARD] = 0; // Reset for each turn
            state.apply_action(Action::PlayKnight { color: color1 });

            // Verify knight was removed and marked as played
            assert_eq!(
                state.get_dev_card_count(color1, DevCard::Knight as usize),
                2 - i
            );
            assert_eq!(
                state.get_played_dev_card_count(color1, DevCard::Knight as usize),
                i + 1
            );
            assert_eq!(state.vector[HAS_PLAYED_DEV_CARD], 1);
            assert_eq!(state.vector[IS_MOVING_ROBBER_INDEX], 1);

            // Check largest army status
            if i == 2 {
                assert_eq!(state.largest_army_color, Some(color1));
                assert_eq!(state.largest_army_count, 3);
                assert_eq!(state.get_actual_victory_points(color1), 2);
                assert_eq!(state.get_actual_victory_points(color2), 0);
            } else {
                assert_eq!(state.largest_army_color, None);
                assert_eq!(state.largest_army_count, 0);
                assert_eq!(state.get_actual_victory_points(color1), 0);
                assert_eq!(state.get_actual_victory_points(color2), 0);
            }
        }

        // Now give second player 4 knight cards and have them take largest army
        for _ in 0..4 {
            state.add_dev_card(color2, DevCard::Knight as usize);
        }

        // Play knights with second player
        for i in 0..4 {
            state.vector[HAS_PLAYED_DEV_CARD] = 0; // Reset for each turn
            state.apply_action(Action::PlayKnight { color: color2 });

            // Verify knight was removed and marked as played
            assert_eq!(
                state.get_dev_card_count(color2, DevCard::Knight as usize),
                3 - i
            );
            assert_eq!(
                state.get_played_dev_card_count(color2, DevCard::Knight as usize),
                i + 1
            );

            // Check largest army status
            if i == 3 {
                // After 4th knight, should take largest army
                assert_eq!(state.largest_army_color, Some(color2));
                assert_eq!(state.largest_army_count, 4);
                assert_eq!(state.get_actual_victory_points(color1), 0); // Lost 2 VPs
                assert_eq!(state.get_actual_victory_points(color2), 2); // Gained 2 VPs
            } else {
                // Still held by first player
                assert_eq!(state.largest_army_color, Some(color1));
                assert_eq!(state.largest_army_count, 3);
                assert_eq!(state.get_actual_victory_points(color1), 2);
                assert_eq!(state.get_actual_victory_points(color2), 0);
            }
        }
    }

    #[test]
    fn test_play_year_of_plenty() {
        let mut state = State::new_base();
        let color = state.get_current_color();

        // Give player a year of plenty card
        state.add_dev_card(color, DevCard::YearOfPlenty as usize);

        let bank_before = state.vector[BANK_RESOURCE_SLICE].to_vec();
        let hand_before = state.get_player_hand(color).to_vec();

        // Play year of plenty for wood and brick
        state.play_year_of_plenty(color, (0, Some(1)));

        // Verify card was removed from hand
        assert_eq!(
            state.get_dev_card_count(color, DevCard::YearOfPlenty as usize),
            0
        );

        // Verify card was marked as played
        assert_eq!(
            state.get_played_dev_card_count(color, DevCard::YearOfPlenty as usize),
            1
        );
        assert_eq!(state.vector[HAS_PLAYED_DEV_CARD], 1);

        // Verify resources were transferred
        assert_eq!(state.vector[BANK_RESOURCE_SLICE][0], bank_before[0] - 1);
        assert_eq!(state.vector[BANK_RESOURCE_SLICE][1], bank_before[1] - 1);
        assert_eq!(state.get_player_hand(color)[0], hand_before[0] + 1);
        assert_eq!(state.get_player_hand(color)[1], hand_before[1] + 1);
    }

    #[test]
    fn test_play_monopoly() {
        let mut state = State::new_base();
        let monopolist_color = state.get_current_color();

        // Give player a monopoly card
        state.add_dev_card(monopolist_color, DevCard::Monopoly as usize);

        // Give other players some wood
        for other_color in 0..state.get_num_players() {
            if other_color != monopolist_color {
                state.get_mut_player_hand(other_color)[0] = 3;
            }
        }

        let initial_wood = state.get_player_hand(monopolist_color)[0];
        let expected_stolen = 3 * (state.get_num_players() - 1) as u8; // 3 wood from each other player

        // Play monopoly on wood (resource index 0)
        state.play_monopoly(monopolist_color, 0);

        assert_eq!(
            state.get_dev_card_count(monopolist_color, DevCard::Monopoly as usize),
            0
        );
        assert_eq!(
            state.get_played_dev_card_count(monopolist_color, DevCard::Monopoly as usize),
            1
        );
        assert_eq!(state.vector[HAS_PLAYED_DEV_CARD], 1);
        assert_eq!(
            state.get_player_hand(monopolist_color)[0],
            initial_wood + expected_stolen
        );

        // Verify other players lost their wood
        for other_color in 0..state.get_num_players() {
            if other_color != monopolist_color {
                assert_eq!(state.get_player_hand(other_color)[0], 0);
            }
        }
    }

    #[test]
    fn test_play_road_building() {
        let mut state = State::new_base();
        let color = state.get_current_color();

        // Give player a road building card
        state.add_dev_card(color, DevCard::RoadBuilding as usize);
        assert_eq!(
            state.get_dev_card_count(color, DevCard::RoadBuilding as usize),
            1
        );
        assert_eq!(
            state.get_played_dev_card_count(color, DevCard::RoadBuilding as usize),
            0
        );
        assert_eq!(state.vector[HAS_PLAYED_DEV_CARD], 0);

        // Play road building card
        state.play_road_building(color);

        // Verify card was removed from hand
        assert_eq!(
            state.get_dev_card_count(color, DevCard::RoadBuilding as usize),
            0
        );

        // Verify card was marked as played
        assert_eq!(
            state.get_played_dev_card_count(color, DevCard::RoadBuilding as usize),
            1
        );
        assert_eq!(state.vector[HAS_PLAYED_DEV_CARD], 1);

        // Verify state was set for free roads
        assert_eq!(state.vector[IS_BUILDING_ROAD_INDEX], 1);
        assert_eq!(state.vector[FREE_ROADS_AVAILABLE_INDEX], 2);
    }

    #[test]
    fn test_road_building_full_flow() {
        let mut state = State::new_base();
        let color = state.get_current_color();

        // Give player a road building card and build initial settlement
        state.add_dev_card(color, DevCard::RoadBuilding as usize);
        state.build_settlement(color, 0);

        // Initial state
        assert_eq!(state.vector[FREE_ROADS_AVAILABLE_INDEX], 0);
        assert!(!state.is_road_building());

        // Play road building card
        state.play_road_building(color);
        assert_eq!(state.vector[FREE_ROADS_AVAILABLE_INDEX], 2);
        assert!(state.is_road_building());

        // Build first free road
        state.build_road(color, (0, 1));
        assert_eq!(state.vector[FREE_ROADS_AVAILABLE_INDEX], 1);
        assert!(state.is_road_building()); // Still in road building mode

        // Build second free road
        state.build_road(color, (1, 2));
        assert_eq!(state.vector[FREE_ROADS_AVAILABLE_INDEX], 0);
        assert!(!state.is_road_building()); // No longer in road building mode

        // Verify roads were built and are owned by player
        assert_eq!(state.roads.get(&(0, 1)), Some(&color));
        assert_eq!(state.roads.get(&(1, 2)), Some(&color));
        assert_eq!(state.roads_by_color[color as usize], 2);
    }

    #[test]
    fn test_road_building_turn_does_not_advance() {
        let mut state = State::new_base();
        let starting_color = state.get_current_color();

        // Set up a simple scenario: build settlement and give road building card
        state.build_settlement(starting_color, 0);
        state.add_dev_card(starting_color, DevCard::RoadBuilding as usize);

        // Manually set state to post-initial phase for road building
        state.vector[IS_INITIAL_BUILD_PHASE_INDEX] = 0;
        state.vector[HAS_ROLLED_INDEX] = 1;

        // Verify initial state
        assert_eq!(state.get_current_color(), starting_color);

        // Play road building card
        state.apply_action(Action::PlayRoadBuilding {
            color: starting_color,
        });

        // Verify turn hasn't advanced after playing Road Building
        assert_eq!(state.get_current_color(), starting_color);
        assert!(state.is_road_building());
        assert_eq!(state.vector[FREE_ROADS_AVAILABLE_INDEX], 2);

        // Build first road
        let actions = state.generate_playable_actions();
        if let Some(Action::BuildRoad { edge_id, .. }) = actions.first() {
            state.apply_action(Action::BuildRoad {
                color: starting_color,
                edge_id: *edge_id,
            });

            // CRITICAL: Verify turn STILL hasn't advanced after building first road
            assert_eq!(state.get_current_color(), starting_color);
            assert_eq!(state.vector[FREE_ROADS_AVAILABLE_INDEX], 1);
            assert!(state.is_road_building());

            // Build second road
            let actions = state.generate_playable_actions();
            if let Some(Action::BuildRoad { edge_id, .. }) = actions.first() {
                state.apply_action(Action::BuildRoad {
                    color: starting_color,
                    edge_id: *edge_id,
                });

                // CRITICAL: Verify turn STILL hasn't advanced after building second road
                assert_eq!(state.get_current_color(), starting_color);
                assert_eq!(state.vector[FREE_ROADS_AVAILABLE_INDEX], 0);
                assert!(!state.is_road_building());

                // Now player should be able to continue their turn normally
                let actions = state.generate_playable_actions();
                assert!(actions.iter().any(|a| matches!(a, Action::EndTurn { .. })));
            }
        }
    }

    #[test]
    fn test_maritime_trade_basic_rate() {
        let mut state = State::new_base();
        let color = state.get_current_color();

        state.get_mut_player_hand(color)[0] = 4; // 4 wood

        let initial_bank_brick = state.vector[BANK_RESOURCE_SLICE][1];

        state.apply_action(Action::MaritimeTrade {
            color,
            give: 0,
            take: 1,
            ratio: 4,
        });

        assert_eq!(state.get_player_hand(color)[0], 0);
        assert_eq!(state.get_player_hand(color)[1], 1);
        assert_eq!(state.vector[BANK_RESOURCE_SLICE][0], 19 + 4);
        assert_eq!(state.vector[BANK_RESOURCE_SLICE][1], initial_bank_brick - 1);
    }

    #[test]
    fn test_end_turn() {
        let mut state = State::new_base();
        let starting_color = state.get_current_color();
        let seating_order = state.get_seating_order().to_vec();

        state.vector[HAS_PLAYED_DEV_CARD] = 1;
        state.vector[HAS_ROLLED_INDEX] = 1;
        state.apply_action(Action::EndTurn {
            color: starting_color,
        });

        assert_eq!(state.vector[HAS_PLAYED_DEV_CARD], 0);
        assert_eq!(state.vector[HAS_ROLLED_INDEX], 0);

        assert_eq!(state.get_current_color(), seating_order[1]);

        for _ in 0..(state.get_num_players() - 1) {
            state.apply_action(Action::EndTurn {
                color: state.get_current_color(),
            });
        }

        assert_eq!(state.get_current_color(), starting_color);
    }

    #[test]
    fn test_update_connected_components() {
        // Create a test state
        let mut state = State::new_base();
        let color = 0; // Red player

        // Create two separate components
        let mut comp1 = HashSet::new();
        comp1.insert(0);
        comp1.insert(1);

        let mut comp2 = HashSet::new();
        comp2.insert(3);
        comp2.insert(4);

        state.connected_components.insert(color, vec![comp1, comp2]);

        // Add roads to connect the components
        state.roads.insert((1, 2), color);
        state.roads.insert((2, 3), color);

        // First update: Connect node 1 to node 2
        // Node 1 is in component index 0, node 2 is not in any component
        let _updated_component = state.update_connected_components(color, 1, 2, Some(0), None);

        // Verify node 2 was added to the first component
        let components = state.connected_components.get(&color).unwrap();
        assert_eq!(components.len(), 2, "Should still have two components");
        assert!(
            components[0].contains(&2),
            "First component should now contain node 2"
        );

        // Second update: Connect node 2 to node 3
        // After the first update, node 2 is now in component index 0
        // Node 3 is in component index 1
        let _updated_component = state.update_connected_components(color, 2, 3, Some(0), Some(1));

        // Verify that the components were merged
        let components = state.connected_components.get(&color).unwrap();
        assert_eq!(components.len(), 1, "Components should be merged into one");

        // Verify the merged component contains all nodes
        let merged = &components[0];
        assert!(
            merged.contains(&0),
            "Merged component should contain node 0"
        );
        assert!(
            merged.contains(&1),
            "Merged component should contain node 1"
        );
        assert!(
            merged.contains(&2),
            "Merged component should contain node 2"
        );
        assert!(
            merged.contains(&3),
            "Merged component should contain node 3"
        );
        assert!(
            merged.contains(&4),
            "Merged component should contain node 4"
        );
    }
}

use crate::enums::{
    Action as EnumAction, DevCard, GameConfiguration, MapType, Resource as EnumResource,
};
use crate::global_state::GlobalState;
use crate::map_instance::{Direction, EdgeRef, LandTile, MapInstance, NodeRef, PortTile, Tile};
use crate::map_template::Coordinate as CubeCoordinate;
// REMOVED: NodeDirection import - no longer needed
use crate::state::{BuildingType, State};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// (tile_id, resource_name, number)
type NodeTileAdjacency = (u8, Option<String>, Option<u8>);
type NodeAdjacencyMap = HashMap<u8, Vec<NodeTileAdjacency>>;
use std::sync::Arc;
use uuid;

// Use EnumAction instead of defining GameAction
pub type GameAction = EnumAction;

// Game state enum to track the current state of the game
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GameState {
    Setup,
    Active,
    Finished { winner: String },
}

// A serializable coordinate for frontend use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

// A serializable tile for frontend use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileData {
    pub resource: Option<String>,
    pub number: Option<u8>,
}

// A tile with its position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilePosition {
    pub coordinate: Coordinate,
    pub tile: TileData,
}

// A port for trading resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    pub resource: Option<String>,
    pub ratio: u8,
    pub direction: String,
}

// A port with its position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortPosition {
    pub coordinate: Coordinate,
    pub port: Port,
}

// A node (intersection) on the board
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub building: Option<String>,
    pub color: Option<String>,
    pub tile_coordinate: Coordinate,
    pub direction: String,
}

// REMOVED: NodeAbsoluteCoordinate struct - no longer needed

// An edge (path) on the board
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub color: Option<String>,
    pub node1_id: u8,
    pub node2_id: u8,
    pub tile_coordinate: Coordinate,
    pub direction: String,
}

// The game board representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameBoard {
    pub tiles: Vec<TilePosition>,
    pub ports: Vec<PortPosition>,
    pub nodes: HashMap<String, Node>,
    pub edges: HashMap<String, Edge>,
    pub robber_coordinate: Option<Coordinate>,
}

// Player information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub color: String,
    pub resources: HashMap<EnumResource, u32>,
    pub dev_cards: Vec<DevCard>,
    pub knights_played: u32,
    pub victory_points: u32,
    pub longest_road: bool,
    pub largest_army: bool,
}

// Action tracking for the game log - format: [player_color, action_type, action_data]
pub type ActionLog = Vec<serde_json::Value>;

// The unified Game struct that replaces both Game enum and GameView
#[derive(Debug, Clone, Deserialize)]
pub struct Game {
    pub id: String,
    pub players: Vec<Player>,
    pub game_state: GameState,
    pub current_player_index: usize,
    pub dice_rolled: bool,
    pub turns: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_dice_roll: Option<[u8; 2]>,
    pub actions: ActionLog, // Track all actions for the game log
    // Frontend compatibility fields
    pub current_playable_actions: Vec<crate::actions::PlayerAction>,
    pub is_initial_build_phase: bool,
    pub current_color: Option<String>,
    pub current_prompt: Option<String>,
    pub bot_colors: Vec<String>, // Colors of bot players for frontend identification
    #[serde(skip)]
    pub state: Option<State>, // Internal game logic state, skipped in serialization
}

// Helper function to convert from template coordinate to serializable coordinate
fn convert_coordinate(coord: CubeCoordinate) -> Coordinate {
    Coordinate {
        x: coord.0 as i32,
        y: coord.1 as i32,
        z: coord.2 as i32,
    }
}

// REMOVED: node_ref_to_direction function - no longer needed

// REMOVED: node_coordinate_to_absolute helper - no longer needed

// REMOVED: find_node_absolute_coordinate function - no longer needed

// Helper function to convert from map_instance::Tile to TilePosition
fn convert_land_tile(coord: CubeCoordinate, land_tile: &LandTile) -> TilePosition {
    let resource = match land_tile.resource {
        Some(EnumResource::Wood) => Some("wood".to_string()),
        Some(EnumResource::Brick) => Some("brick".to_string()),
        Some(EnumResource::Sheep) => Some("sheep".to_string()),
        Some(EnumResource::Wheat) => Some("wheat".to_string()),
        Some(EnumResource::Ore) => Some("ore".to_string()),
        None => None,
    };

    TilePosition {
        coordinate: convert_coordinate(coord),
        tile: TileData {
            resource,
            number: land_tile.number,
        },
    }
}

// Helper function to convert from map_instance::PortTile to PortPosition
fn convert_port_tile(coord: CubeCoordinate, port_tile: &PortTile) -> PortPosition {
    let resource = match port_tile.resource {
        Some(EnumResource::Wood) => Some("wood".to_string()),
        Some(EnumResource::Brick) => Some("brick".to_string()),
        Some(EnumResource::Sheep) => Some("sheep".to_string()),
        Some(EnumResource::Wheat) => Some("wheat".to_string()),
        Some(EnumResource::Ore) => Some("ore".to_string()),
        None => None,
    };

    let direction = match port_tile.direction {
        Direction::NorthWest => "NW",
        Direction::NorthEast => "NE",
        Direction::East => "E",
        Direction::SouthEast => "SE",
        Direction::SouthWest => "SW",
        Direction::West => "W",
        Direction::North => "N",
        Direction::South => "S",
    };

    // Check if it's a resource port before using the resource value
    let is_resource_port = resource.is_some();

    PortPosition {
        coordinate: convert_coordinate(coord),
        port: Port {
            resource,
            ratio: if is_resource_port { 2 } else { 3 },
            direction: direction.to_string(),
        },
    }
}

// Create a new Player struct from the given information
pub fn create_player(id: String, name: String, color: String) -> Player {
    Player {
        id,
        name,
        color,
        resources: HashMap::new(),
        dev_cards: Vec::new(), // Will be populated from internal state via update_players_from_state
        knights_played: 0,
        victory_points: 0,
        longest_road: false,
        largest_army: false,
    }
}

// Create a new Game with default settings
pub fn create_game(id: String, player_names: Vec<String>) -> Game {
    let colors = ["red", "blue", "white", "orange"];

    let players = player_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let player_id = format!("player_{i}");
            let color = colors[i % colors.len()].to_string();
            create_player(player_id, name.clone(), color)
        })
        .collect();

    // Create configuration for the game state
    let config = GameConfiguration {
        discard_limit: 7,
        vps_to_win: 10,
        map_type: MapType::Base,
        num_players: player_names.len() as u8,
        max_ticks: 100, // Reasonable default
    };

    // Create map instance for the game
    let global_state = GlobalState::new();
    let map_instance = MapInstance::new(
        &global_state.base_map_template,
        &global_state.dice_probas,
        0, // Use a fixed seed for predictable board generation
    );

    // Create the State object first (it owns the canonical map)
    let state = State::new(Arc::new(config), Arc::new(map_instance));

    // Create the Game object (board is generated on-demand via get_board())
    let mut game = Game {
        id,
        players,
        game_state: GameState::Setup,
        current_player_index: 0,
        dice_rolled: false,
        turns: 0,
        current_dice_roll: None,
        actions: Vec::new(), // Initialize empty actions log
        current_playable_actions: Vec::new(),
        is_initial_build_phase: true,
        current_color: None,
        current_prompt: None,
        bot_colors: Vec::new(),
        state: Some(state),
    };

    // Update metadata from the initial state
    game.update_metadata_from_state();

    game
}

// Game simulation for bot play
pub fn simulate_bot_game(num_players: u8) -> Game {
    let player_names = (0..num_players).map(|i| format!("Bot {}", i + 1)).collect();
    let game_id = format!("sim_{}", uuid::Uuid::new_v4());
    Game::new(game_id, player_names)
}

// Initial setup for a game against Catanatron
pub fn start_human_vs_catanatron(human_name: String, num_bots: u8) -> Game {
    println!("🎮 DEBUG start_human_vs_catanatron:");
    println!("  - Human name: {human_name}");
    println!("  - Number of bots: {num_bots}");

    let mut player_names = vec![human_name.clone()];

    for i in 0..num_bots {
        player_names.push(format!("Bot {}", i + 1));
    }

    println!("  - Player names: {player_names:?}");

    let game_id = format!("hvs_{}", uuid::Uuid::new_v4());
    println!("  - Game ID: {game_id}");

    let mut game = Game::new(game_id, player_names);

    // Set bot_colors - all players except the first one (human) are bots
    game.bot_colors = game
        .players
        .iter()
        .skip(1) // Skip the first player (human)
        .map(|p| p.color.clone())
        .collect();

    println!("  - Bot colors: {:?}", game.bot_colors);
    println!("  - All players:");
    for (i, player) in game.players.iter().enumerate() {
        let is_bot = game.bot_colors.contains(&player.color);
        println!(
            "    - Player {}: {} (color: {}, is_bot: {})",
            i, player.name, player.color, is_bot
        );
    }

    // Update metadata from state
    println!("  - Calling update_metadata_from_state...");
    game.update_metadata_from_state();

    println!("🎮 END start_human_vs_catanatron debug\n");

    game
}

impl Game {
    pub fn new(id: String, player_names: Vec<String>) -> Self {
        let colors = ["red", "blue", "white", "orange"];

        let players = player_names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let player_id = format!("player_{i}");
                let color = colors[i % colors.len()].to_string();
                create_player(player_id, name.clone(), color)
            })
            .collect();

        // Create configuration for the game state
        let config = GameConfiguration {
            discard_limit: 7,
            vps_to_win: 10,
            map_type: MapType::Base,
            num_players: player_names.len() as u8,
            max_ticks: 100, // Reasonable default
        };

        // Create map instance for the game
        let global_state = GlobalState::new();
        let map_instance = MapInstance::new(
            &global_state.base_map_template,
            &global_state.dice_probas,
            0, // Use a fixed seed for predictable board generation
        );

        // Create the State object first (it owns the canonical map)
        let mut state = State::new(Arc::new(config), Arc::new(map_instance));

        // For testing: Add dev cards to each player in the internal state
        for player_idx in 0..player_names.len() {
            let color = player_idx as u8;
            // Add 2 of each dev card type to the internal state
            state.add_dev_card(color, DevCard::Knight as usize);
            state.add_dev_card(color, DevCard::Knight as usize);
            state.add_dev_card(color, DevCard::VictoryPoint as usize);
            state.add_dev_card(color, DevCard::VictoryPoint as usize);
            state.add_dev_card(color, DevCard::Monopoly as usize);
            state.add_dev_card(color, DevCard::Monopoly as usize);
            state.add_dev_card(color, DevCard::YearOfPlenty as usize);
            state.add_dev_card(color, DevCard::YearOfPlenty as usize);
            state.add_dev_card(color, DevCard::RoadBuilding as usize);
            state.add_dev_card(color, DevCard::RoadBuilding as usize);
        }

        // Create the Game object (board is generated on-demand via get_board())
        let mut game = Game {
            id,
            players,
            game_state: GameState::Setup,
            current_player_index: 0,
            dice_rolled: false,
            turns: 0,
            current_dice_roll: None,
            actions: Vec::new(), // Initialize empty actions log
            current_playable_actions: Vec::new(),
            is_initial_build_phase: true,
            current_color: None,
            current_prompt: None,
            bot_colors: Vec::new(),
            state: Some(state),
        };

        // Update metadata from the initial state
        game.update_metadata_from_state();

        game
    }

    /// Generate board data on-demand from the current state
    pub fn get_board(&self) -> GameBoard {
        if let Some(state) = &self.state {
            generate_board_from_state(state, state.get_map_instance())
        } else {
            // Fallback empty board if no state (shouldn't happen)
            GameBoard {
                tiles: Vec::new(),
                ports: Vec::new(),
                nodes: HashMap::new(),
                edges: HashMap::new(),
                robber_coordinate: None,
            }
        }
    }

    // Process an action on the game
    pub fn process_action(&mut self, player_id: &str, action: GameAction) -> Result<(), String> {
        // Check if the game is in a valid state for actions
        match self.game_state {
            GameState::Finished { .. } => return Err("Game already finished".into()),
            GameState::Setup | GameState::Active => {} // Allow actions in both Setup and Active phases
        }

        // Find the player's color index first
        let player_index = self.players.iter().position(|p| p.id == player_id);

        if player_index.is_none() {
            return Err("Player not found".into());
        }

        let player_index = player_index.unwrap();
        let _color_idx = player_index as u8; // Prefix with underscore to indicate it's unused

        // Get player color for logging (clone to avoid borrowing issues)
        let player_color = self.players[player_index].color.clone();

        // Apply the action and get updated state info
        let (new_current_player, new_dice_rolled) = {
            // Scope the mutable borrow of state
            let state = match &mut self.state {
                Some(state) => state,
                None => return Err("Game state is missing".into()),
            };

            // Apply the action directly since GameAction is now an alias for EnumAction
            state.apply_action(action);

            // Update frontend players from the state
            update_players_from_state(&mut self.players, state);

            // Get current player and dice rolled status
            (
                state.get_current_color() as usize,
                state.current_player_rolled(),
            )
        };

        // Now we can safely update other fields
        self.current_player_index = new_current_player;
        self.dice_rolled = new_dice_rolled;

        // Board representation is generated on-demand via get_board() - no update needed

        // Log the action for the game log - format: [player_color, action_type, action_data]
        let action_log_entry = {
            let (action_type, action_data) = match &action {
                EnumAction::BuildSettlement { node_id, .. } => {
                    ("BuildSettlement", serde_json::json!(node_id))
                }
                EnumAction::BuildCity { node_id, .. } => ("BuildCity", serde_json::json!(node_id)),
                EnumAction::BuildRoad { edge_id, .. } => ("BuildRoad", serde_json::json!(edge_id)),
                EnumAction::BuyDevelopmentCard { .. } => {
                    ("BuyDevelopmentCard", serde_json::Value::Null)
                }
                EnumAction::PlayKnight { .. } => ("PlayKnight", serde_json::Value::Null),
                EnumAction::PlayMonopoly { resource, .. } => {
                    ("PlayMonopoly", serde_json::json!(resource))
                }
                EnumAction::PlayYearOfPlenty { resources, .. } => {
                    ("PlayYearOfPlenty", serde_json::json!(resources))
                }
                EnumAction::PlayRoadBuilding { .. } => {
                    ("PlayRoadBuilding", serde_json::Value::Null)
                }
                EnumAction::MoveRobber {
                    coordinate,
                    victim_opt,
                    ..
                } => {
                    let mut data = serde_json::json!([coordinate.0, coordinate.1, coordinate.2]);
                    if let Some(victim) = victim_opt {
                        if let serde_json::Value::Array(ref mut arr) = data {
                            arr.push(serde_json::json!(victim));
                        }
                    }
                    ("MoveRobber", data)
                }
                EnumAction::MaritimeTrade {
                    give, take, ratio, ..
                } => ("MaritimeTrade", serde_json::json!([give, take, ratio])),
                EnumAction::EndTurn { .. } => ("EndTurn", serde_json::Value::Null),
                EnumAction::Roll { .. } => {
                    // Simple approach: get dice from state after action is applied
                    let dice_data = if let Some(ref state) = self.state {
                        if let Some((die1, die2)) = state.get_last_dice_roll() {
                            let total = die1 + die2;
                            serde_json::json!(total)
                        } else {
                            serde_json::Value::Null
                        }
                    } else {
                        serde_json::Value::Null
                    };
                    ("Roll", dice_data)
                }
                EnumAction::Discard { .. } => ("Discard", serde_json::Value::Null),
                _ => ("Unknown", serde_json::Value::Null),
            };

            serde_json::json!([player_color.to_uppercase(), action_type, action_data])
        };

        self.actions.push(action_log_entry);

        // Sync frontend game_state with internal state phase transitions
        // Check if we should transition from Setup to Active phase
        if self.game_state == GameState::Setup {
            if let Some(ref state) = self.state {
                if !state.is_initial_build_phase() {
                    println!("🎮 Game {}: Transitioning from Setup to Active phase - initial build phase completed", self.id);
                    self.game_state = GameState::Active;
                }
            }
        }

        // Check if the game is finished
        let winner_color = self.state.as_ref().and_then(|s| s.winner());
        if let Some(winner_color) = winner_color {
            let winner_name = if (winner_color as usize) < self.players.len() {
                self.players[winner_color as usize].name.clone()
            } else {
                "Unknown".to_string()
            };

            // Update the game state to finished
            self.game_state = GameState::Finished {
                winner: winner_name,
            };
        }

        Ok(())
    }

    /// Check if the game is in the initial build phase from the internal state
    pub fn is_initial_build_phase(&self) -> bool {
        self.state
            .as_ref()
            .map(|s| s.is_initial_build_phase())
            .unwrap_or(true) // Default to true if no state available
    }

    /// Update game metadata from the internal state
    pub fn update_metadata_from_state(&mut self) {
        if let Some(state) = &self.state {
            // DEBUG: Log essential information about the current state
            let current_color_index = state.get_current_color();
            let is_initial_phase = state.is_initial_build_phase();
            let action_prompt = state.get_action_prompt();

            println!("🔍 DEBUG update_metadata_from_state:");
            println!("  - Current color index: {current_color_index}");
            println!("  - Is initial build phase: {is_initial_phase}");
            println!("  - Action prompt: {action_prompt:?}");

            // Update current_playable_actions
            let playable_actions = state.generate_playable_actions();
            println!("  - Generated {} playable actions", playable_actions.len());

            // Debug: Log just the first action for verification
            if !playable_actions.is_empty() {
                println!("    - First action: {:?}", playable_actions[0]);
            }

            self.current_playable_actions = playable_actions
                .iter()
                .map(|action| crate::actions::PlayerAction::from(*action))
                .collect();

            println!(
                "  - Converted to {} PlayerActions",
                self.current_playable_actions.len()
            );

            // Update current_color
            self.current_color = Some(match current_color_index {
                0 => "RED".to_string(),
                1 => "BLUE".to_string(),
                2 => "WHITE".to_string(),
                3 => "ORANGE".to_string(),
                _ => format!("PLAYER_{current_color_index}"),
            });

            // Update is_initial_build_phase
            self.is_initial_build_phase = is_initial_phase;

            // Update current_prompt based on action prompt
            use crate::enums::ActionPrompt;
            self.current_prompt = Some(match action_prompt {
                ActionPrompt::BuildInitialSettlement => "BUILD_INITIAL_SETTLEMENT".to_string(),
                ActionPrompt::BuildInitialRoad => "BUILD_INITIAL_ROAD".to_string(),
                ActionPrompt::PlayTurn => "PLAY_TURN".to_string(),
                ActionPrompt::Discard => "DISCARD".to_string(),
                ActionPrompt::MoveRobber => "MOVE_ROBBER".to_string(),
                ActionPrompt::DecideTrade => "DECIDE_TRADE".to_string(),
                ActionPrompt::DecideAcceptees => "DECIDE_ACCEPTEES".to_string(),
            });

            println!(
                "  - Current color: {:?}, Current prompt: {:?}",
                self.current_color, self.current_prompt
            );
            println!("🔍 END update_metadata_from_state debug\n");
        } else {
            println!("❌ update_metadata_from_state: No internal state available!");
        }
    }

    /// Helper method to verify frontend and backend state consistency
    pub fn verify_state_consistency(&self) -> bool {
        match (&self.game_state, self.state.as_ref()) {
            (GameState::Setup, Some(state)) => state.is_initial_build_phase(),
            (GameState::Active, Some(state)) => {
                !state.is_initial_build_phase() && state.winner().is_none()
            }
            (GameState::Finished { .. }, Some(state)) => state.winner().is_some(),
            _ => false, // Inconsistent if no internal state
        }
    }

    /// Get all adjacent tiles for a specific node ID using backend's authoritative adjacency calculation
    pub fn get_node_adjacent_tiles(
        &self,
        node_id: u8,
    ) -> Option<Vec<NodeTileAdjacency>> {
        if let Some(state) = &self.state {
            let map_instance = state.get_map_instance();
            if let Some(adjacent_tiles) = map_instance.get_adjacent_tiles(node_id) {
                let result: Vec<NodeTileAdjacency> = adjacent_tiles
                    .iter()
                    .map(|tile| {
                        let resource_str = tile.resource.map(|r| match r {
                            crate::enums::Resource::Wood => "wood".to_string(),
                            crate::enums::Resource::Brick => "brick".to_string(),
                            crate::enums::Resource::Sheep => "sheep".to_string(),
                            crate::enums::Resource::Wheat => "wheat".to_string(),
                            crate::enums::Resource::Ore => "ore".to_string(),
                        });
                        (tile.id, resource_str, tile.number)
                    })
                    .collect();
                Some(result)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get complete node-to-tile adjacency mapping for all nodes using backend's authoritative calculations
    /// Returns HashMap<NodeId, Vec<(TileId, Resource, Number)>>
    pub fn get_all_node_tile_adjacencies(
        &self,
    ) -> NodeAdjacencyMap {
        let mut adjacencies: NodeAdjacencyMap = HashMap::new();

        if let Some(state) = &self.state {
            let map_instance = state.get_map_instance();
            // Iterate through all land nodes to build complete mapping
            for &node_id in map_instance.land_nodes() {
                if let Some(adjacent_tiles) = self.get_node_adjacent_tiles(node_id) {
                    adjacencies.insert(node_id, adjacent_tiles);
                }
            }
        }

        adjacencies
    }
}

// Custom Serialize implementation to include board field generated on-demand
impl Serialize for Game {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("Game", 14)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("players", &self.players)?;
        state.serialize_field("game_state", &self.game_state)?;
        state.serialize_field("current_player_index", &self.current_player_index)?;
        state.serialize_field("dice_rolled", &self.dice_rolled)?;
        state.serialize_field("turns", &self.turns)?;
        state.serialize_field("current_dice_roll", &self.current_dice_roll)?;
        state.serialize_field("actions", &self.actions)?;
        state.serialize_field("current_playable_actions", &self.current_playable_actions)?;
        state.serialize_field("is_initial_build_phase", &self.is_initial_build_phase)?;
        state.serialize_field("current_color", &self.current_color)?;
        state.serialize_field("current_prompt", &self.current_prompt)?;
        state.serialize_field("bot_colors", &self.bot_colors)?;

        // Generate board on-demand during serialization
        let board = self.get_board();
        state.serialize_field("board", &board)?;

        state.end()
    }
}

// Generate a serializable game board from the State and MapInstance
fn generate_board_from_state(state: &State, map_instance: &MapInstance) -> GameBoard {
    let mut tiles = Vec::new();
    let mut ports = Vec::new();
    let mut nodes = HashMap::new();
    let mut edges = HashMap::new();
    let mut robber_coordinate = None;

    // Collect all node information in a deterministic way
    let mut node_info: std::collections::BTreeMap<u8, (Coordinate, String)> =
        std::collections::BTreeMap::new();

    // Get robber position from state
    let robber_tile_id = state.get_robber_tile();

    // Sort tiles by coordinate for deterministic iteration
    let mut sorted_tiles: Vec<_> = map_instance.tiles.iter().collect();
    sorted_tiles.sort_by_key(|(coord, _)| (coord.0, coord.1, coord.2));

    // Process all tiles in deterministic order
    for (&coordinate, tile) in sorted_tiles {
        match tile {
            Tile::Land(land_tile) => {
                // Convert land tile to frontend format
                tiles.push(convert_land_tile(coordinate, land_tile));

                // Track the robber location
                if land_tile.id == robber_tile_id {
                    robber_coordinate = Some(convert_coordinate(coordinate));
                }

                // Collect node information (store the first occurrence for deterministic positioning)
                for (&node_ref, &node_id) in &land_tile.hexagon.nodes {
                    // Only store if not already stored (ensures deterministic positioning)
                    node_info.entry(node_id).or_insert_with(|| {
                        let direction = match node_ref {
                            NodeRef::North => "N",
                            NodeRef::NorthEast => "NE",
                            NodeRef::SouthEast => "SE",
                            NodeRef::South => "S",
                            NodeRef::SouthWest => "SW",
                            NodeRef::NorthWest => "NW",
                        };

                        (convert_coordinate(coordinate), direction.to_string())
                    });
                }

                // Generate edges for this tile
                for (&edge_ref, &(node1, node2)) in &land_tile.hexagon.edges {
                    let direction = match edge_ref {
                        EdgeRef::North => "N",
                        EdgeRef::SouthEast => "SE",
                        EdgeRef::SouthWest => "SW",
                        EdgeRef::South => "S",
                        EdgeRef::NorthWest => "NW",
                        EdgeRef::NorthEast => "NE",
                    };

                    // Use edge ID format that shows both connected nodes
                    let edge_id_str = format!("e{}_{}", node1.min(node2), node1.max(node2));

                    // Get road owner using order-agnostic helper
                    let edge_color_idx = state.get_edge_owner((node1, node2));
                    let edge_color = edge_color_idx.map(|color_idx| match color_idx {
                        0 => "red".to_string(),
                        1 => "blue".to_string(),
                        2 => "white".to_string(),
                        3 => "orange".to_string(),
                        _ => "unknown".to_string(),
                    });

                    edges.insert(
                        edge_id_str,
                        Edge {
                            color: edge_color,
                            node1_id: node1,
                            node2_id: node2,
                            tile_coordinate: convert_coordinate(coordinate),
                            direction: direction.to_string(),
                        },
                    );
                }
            }
            Tile::Port(port_tile) => {
                // Convert port tile to frontend format
                ports.push(convert_port_tile(coordinate, port_tile));
            }
            Tile::Water(_) => {
                // Skip water tiles - we don't need to send them to the frontend
            }
        }
    }

    // Now process nodes in deterministic order (sorted by node_id)
    for (node_id, (tile_coordinate, direction)) in node_info {
        // Use just the node ID as the key to ensure uniqueness
        let node_id_str = format!("n{node_id}");

        // Get building info using public state methods
        let building_type_opt = state.get_building_type(node_id);
        let building_color_idx_opt = state.get_node_color(node_id);

        let building_type = match building_type_opt {
            Some(BuildingType::Settlement) => Some("settlement".to_string()),
            Some(BuildingType::City) => Some("city".to_string()),
            None => None,
        };

        let building_color = building_color_idx_opt.map(|color_idx| match color_idx {
            0 => "red".to_string(),
            1 => "blue".to_string(),
            2 => "white".to_string(),
            3 => "orange".to_string(),
            _ => "unknown".to_string(), // Handle unexpected color index
        });

        nodes.insert(
            node_id_str,
            Node {
                building: building_type,
                color: building_color,
                tile_coordinate,
                direction,
            },
        );
    }

    GameBoard {
        tiles,
        ports,
        nodes,
        edges,
        robber_coordinate,
    }
}

// Update frontend players from the State
fn update_players_from_state(players: &mut [Player], state: &State) {
    for (i, player) in players.iter_mut().enumerate() {
        let color_idx = i as u8;

        // Get resources
        let player_hand = state.get_player_hand(color_idx);
        if player_hand.len() >= 5 {
            player.resources.clear();
            player
                .resources
                .insert(EnumResource::Wood, player_hand[0] as u32);
            player
                .resources
                .insert(EnumResource::Brick, player_hand[1] as u32);
            player
                .resources
                .insert(EnumResource::Sheep, player_hand[2] as u32);
            player
                .resources
                .insert(EnumResource::Wheat, player_hand[3] as u32);
            player
                .resources
                .insert(EnumResource::Ore, player_hand[4] as u32);
        }

        // Get development cards
        player.dev_cards.clear();
        let player_dev_hand = state.get_player_devhand(color_idx);
        if player_dev_hand.len() >= 5 {
            for _ in 0..(player_dev_hand[0]) {
                player.dev_cards.push(DevCard::Knight);
            }
            for _ in 0..(player_dev_hand[1]) {
                player.dev_cards.push(DevCard::VictoryPoint);
            }
            for _ in 0..(player_dev_hand[2]) {
                player.dev_cards.push(DevCard::RoadBuilding);
            }
            for _ in 0..(player_dev_hand[3]) {
                player.dev_cards.push(DevCard::YearOfPlenty);
            }
            for _ in 0..(player_dev_hand[4]) {
                player.dev_cards.push(DevCard::Monopoly);
            }
        }

        // Update player stats
        player.knights_played = state.get_played_dev_card_count(color_idx, 0) as u32;
        player.victory_points = state.get_actual_victory_points(color_idx) as u32;

        // Update special awards - defaults used since direct access may not be available
        player.longest_road = false; // TODO: Get from state
        player.largest_army = false; // TODO: Get from state
    }
}

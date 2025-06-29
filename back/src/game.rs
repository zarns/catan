use crate::enums::{
    Action as EnumAction, DevCard, GameConfiguration, MapType, Resource as EnumResource,
};
use crate::global_state::GlobalState;
use crate::map_instance::{Direction, EdgeRef, LandTile, MapInstance, NodeRef, PortTile, Tile};
use crate::map_template::Coordinate as CubeCoordinate;
use crate::node_coordinates::{calculate_node_coordinate, NodeCoordinate, NodeDirection};
use crate::state::{BuildingType, State};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    // DEPRECATED: Keep for backward compatibility, will be removed in future version
    pub tile_coordinate: Coordinate,
    pub direction: String,
    // NEW: Absolute coordinates for deterministic positioning
    pub absolute_coordinate: NodeAbsoluteCoordinate,
}

// Absolute coordinate for nodes in normalized hexagonal space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAbsoluteCoordinate {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

// An edge (path) on the board
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub color: Option<String>,
    pub node1_id: u8,
    pub node2_id: u8,
    // DEPRECATED: Keep for backward compatibility, will be removed in future version
    pub tile_coordinate: Coordinate,
    pub direction: String,
    // NEW: Absolute coordinates of the two connected nodes for deterministic positioning
    pub node1_absolute_coordinate: NodeAbsoluteCoordinate,
    pub node2_absolute_coordinate: NodeAbsoluteCoordinate,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub players: Vec<Player>,
    pub game_state: GameState,
    pub board: GameBoard,
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

// Helper function to convert NodeRef to NodeDirection
fn node_ref_to_direction(node_ref: NodeRef) -> NodeDirection {
    match node_ref {
        NodeRef::North => NodeDirection::North,
        NodeRef::NorthEast => NodeDirection::NorthEast,
        NodeRef::SouthEast => NodeDirection::SouthEast,
        NodeRef::South => NodeDirection::South,
        NodeRef::SouthWest => NodeDirection::SouthWest,
        NodeRef::NorthWest => NodeDirection::NorthWest,
    }
}

// Helper function to convert NodeCoordinate to NodeAbsoluteCoordinate
fn node_coordinate_to_absolute(coord: NodeCoordinate) -> NodeAbsoluteCoordinate {
    NodeAbsoluteCoordinate {
        x: coord.x,
        y: coord.y,
        z: coord.z,
    }
}

// Helper function to find a node's absolute coordinate from the map_instance
fn find_node_absolute_coordinate(
    node_id: u8,
    map_instance: &MapInstance,
) -> Option<NodeAbsoluteCoordinate> {
    // Search through all tiles to find this node_id and calculate its absolute coordinate
    for (&tile_coord, tile) in &map_instance.tiles {
        if let Tile::Land(land_tile) = tile {
            for (&node_ref, &found_node_id) in &land_tile.hexagon.nodes {
                if found_node_id == node_id {
                    // Found the node! Calculate its absolute coordinate
                    let node_direction = node_ref_to_direction(node_ref);
                    let absolute_coord = calculate_node_coordinate(tile_coord, node_direction);
                    return Some(node_coordinate_to_absolute(absolute_coord));
                }
            }
        }
    }
    None
}

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

// Generate a serializable game board from the MapInstance
pub fn generate_board_from_template(map_instance: &MapInstance) -> GameBoard {
    let mut tiles = Vec::new();
    let mut ports = Vec::new();
    let mut nodes = HashMap::new();
    let mut edges = HashMap::new();
    let mut robber_coordinate = None;

    // Process all tiles
    for (&coordinate, tile) in &map_instance.tiles {
        match tile {
            Tile::Land(land_tile) => {
                // Convert land tile to frontend format
                tiles.push(convert_land_tile(coordinate, land_tile));

                // Track the robber location (desert tile or first tile)
                if land_tile.resource.is_none() || robber_coordinate.is_none() {
                    robber_coordinate = Some(convert_coordinate(coordinate));
                }

                // Generate nodes for this tile
                for (&node_ref, &node_id) in &land_tile.hexagon.nodes {
                    let direction = match node_ref {
                        NodeRef::North => "N",
                        NodeRef::NorthEast => "NE",
                        NodeRef::SouthEast => "SE",
                        NodeRef::South => "S",
                        NodeRef::SouthWest => "SW",
                        NodeRef::NorthWest => "NW",
                    };

                    // Use a more compact node ID format: n{id}_{direction}
                    let node_id_str = format!("n{}_{}", node_id, direction);

                    // Calculate absolute coordinate for this node
                    let node_direction = node_ref_to_direction(node_ref);
                    let absolute_coord = calculate_node_coordinate(coordinate, node_direction);

                    nodes.insert(
                        node_id_str,
                        Node {
                            building: None,
                            color: None,
                            tile_coordinate: convert_coordinate(coordinate),
                            direction: direction.to_string(),
                            absolute_coordinate: node_coordinate_to_absolute(absolute_coord),
                        },
                    );
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
                    let edge_id = format!("e{}_{}", node1.min(node2), node1.max(node2));

                    // Calculate absolute coordinates for the edge endpoints using actual node IDs
                    let node1_abs = find_node_absolute_coordinate(node1, map_instance).unwrap_or(
                        NodeAbsoluteCoordinate {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        },
                    );
                    let node2_abs = find_node_absolute_coordinate(node2, map_instance).unwrap_or(
                        NodeAbsoluteCoordinate {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        },
                    );

                    edges.insert(
                        edge_id,
                        Edge {
                            color: None,
                            node1_id: node1,
                            node2_id: node2,
                            tile_coordinate: convert_coordinate(coordinate),
                            direction: direction.to_string(),
                            node1_absolute_coordinate: node1_abs,
                            node2_absolute_coordinate: node2_abs,
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

    GameBoard {
        tiles,
        ports,
        nodes,
        edges,
        robber_coordinate,
    }
}

// Create a new Player struct from the given information
pub fn create_player(id: String, name: String, color: String) -> Player {
    Player {
        id,
        name,
        color,
        resources: HashMap::new(),
        dev_cards: Vec::new(),
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
            let player_id = format!("player_{}", i);
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

    // Create the State object
    let state = State::new(Arc::new(config), Arc::new(map_instance.clone()));

    // Create the Game object
    let mut game = Game {
        id,
        players,
        game_state: GameState::Setup,
        board: generate_board_from_template(&map_instance),
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
    println!("  - Human name: {}", human_name);
    println!("  - Number of bots: {}", num_bots);

    let mut player_names = vec![human_name.clone()];

    for i in 0..num_bots {
        player_names.push(format!("Bot {}", i + 1));
    }

    println!("  - Player names: {:?}", player_names);

    let game_id = format!("hvs_{}", uuid::Uuid::new_v4());
    println!("  - Game ID: {}", game_id);

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
                let player_id = format!("player_{}", i);
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

        // Create the State object
        let state = State::new(Arc::new(config), Arc::new(map_instance.clone()));

        // Create the Game object
        let mut game = Game {
            id,
            players,
            game_state: GameState::Setup,
            board: generate_board_from_template(&map_instance),
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

        // Update the board representation to reflect the new state
        self.update_board();

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

        // BUGFIX - Sync frontend game_state with internal state phase transitions
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

    // Update the game board from the current state
    pub fn update_board(&mut self) {
        if let Some(state) = &self.state {
            // Get the map instance
            let global_state = GlobalState::new();
            let map_instance = MapInstance::new(
                &global_state.base_map_template,
                &global_state.dice_probas,
                0, // Use a fixed seed for predictable board generation
            );

            // Update the board with buildings, roads, and robber
            self.board = generate_board_from_state(state, &map_instance);
        }
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
            println!("  - Current color index: {}", current_color_index);
            println!("  - Is initial build phase: {}", is_initial_phase);
            println!("  - Action prompt: {:?}", action_prompt);

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
                _ => format!("PLAYER_{}", current_color_index),
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

    // BUGFIX: Sort tiles by coordinate for deterministic iteration
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
                    if !node_info.contains_key(&node_id) {
                        let direction = match node_ref {
                            NodeRef::North => "N",
                            NodeRef::NorthEast => "NE",
                            NodeRef::SouthEast => "SE",
                            NodeRef::South => "S",
                            NodeRef::SouthWest => "SW",
                            NodeRef::NorthWest => "NW",
                        };

                        node_info.insert(
                            node_id,
                            (convert_coordinate(coordinate), direction.to_string()),
                        );
                    }
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

                    // BUGFIX: Get road info from state using the actual EdgeId tuple
                    let edge_id_tuple = (node1, node2);
                    let edge_color_idx = state.get_edge_owner(edge_id_tuple);
                    let edge_color = edge_color_idx.map(|color_idx| match color_idx {
                        0 => "red".to_string(),
                        1 => "blue".to_string(),
                        2 => "white".to_string(),
                        3 => "orange".to_string(),
                        _ => "unknown".to_string(),
                    });

                    // Calculate absolute coordinates for the edge endpoints using actual node IDs
                    let node1_abs = find_node_absolute_coordinate(node1, map_instance).unwrap_or(
                        NodeAbsoluteCoordinate {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        },
                    );
                    let node2_abs = find_node_absolute_coordinate(node2, map_instance).unwrap_or(
                        NodeAbsoluteCoordinate {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        },
                    );

                    edges.insert(
                        edge_id_str,
                        Edge {
                            color: edge_color,
                            node1_id: node1,
                            node2_id: node2,
                            tile_coordinate: convert_coordinate(coordinate),
                            direction: direction.to_string(),
                            node1_absolute_coordinate: node1_abs,
                            node2_absolute_coordinate: node2_abs,
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
        let node_id_str = format!("n{}", node_id);

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

        // Calculate absolute coordinate for this node
        let cube_coord = (
            tile_coordinate.x as i8,
            tile_coordinate.y as i8,
            tile_coordinate.z as i8,
        );
        let node_direction = NodeDirection::from_str(&direction).unwrap_or(NodeDirection::North);
        let absolute_coord = calculate_node_coordinate(cube_coord, node_direction);

        nodes.insert(
            node_id_str,
            Node {
                building: building_type,
                color: building_color,
                tile_coordinate,
                direction,
                absolute_coordinate: node_coordinate_to_absolute(absolute_coord),
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
                .insert(EnumResource::Brick, player_hand[0] as u32);
            player
                .resources
                .insert(EnumResource::Wood, player_hand[1] as u32);
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

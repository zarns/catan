use crate::enums::{
    Action as EnumAction, DevCard, GameConfiguration, MapType, Resource as EnumResource,
};
use crate::global_state::GlobalState;
use crate::map_instance::{
    Direction, EdgeRef, LandTile, MapInstance, NodeRef, PortTile, Tile,
};
use crate::map_template::Coordinate as CubeCoordinate;
use crate::state::{State, BuildingType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// Use EnumAction instead of defining GameAction
pub type GameAction = EnumAction;

// Game state enum to track the current state of the game
#[derive(Debug, Clone, Serialize, Deserialize)]
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

// An edge (path) on the board
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub color: Option<String>,
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

                    nodes.insert(
                        node_id_str,
                        Node {
                            building: None,
                            color: None,
                            tile_coordinate: convert_coordinate(coordinate),
                            direction: direction.to_string(),
                        },
                    );
                }

                // Generate edges for this tile
                for (&edge_ref, &(node1, _node2)) in &land_tile.hexagon.edges {
                    let direction = match edge_ref {
                        EdgeRef::East => "E",
                        EdgeRef::SouthEast => "SE",
                        EdgeRef::SouthWest => "SW",
                        EdgeRef::West => "W",
                        EdgeRef::NorthWest => "NW",
                        EdgeRef::NorthEast => "NE",
                    };

                    // Use a more compact edge ID format: e{direction}_{node1}
                    let edge_id = format!("e{}_{}", direction, node1);

                    edges.insert(
                        edge_id,
                        Edge {
                            color: None,
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
    let colors = vec!["red", "blue", "white", "orange"];

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
    Game {
        id,
        players,
        game_state: GameState::Active,
        board: generate_board_from_template(&map_instance),
        current_player_index: 0,
        dice_rolled: false,
        turns: 0,
        current_dice_roll: None,
        state: Some(state),
    }
}

// Game simulation for bot play
pub fn simulate_bot_game(num_players: u8) -> Game {
    let player_names = (0..num_players).map(|i| format!("Bot {}", i + 1)).collect();
    let game_id = format!("sim_{}", uuid::Uuid::new_v4());
    Game::new(game_id, player_names)
}

// Initial setup for a game against Catanatron
pub fn start_human_vs_catanatron(human_name: String, num_bots: u8) -> Game {
    let mut player_names = vec![human_name];

    for i in 0..num_bots {
        player_names.push(format!("Catanatron {}", i + 1));
    }

    let game_id = format!("hvs_{}", uuid::Uuid::new_v4());
    Game::new(game_id, player_names)
}

impl Game {
    pub fn new(id: String, player_names: Vec<String>) -> Self {
        let colors = vec!["red", "blue", "white", "orange"];

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
        Game {
            id,
            players,
            game_state: GameState::Active,
            board: generate_board_from_template(&map_instance),
            current_player_index: 0,
            dice_rolled: false,
            turns: 0,
            current_dice_roll: None,
            state: Some(state),
        }
    }

    // Process an action on the game
    pub fn process_action(&mut self, player_id: &str, action: GameAction) -> Result<(), String> {
        // Check if the game is in an active state
        match self.game_state {
            GameState::Setup => return Err("Game is in setup phase".into()),
            GameState::Finished { .. } => return Err("Game already finished".into()),
            GameState::Active => {}
        }

        // Ensure we have a valid state
        let state = match &mut self.state {
            Some(state) => state,
            None => return Err("Game state is missing".into()),
        };

        // Find the player's color index
        let player_index = self.players.iter().position(|p| p.id == player_id);

        if player_index.is_none() {
            return Err("Player not found".into());
        }

        let player_index = player_index.unwrap();
        let _color_idx = player_index as u8; // Prefix with underscore to indicate it's unused

        // Apply the action directly since GameAction is now an alias for EnumAction
        state.apply_action(action);

        // Update frontend players from the state
        update_players_from_state(&mut self.players, state);

        // Update current player and dice rolled status
        self.current_player_index = state.get_current_color() as usize;
        self.dice_rolled = state.current_player_rolled();

        // Check if the game is finished
        if let Some(winner_color) = state.winner() {
            let winner_name = if (winner_color as usize) < self.players.len() {
                self.players[winner_color as usize].name.clone()
            } else {
                "Unknown".to_string()
            };

            // Update the game state to finished
            self.game_state = GameState::Finished { winner: winner_name };
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
}

// Generate a serializable game board from the State and MapInstance
fn generate_board_from_state(state: &State, map_instance: &MapInstance) -> GameBoard {
    let mut tiles = Vec::new();
    let mut ports = Vec::new();
    let mut nodes = HashMap::new();
    let mut edges = HashMap::new();
    let mut robber_coordinate = None;

    // Get robber position from state
    let robber_tile_id = state.get_robber_tile();

    // Process all tiles
    for (&coordinate, tile) in &map_instance.tiles {
        match tile {
            Tile::Land(land_tile) => {
                // Convert land tile to frontend format
                tiles.push(convert_land_tile(coordinate, land_tile));

                // Track the robber location
                if land_tile.id == robber_tile_id {
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

                    // Get building info using public state methods
                    let building_type_opt = state.get_building_type(node_id);
                    let building_color_idx_opt = state.get_node_color(node_id);

                    let building_type = match building_type_opt {
                        Some(BuildingType::Settlement) => Some("settlement".to_string()),
                        Some(BuildingType::City) => Some("city".to_string()),
                        None => None,
                    };

                    let building_color = match building_color_idx_opt {
                        Some(color_idx) => Some(match color_idx {
                            0 => "red".to_string(),
                            1 => "blue".to_string(),
                            2 => "white".to_string(),
                            3 => "orange".to_string(),
                            _ => "unknown".to_string(), // Handle unexpected color index
                        }),
                        None => None,
                    };

                    nodes.insert(
                        node_id_str,
                        Node {
                            building: building_type,
                            color: building_color,
                            tile_coordinate: convert_coordinate(coordinate),
                            direction: direction.to_string(),
                        },
                    );
                }

                // Generate edges for this tile
                for (&edge_ref, &(node1, _node2)) in &land_tile.hexagon.edges {
                    let direction = match edge_ref {
                        EdgeRef::East => "E",
                        EdgeRef::SouthEast => "SE",
                        EdgeRef::SouthWest => "SW",
                        EdgeRef::West => "W",
                        EdgeRef::NorthWest => "NW",
                        EdgeRef::NorthEast => "NE",
                    };

                    // Use a more compact edge ID format: e{direction}_{node1}
                    let edge_id = format!("e{}_{}", direction, node1);

                    // Get road info from state
                    // This is a placeholder. You would need to implement a way to get the 
                    // edge owner from your State object.
                    let edge_color = None; // To be implemented: state.get_edge_owner(edge_id)

                    edges.insert(
                        edge_id,
                        Edge {
                            color: edge_color,
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

    GameBoard {
        tiles,
        ports,
        nodes,
        edges,
        robber_coordinate,
    }
}

// Update frontend players from the State
fn update_players_from_state(players: &mut Vec<Player>, state: &State) {
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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Basic game types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Resource {
    Brick,
    Lumber,
    Wool,
    Grain,
    Ore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DevelopmentCard {
    Knight,
    VictoryPoint,
    RoadBuilding,
    YearOfPlenty,
    Monopoly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub color: String,
    pub resources: HashMap<Resource, u32>,
    pub dev_cards: Vec<DevelopmentCard>,
    pub knights_played: u32,
    pub victory_points: u32,
    pub longest_road: bool,
    pub largest_army: bool,
}

impl Player {
    pub fn new(id: String, name: String, color: String) -> Self {
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub resource: Option<String>,
    pub number: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilePosition {
    pub coordinate: Coordinate,
    pub tile: Tile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub building: Option<String>,
    pub color: Option<String>,
    pub tile_coordinate: Coordinate,
    pub direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub color: Option<String>,
    pub tile_coordinate: Coordinate,
    pub direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameBoard {
    pub tiles: Vec<TilePosition>,
    pub nodes: HashMap<String, Node>,
    pub edges: HashMap<String, Edge>,
    pub robber_coordinate: Option<Coordinate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum GameAction {
    RollDice,
    BuildRoad,
    BuildSettlement,
    BuildCity,
    BuyDevelopmentCard,
    PlayDevelopmentCard,
    TradeWithBank,
    TradeWithPlayer,
    EndTurn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub players: Vec<Player>,
    pub board: GameBoard,
    pub current_player_index: usize,
    pub dice_rolled: bool,
    pub winner: Option<String>,
    pub turns: u32,
}

impl Game {
    pub fn new(id: String, player_names: Vec<String>) -> Self {
        let colors = vec!["red", "blue", "white", "orange"];
        
        let players = player_names
            .into_iter()
            .enumerate()
            .map(|(i, name)| {
                let player_id = format!("player_{}", i);
                let color = colors[i % colors.len()].to_string();
                Player::new(player_id, name, color)
            })
            .collect();

        Game {
            id,
            players,
            board: generate_mock_board(),
            current_player_index: 0,
            dice_rolled: false,
            winner: None,
            turns: 0,
        }
    }

    pub fn process_action(&mut self, _player_id: &str, _action: GameAction) -> Result<(), String> {
        // Basic validation
        if self.winner.is_some() {
            return Err("Game already has a winner".into());
        }

        // Game action handling will be implemented later
        Ok(())
    }
}

// Generate a mock game board with a standard Catan layout
pub fn generate_mock_board() -> GameBoard {
    let mut tiles = Vec::new();
    let mut nodes = HashMap::new();
    let mut edges = HashMap::new();
    
    // Create a simple hexagonal grid
    // Center tile
    tiles.push(TilePosition {
        coordinate: Coordinate { x: 0, y: 0, z: 0 },
        tile: Tile {
            resource: Some("lumber".to_string()),
            number: Some(6),
        },
    });
    
    // First ring - 6 tiles
    let first_ring_resources = vec![
        "brick", "wool", "grain", "ore", "lumber", "wool"
    ];
    let first_ring_numbers = vec![3, 4, 5, 8, 9, 10];
    let first_ring_coords = vec![
        Coordinate { x: 1, y: -1, z: 0 },
        Coordinate { x: 0, y: -1, z: 1 },
        Coordinate { x: -1, y: 0, z: 1 },
        Coordinate { x: -1, y: 1, z: 0 },
        Coordinate { x: 0, y: 1, z: -1 },
        Coordinate { x: 1, y: 0, z: -1 },
    ];
    
    for i in 0..6 {
        tiles.push(TilePosition {
            coordinate: first_ring_coords[i].clone(),
            tile: Tile {
                resource: Some(first_ring_resources[i].to_string()),
                number: Some(first_ring_numbers[i]),
            },
        });
    }
    
    // Second ring - 12 tiles (simplified for this mock)
    let second_ring_resources = vec![
        Some("grain"), Some("brick"), None, Some("lumber"),
        Some("wool"), Some("ore"), Some("grain"), Some("brick"),
        Some("wool"), Some("lumber"), Some("ore"), Some("grain"),
    ];
    let second_ring_numbers = vec![2, 3, 0, 4, 5, 6, 8, 9, 10, 11, 12, 11];
    let second_ring_coords = vec![
        Coordinate { x: 2, y: -2, z: 0 },
        Coordinate { x: 1, y: -2, z: 1 },
        Coordinate { x: 0, y: -2, z: 2 }, // Desert
        Coordinate { x: -1, y: -1, z: 2 },
        Coordinate { x: -2, y: 0, z: 2 },
        Coordinate { x: -2, y: 1, z: 1 },
        Coordinate { x: -2, y: 2, z: 0 },
        Coordinate { x: -1, y: 2, z: -1 },
        Coordinate { x: 0, y: 2, z: -2 },
        Coordinate { x: 1, y: 1, z: -2 },
        Coordinate { x: 2, y: 0, z: -2 },
        Coordinate { x: 2, y: -1, z: -1 },
    ];
    
    for i in 0..12 {
        tiles.push(TilePosition {
            coordinate: second_ring_coords[i].clone(),
            tile: Tile {
                resource: second_ring_resources[i].map(|s| s.to_string()),
                number: if second_ring_numbers[i] == 0 { None } else { Some(second_ring_numbers[i]) },
            },
        });
    }
    
    // Generate nodes (intersections)
    // We'll create a simplified set of nodes for demonstration
    let node_directions = vec!["N", "NE", "SE", "S", "SW", "NW"];
    
    // For each tile, create 6 nodes (one at each corner)
    for tile in &tiles {
        for (i, direction) in node_directions.iter().enumerate() {
            let node_id = format!("node_{}_{}", Uuid::new_v4().to_string().split('-').next().unwrap(), i);
            nodes.insert(node_id.clone(), Node {
                building: None,
                color: None,
                tile_coordinate: tile.coordinate.clone(),
                direction: direction.to_string(),
            });
        }
    }
    
    // Generate edges (between nodes)
    // Again, simplified for demonstration
    let edge_directions = vec!["NE", "E", "SE", "SW", "W", "NW"];
    
    // For each tile, create 6 edges
    for tile in &tiles {
        for (i, direction) in edge_directions.iter().enumerate() {
            let edge_id = format!("edge_{}_{}", Uuid::new_v4().to_string().split('-').next().unwrap(), i);
            edges.insert(edge_id.clone(), Edge {
                color: None,
                tile_coordinate: tile.coordinate.clone(),
                direction: direction.to_string(),
            });
        }
    }
    
    // Place the robber on the desert tile if it exists, otherwise on a random tile
    let robber_coordinate = tiles.iter()
        .find(|t| t.tile.resource.is_none())
        .map(|t| t.coordinate.clone())
        .or_else(|| Some(tiles[0].coordinate.clone()));
    
    GameBoard {
        tiles,
        nodes,
        edges,
        robber_coordinate,
    }
}

// Game simulation for bot play
pub fn simulate_bot_game(num_players: u8) -> Game {
    let player_names = (0..num_players)
        .map(|i| format!("Bot {}", i + 1))
        .collect();
    
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
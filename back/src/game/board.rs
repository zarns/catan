// src/game/board.rs
use rand::prelude::*;
use std::collections::{HashMap, HashSet};
use super::types::*;

pub struct BoardGenerator;

impl BoardGenerator {
    pub fn generate_board() -> Board {
        let mut rng = thread_rng();
        
        // Create terrain tiles
        let mut terrains = vec![
            TerrainType::Hills,    TerrainType::Hills,    TerrainType::Hills,
            TerrainType::Forest,   TerrainType::Forest,   TerrainType::Forest,   TerrainType::Forest,
            TerrainType::Mountains, TerrainType::Mountains, TerrainType::Mountains,
            TerrainType::Fields,   TerrainType::Fields,   TerrainType::Fields,   TerrainType::Fields,
            TerrainType::Pasture,  TerrainType::Pasture,  TerrainType::Pasture,  TerrainType::Pasture,
            TerrainType::Desert,
        ];
        terrains.shuffle(&mut rng);

        // Create number tokens (excluding 7)
        let mut tokens = vec![
            2,
            3, 3,
            4, 4,
            5, 5,
            6, 6,
            8, 8,
            9, 9,
            10, 10,
            11, 11,
            12,
        ];
        tokens.shuffle(&mut rng);

        // Generate hex grid
        let mut hexes = Vec::new();
        let mut token_idx = 0;
        let positions = Self::generate_hex_positions();

        for (i, pos) in positions.iter().enumerate() {
            let terrain = terrains[i];
            let token = if terrain != TerrainType::Desert {
                let token = tokens[token_idx];
                token_idx += 1;
                Some(token)
            } else {
                None
            };

            hexes.push(Hex {
                position: pos.clone(),
                terrain,
                token,
                has_robber: terrain == TerrainType::Desert,
            });
        }

        // Generate harbors
        let harbors = Self::generate_harbors();

        Board {
            hexes,
            harbors,
            roads: HashMap::new(),
            settlements: HashMap::new(),
        }
    }

    fn generate_hex_positions() -> Vec<Position> {
        // Layout coordinates for a standard Catan board
        let layout = [
            // Row 1
            vec![(0, 0), (2, 0), (4, 0)],
            // Row 2
            vec![(-1, 2), (1, 2), (3, 2), (5, 2)],
            // Row 3
            vec![(-2, 4), (0, 4), (2, 4), (4, 4), (6, 4)],
            // Row 4
            vec![(-1, 6), (1, 6), (3, 6), (5, 6)],
            // Row 5
            vec![(0, 8), (2, 8), (4, 8)],
        ];

        let mut positions = Vec::new();
        for row in layout.iter() {
            for &(x, y) in row {
                positions.push(Position { 
                    x: (x + 2) as u32,  // Shift to ensure all coordinates are positive
                    y: y as u32 
                });
            }
        }
        positions
    }

    fn generate_harbors() -> Vec<Harbor> {
        let mut harbor_types = vec![
            HarborType::Generic, HarborType::Generic, HarborType::Generic, HarborType::Generic,
            HarborType::Brick, HarborType::Lumber, HarborType::Ore, HarborType::Grain, HarborType::Wool,
        ];
        
        let mut rng = thread_rng();
        harbor_types.shuffle(&mut rng);

        // Pre-defined harbor positions (edge positions)
        let harbor_positions = vec![
            Position { x: 0, y: 1 },  // Example positions
            Position { x: 1, y: 3 },
            Position { x: 3, y: 0 },
            Position { x: 5, y: 1 },
            Position { x: 6, y: 3 },
            Position { x: 5, y: 5 },
            Position { x: 3, y: 6 },
            Position { x: 1, y: 5 },
            Position { x: 0, y: 3 },
        ];

        harbor_positions.into_iter()
            .zip(harbor_types.into_iter())
            .map(|(position, harbor_type)| Harbor { position, harbor_type })
            .collect()
    }

    pub fn get_valid_settlement_positions(board: &Board, must_connect_to_road: bool) -> HashSet<Position> {
        let mut valid_positions = HashSet::new();
        
        // Define valid vertex positions based on hex positions
        for hex in &board.hexes {
            // Add all vertices around the hex
            let vertices = Self::get_hex_vertices(&hex.position);
            for vertex in vertices {
                // Check if this position is valid for a settlement
                if Self::is_valid_settlement_position(board, &vertex, must_connect_to_road) {
                    valid_positions.insert(vertex);
                }
            }
        }

        valid_positions
    }

    fn get_hex_vertices(hex_pos: &Position) -> Vec<Position> {
        // Return the 6 vertex positions around a hex
        vec![
            Position { x: hex_pos.x, y: hex_pos.y - 1 },
            Position { x: hex_pos.x + 1, y: hex_pos.y - 1 },
            Position { x: hex_pos.x + 1, y: hex_pos.y },
            Position { x: hex_pos.x, y: hex_pos.y + 1 },
            Position { x: hex_pos.x - 1, y: hex_pos.y + 1 },
            Position { x: hex_pos.x - 1, y: hex_pos.y },
        ]
    }

    fn is_valid_settlement_position(board: &Board, pos: &Position, must_connect_to_road: bool) -> bool {
        // Check distance rule (no adjacent settlements)
        for settlement_pos in board.settlements.keys() {
            if Self::are_positions_adjacent(pos, settlement_pos) {
                return false;
            }
        }

        // If in setup phase, don't check for roads
        if !must_connect_to_road {
            return true;
        }

        // Check if connected to player's road
        let mut has_connected_road = false;
        for ((road_start, road_end), _player_id) in &board.roads {
            if road_start == pos || road_end == pos {
                has_connected_road = true;
                break;
            }
        }

        has_connected_road
    }

    fn are_positions_adjacent(pos1: &Position, pos2: &Position) -> bool {
        let dx = (pos1.x as i32 - pos2.x as i32).abs();
        let dy = (pos1.y as i32 - pos2.y as i32).abs();
        dx <= 1 && dy <= 1
    }

    pub fn get_adjacent_vertices(pos: &Position) -> Vec<Position> {
      vec![
          Position { x: pos.x + 1, y: pos.y },
          Position { x: pos.x - 1, y: pos.y },
          Position { x: pos.x, y: pos.y + 1 },
          Position { x: pos.x, y: pos.y - 1 },
      ]
  }
}
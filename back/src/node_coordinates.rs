use crate::map_template::Coordinate as CubeCoordinate;
use std::collections::HashMap;

/// Represents an absolute coordinate for a node in hexagonal grid space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeCoordinate {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Direction enum for node positioning relative to hex centers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeDirection {
    North,
    NorthEast,
    SouthEast,
    South,
    SouthWest,
    NorthWest,
}

impl NodeDirection {
    /// Convert from string representation to enum
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "N" | "NORTH" => Some(Self::North),
            "NE" | "NORTHEAST" => Some(Self::NorthEast),
            "SE" | "SOUTHEAST" => Some(Self::SouthEast),
            "S" | "SOUTH" => Some(Self::South),
            "SW" | "SOUTHWEST" => Some(Self::SouthWest),
            "NW" | "NORTHWEST" => Some(Self::NorthWest),
            _ => None,
        }
    }

    /// Convert to string representation
    pub fn to_str(self) -> &'static str {
        match self {
            Self::North => "N",
            Self::NorthEast => "NE",
            Self::SouthEast => "SE",
            Self::South => "S",
            Self::SouthWest => "SW",
            Self::NorthWest => "NW",
        }
    }

    /// Get the angle in radians for this direction (0 = North, clockwise)
    pub fn angle_radians(self) -> f64 {
        match self {
            Self::North => 0.0,
            Self::NorthEast => std::f64::consts::PI / 3.0,
            Self::SouthEast => 2.0 * std::f64::consts::PI / 3.0,
            Self::South => std::f64::consts::PI,
            Self::SouthWest => 4.0 * std::f64::consts::PI / 3.0,
            Self::NorthWest => 5.0 * std::f64::consts::PI / 3.0,
        }
    }

    /// Get all directions in clockwise order starting from North
    pub fn all_clockwise() -> [Self; 6] {
        [
            Self::North,
            Self::NorthEast,
            Self::SouthEast,
            Self::South,
            Self::SouthWest,
            Self::NorthWest,
        ]
    }
}

/// DEPRECATED: Coordinate calculation moved to frontend
/// This stub exists only for backward compatibility during migration
pub fn calculate_node_coordinate(
    _tile_coord: CubeCoordinate,
    _direction: NodeDirection,
) -> NodeCoordinate {
    // Return zero coordinates - frontend will calculate actual positions
    NodeCoordinate {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    }
}

/// Generate a canonical node numbering system starting from center tile (0,0,0)
/// Node 0 will be at the North position of the center tile
/// Nodes are numbered in expanding rings, clockwise within each ring
pub fn generate_canonical_node_map(
    tile_map: &HashMap<CubeCoordinate, Vec<NodeDirection>>,
) -> HashMap<u8, NodeCoordinate> {
    let mut node_map = HashMap::new();
    let mut node_id: u8 = 0;

    // Center tile first - always (0, 0, 0)
    let center_coord = (0, 0, 0);
    if let Some(directions) = tile_map.get(&center_coord) {
        // Start with North direction for node 0
        let mut sorted_directions = directions.clone();
        sorted_directions.sort_by_key(|d| match d {
            NodeDirection::North => 0,
            NodeDirection::NorthEast => 1,
            NodeDirection::SouthEast => 2,
            NodeDirection::South => 3,
            NodeDirection::SouthWest => 4,
            NodeDirection::NorthWest => 5,
        });

        for &direction in &sorted_directions {
            let node_coord = calculate_node_coordinate(center_coord, direction);
            node_map.insert(node_id, node_coord);
            node_id += 1;
        }
    }

    // Ring 1: Adjacent tiles to center
    let ring1_tiles = [
        (0, 1, -1), // NorthWest
        (1, 0, -1), // NorthEast
        (1, -1, 0), // East
        (0, -1, 1), // SouthEast
        (-1, 0, 1), // SouthWest
        (-1, 1, 0), // West
    ];

    for &tile_coord in &ring1_tiles {
        if let Some(directions) = tile_map.get(&tile_coord) {
            let mut sorted_directions = directions.clone();
            sorted_directions.sort_by_key(|d| match d {
                NodeDirection::North => 0,
                NodeDirection::NorthEast => 1,
                NodeDirection::SouthEast => 2,
                NodeDirection::South => 3,
                NodeDirection::SouthWest => 4,
                NodeDirection::NorthWest => 5,
            });

            for &direction in &sorted_directions {
                let node_coord = calculate_node_coordinate(tile_coord, direction);

                // Only add if this coordinate doesn't already exist
                let exists = node_map.values().any(|existing| {
                    (existing.x - node_coord.x).abs() < 0.001
                        && (existing.y - node_coord.y).abs() < 0.001
                });

                if !exists {
                    node_map.insert(node_id, node_coord);
                    node_id += 1;
                }
            }
        }
    }

    // Ring 2: Next outer ring (if needed)
    let ring2_tiles = [
        (0, 2, -2),
        (-1, 2, -1),
        (-2, 2, 0), // North side
        (1, 1, -2),
        (2, 0, -2),
        (2, -1, -1), // NorthEast side
        (2, -2, 0),
        (1, -2, 1),
        (0, -2, 2), // Southeast side
        (-1, -1, 2),
        (-2, 0, 2),
        (-2, 1, 1), // Southwest side
        (-1, 1, 0), // West vertex (already covered)
    ];

    for &tile_coord in &ring2_tiles {
        if let Some(directions) = tile_map.get(&tile_coord) {
            let mut sorted_directions = directions.clone();
            sorted_directions.sort_by_key(|d| match d {
                NodeDirection::North => 0,
                NodeDirection::NorthEast => 1,
                NodeDirection::SouthEast => 2,
                NodeDirection::South => 3,
                NodeDirection::SouthWest => 4,
                NodeDirection::NorthWest => 5,
            });

            for &direction in &sorted_directions {
                let node_coord = calculate_node_coordinate(tile_coord, direction);

                // Only add if this coordinate doesn't already exist
                let exists = node_map.values().any(|existing| {
                    (existing.x - node_coord.x).abs() < 0.001
                        && (existing.y - node_coord.y).abs() < 0.001
                });

                if !exists {
                    node_map.insert(node_id, node_coord);
                    node_id += 1;
                }
            }
        }
    }

    node_map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_direction_angles() {
        assert_eq!(NodeDirection::North.angle_radians(), 0.0);
        assert_eq!(
            NodeDirection::NorthEast.angle_radians(),
            std::f64::consts::PI / 3.0
        );
        assert_eq!(NodeDirection::South.angle_radians(), std::f64::consts::PI);
    }

    #[test]
    fn test_calculate_node_coordinate() {
        let center_tile = (0, 0, 0);
        let north_node = calculate_node_coordinate(center_tile, NodeDirection::North);

        // North node should be directly above center (y negative in screen coordinates)
        assert!((north_node.x - 0.0).abs() < 0.001);
        assert!(north_node.y < 0.0);
    }

    #[test]
    fn test_node_direction_conversion() {
        // Test string to enum conversion
        assert_eq!(NodeDirection::from_str("N"), Some(NodeDirection::North));
        assert_eq!(NodeDirection::from_str("NORTH"), Some(NodeDirection::North));
        assert_eq!(
            NodeDirection::from_str("NE"),
            Some(NodeDirection::NorthEast)
        );
        assert_eq!(
            NodeDirection::from_str("NORTHEAST"),
            Some(NodeDirection::NorthEast)
        );
        assert_eq!(NodeDirection::from_str("invalid"), None);

        // Test enum to string conversion
        assert_eq!(NodeDirection::North.to_str(), "N");
        assert_eq!(NodeDirection::NorthEast.to_str(), "NE");
        assert_eq!(NodeDirection::SouthWest.to_str(), "SW");
    }

    #[test]
    fn test_hexagonal_grid_geometry() {
        // Test that adjacent tiles have the correct distance
        let center = (0, 0, 0);
        let adjacent = (1, 0, -1); // NorthEast tile

        let center_north = calculate_node_coordinate(center, NodeDirection::North);
        let adjacent_south = calculate_node_coordinate(adjacent, NodeDirection::South);

        // These nodes should be at different positions
        let distance = ((center_north.x - adjacent_south.x).powi(2)
            + (center_north.y - adjacent_south.y).powi(2))
        .sqrt();

        // Distance should be greater than 0 (they're different positions)
        assert!(distance > 0.1);
    }

    #[test]
    fn test_canonical_node_numbering() {
        // Create a simple tile map with center tile
        let mut tile_map = HashMap::new();
        tile_map.insert(
            (0, 0, 0),
            vec![
                NodeDirection::North,
                NodeDirection::NorthEast,
                NodeDirection::SouthEast,
            ],
        );

        let node_map = generate_canonical_node_map(&tile_map);

        // Should have 3 nodes
        assert_eq!(node_map.len(), 3);

        // Node 0 should be North of center (closest to North)
        let node_0 = node_map.get(&0).unwrap();
        assert!(node_0.y < 0.0); // North is negative Y in screen coordinates
        assert!((node_0.x - 0.0).abs() < 0.001); // Should be centered on X axis
    }

    // REMOVED: test_coordinate_uniqueness - coordinate calculation moved to frontend
}

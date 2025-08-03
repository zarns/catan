/**
 * Hexagonal Grid Mathematics Utility
 * 
 * Based on Red Blob Games hex grid algorithms:
 * https://www.redblobgames.com/grids/hexagons/
 * 
 * This utility provides conversion between cube coordinates and pixel coordinates
 * for pointy-top hexagonal grids, plus node positioning within hexes.
 */

export interface CubeCoordinate {
  x: number;
  y: number;
  z: number;
}

export interface PixelCoordinate {
  x: number;
  y: number;
}

export enum NodeDirection {
  North = 'N',
  NorthEast = 'NE', 
  SouthEast = 'SE',
  South = 'S',
  SouthWest = 'SW',
  NorthWest = 'NW'
}

// Constants for hex grid calculations
const SQRT3 = Math.sqrt(3);

/**
 * Convert cube coordinates to pixel coordinates using Red Blob Games formula
 * For pointy-top hexagons (flat side horizontal)
 * 
 * @param cube - Cube coordinate {x, y, z} where x + y + z = 0
 * @param size - Hex radius (distance from center to vertex)
 * @returns Pixel coordinate {x, y}
 */
export function cubeToPixel(cube: CubeCoordinate, size: number): PixelCoordinate {
  // Convert cube to axial coordinates (q = x, r = z)
  const q = cube.x;
  const r = cube.z;
  
  // Red Blob Games pointy-top hex-to-pixel formula
  // Match exactly what the tile component uses
  const x = size * (SQRT3 * q + SQRT3 / 2.0 * r);
  const y = size * (3.0 / 2.0 * r);
  
  return { x, y };
}

/**
 * Get the angle in radians for a node direction
 * For pointy-top hexagons, vertices are at 30°, 90°, 150°, 210°, 270°, 330°
 * 
 * @param direction - Node direction enum
 * @returns Angle in radians
 */
function getNodeDirectionAngle(direction: NodeDirection): number {
  switch (direction) {
    case NodeDirection.North:     return Math.PI / 2;           // 90°
    case NodeDirection.NorthEast: return Math.PI / 6;           // 30°
    case NodeDirection.SouthEast: return -Math.PI / 6;          // 330° = -30°
    case NodeDirection.South:     return -Math.PI / 2;          // 270° = -90°
    case NodeDirection.SouthWest: return -5 * Math.PI / 6;      // 210° = -150°
    case NodeDirection.NorthWest: return 5 * Math.PI / 6;       // 150°
    default:
      console.warn(`Unknown node direction: ${direction}`);
      return 0;
  }
}

/**
 * Get the pixel offset from hex center to a node (vertex) position
 * 
 * @param direction - Which vertex of the hex (N, NE, SE, S, SW, NW)
 * @param size - Hex radius (distance from center to vertex)
 * @returns Pixel offset {x, y} from hex center to vertex
 */
export function getNodeDirectionOffset(direction: NodeDirection, size: number): PixelCoordinate {
  const angle = getNodeDirectionAngle(direction);
  
  // For pointy-top hexagons, vertices are at distance 'size' from center
  const x = size * Math.cos(angle);
  const y = -size * Math.sin(angle); // FLIP Y for web coordinate system (Y increases downward)
  
  return { x, y };
}

/**
 * Convert a node direction string to NodeDirection enum
 * Handles both full names and abbreviations
 * 
 * @param directionStr - Direction string like "N", "NORTH", "NE", "NORTHEAST"
 * @returns NodeDirection enum or North as fallback
 */
export function parseNodeDirection(directionStr: string): NodeDirection {
  const normalized = directionStr.toUpperCase();
  
  switch (normalized) {
    case 'N':
    case 'NORTH':
      return NodeDirection.North;
    case 'NE':
    case 'NORTHEAST':
      return NodeDirection.NorthEast;
    case 'SE':
    case 'SOUTHEAST':
      return NodeDirection.SouthEast;
    case 'S':
    case 'SOUTH':
      return NodeDirection.South;
    case 'SW':
    case 'SOUTHWEST':
      return NodeDirection.SouthWest;
    case 'NW':
    case 'NORTHWEST':
      return NodeDirection.NorthWest;
    default:
      console.warn(`Unknown direction string: ${directionStr}, using North as fallback`);
      return NodeDirection.North;
  }
}

/**
 * Calculate the pixel position of a node given its canonical tile and direction
 * This is the main function that replaces backend coordinate calculation
 * 
 * @param tileCoordinate - Cube coordinate of the canonical tile
 * @param direction - Direction string (N, NE, SE, S, SW, NW)
 * @param hexSize - Hex radius for scaling
 * @returns Absolute pixel position of the node
 */
export function calculateNodePixelPosition(
  tileCoordinate: CubeCoordinate, 
  direction: string, 
  hexSize: number
): PixelCoordinate {
  // Use EXACT same calculation as tile component for hex center
  const SQRT3 = Math.sqrt(3);
  const q = tileCoordinate.x;
  const r = tileCoordinate.z;
  
  const tile_center_x = hexSize * (SQRT3 * q + SQRT3 / 2.0 * r);
  const tile_center_y = hexSize * (3.0 / 2.0 * r);
  
  const hexCenter = { x: tile_center_x, y: tile_center_y };
  
  // Parse direction and get node offset
  const nodeDirection = parseNodeDirection(direction);
  const nodeOffset = getNodeDirectionOffset(nodeDirection, hexSize);
  
  // Calculate final node position
  return {
    x: hexCenter.x + nodeOffset.x,
    y: hexCenter.y + nodeOffset.y
  };
}

/**
 * Validate that a cube coordinate satisfies the constraint x + y + z = 0
 * 
 * @param cube - Cube coordinate to validate
 * @returns true if valid, false otherwise
 */
export function isValidCubeCoordinate(cube: CubeCoordinate): boolean {
  const sum = cube.x + cube.y + cube.z;
  return Math.abs(sum) < 0.001; // Allow for floating point precision
}

/**
 * Calculate the distance between two nodes in pixels
 * 
 * @param pos1 - First pixel position
 * @param pos2 - Second pixel position  
 * @returns Euclidean distance in pixels
 */
export function pixelDistance(pos1: PixelCoordinate, pos2: PixelCoordinate): number {
  const dx = pos1.x - pos2.x;
  const dy = pos1.y - pos2.y;
  return Math.sqrt(dx * dx + dy * dy);
}
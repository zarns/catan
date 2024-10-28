// src/app/components/board/coordinates.ts
export const SQRT3 = 1.73205080757;
export const SPACING_FACTOR = 2; // Add this to increase spacing between hexes

export interface PixelPosition {
    x: number;
    y: number;
}

// src/app/components/board/coordinates.ts
export const CATAN_LAYOUT = [
  { q: -2, r: 0 },
  { q: -2, r: 1 },
  { q: -2, r: 2 },
  { q: -1, r: -1 },
  { q: -1, r: 0 },
  { q: -1, r: 1 },
  { q: -1, r: 2 },
  { q: 0, r: -2 },
  { q: 0, r: -1 },
  { q: 0, r: 0 },
  { q: 0, r: 1 },
  { q: 0, r: 2 },
  { q: 1, r: -2 },
  { q: 1, r: -1 },
  { q: 1, r: 0 },
  { q: 1, r: 1 },
  { q: 2, r: -2 },
  { q: 2, r: -1 },
  { q: 2, r: 0 }
];

export function tilePixelVector(
    coordinate: number[], 
    size: number, 
    centerX: number, 
    centerY: number
): PixelPosition {
    const [x, y] = coordinate;
    
    // Increase the spacing between hexes horizontally and vertically
    const position = {
        x: size * SPACING_FACTOR * (SQRT3 * x) + centerX,
        y: size * SPACING_FACTOR * (y * 1.5) + centerY
    };
    
    console.log('Calculated hex position:', {
        inputCoordinate: [x, y],
        pixelPosition: position,
        size,
        spacingFactor: SPACING_FACTOR
    });
    
    return position;
}

export function getHexPoints(pos: PixelPosition, size: number): string {
    const points: [number, number][] = [];
    for (let i = 0; i < 6; i++) {
        const angle = (Math.PI / 3) * i;
        points.push([
            pos.x + size * Math.cos(angle),
            pos.y + size * Math.sin(angle)
        ]);
    }
    return points.map(p => p.join(',')).join(' ');
}
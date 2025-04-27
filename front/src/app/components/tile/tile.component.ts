import { Component, Input, EventEmitter, Output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatCardModule } from '@angular/material/card';

@Component({
  selector: 'app-tile',
  standalone: true,
  imports: [CommonModule, MatCardModule],
  template: `
    <div class="tile" 
         [ngClass]="{'port-tile': isPort, 'hex-coord': true, 'hex-x-{{coordinate?.x}}': true, 'hex-y-{{coordinate?.y}}': true, 'hex-z-{{coordinate?.z}}': true}"
         [ngStyle]="tileStyle"
         [attr.data-hex-x]="coordinate?.x"
         [attr.data-hex-y]="coordinate?.y"
         [attr.data-hex-z]="coordinate?.z"
         [attr.data-hex-coord]="getCoordinateString()"
         [attr.data-direction]="isPort ? portDirection : ''"
         (click)="onClick.emit(coordinate)">
      
      <img [src]="getTileImageSrc()" class="tile-image" [alt]="getNormalizedResource() + ' tile'">
      
      <!-- Only render the number token if there's a number -->
      <div *ngIf="number" class="number-token" [ngClass]="{ 'flashing': flashing, 'high-probability': isHighProbability() }">
        <div class="number">{{ number }}</div>
        <div class="pips">{{ numberToPips(number) }}</div>
      </div>
      
      <!-- Port indicator for harbors - move this with the same transform as the port tile -->
      <div *ngIf="isPort" class="port-container" [ngStyle]="getPortTranslateStyle()">
        <div class="port-indicator">
          <div class="port-ratio">{{ getPortRatio() }}</div>
          <div *ngIf="isResourcePort()" class="resource-hex" [ngClass]="getPortResourceClass()">
            <div class="resource-icon">{{ getResourceIconText() }}</div>
          </div>
          <!-- Comment out coordinates for debugging -->
          <!-- <div class="coord-debug">{{getCoordinateString()}}</div> -->
        </div>
      </div>
    </div>
  `,
  styleUrls: ['./tile.component.scss']
})
export class TileComponent {
  @Input() coordinate: any;
  @Input() resource: string | null = '';
  @Input() number: number | null | undefined = null;
  @Input() size: number = 60;
  @Input() centerX: number = 0;
  @Input() centerY: number = 0;
  @Input() flashing: boolean = false;
  
  // Port-specific properties
  @Input() isPort: boolean = false;
  @Input() portResource: string | null = null;
  @Input() portRatio: number = 3;
  @Input() portDirection: string = '';
  
  @Output() onClick = new EventEmitter<any>();
  
  // Constants
  readonly SQRT3 = 1.732;
  
  // Get coordinate as a string for display
  getCoordinateString(): string {
    if (!this.coordinate) return '';
    return `(${this.coordinate.x},${this.coordinate.y},${this.coordinate.z})`;
  }
  
  get tileStyle() {
    const w = this.SQRT3 * this.size;
    const h = 2 * this.size;
    
    const [x, y] = this.tilePixelVector();
    
    // Make port tiles smaller (60% of normal size)
    const scale = this.isPort ? 0.6 : 1.0;
    
    // For port tiles, apply both translation and scaling
    let transform = 'none';
    if (this.isPort) {
      transform = `scale(${scale}) ${this.getPortTileTransform()}`;
    }
    
    // Add the coordinates as a custom CSS property
    return {
      left: `${x - w/2}px`, // Don't apply scale here
      top: `${y - h/2}px`,  // Don't apply scale here
      width: `${w}px`,      // Use original size
      height: `${h}px`,     // Use original size
      transform: transform, // Apply both scaling and translation in one transform
      '--hex-x': this.coordinate?.x,
      '--hex-y': this.coordinate?.y,
      '--hex-z': this.coordinate?.z
    };
  }
  
  // Convert cube coordinates to pixel coordinates
  tilePixelVector(): [number, number] {
    if (!this.coordinate) {
      return [0, 0];
    }
    
    const { x, y, z } = this.coordinate;
    const size = this.size;
    const width = this.SQRT3 * size;
    const height = 2 * size;
    
    // Convert cube coordinates to pixel coordinates
    const pixelX = this.centerX + width * (x + y/2);
    const pixelY = this.centerY + height * (3/4) * y;
    
    return [pixelX, pixelY];
  }
  
  // Check if this is a high probability number (6 or 8)
  isHighProbability(): boolean {
    return this.number === 6 || this.number === 8;
  }
  
  // Convert number to dots for display
  numberToPips(number: number): string {
    switch (number) {
      case 2:
      case 12:
        return '•';
      case 3:
      case 11:
        return '••';
      case 4:
      case 10:
        return '•••';
      case 5:
      case 9:
        return '••••';
      case 6:
      case 8:
        return '•••••';
      default:
        return '';
    }
  }
  
  // Normalize the resource value (handle null, empty string as 'desert')
  getNormalizedResource(): string {
    return this.resource === null || this.resource === '' ? 'desert' : this.resource.toLowerCase();
  }
  
  // Get SVG image source for the tile based on resource type
  getTileImageSrc(): string {
    const normalizedResource = this.getNormalizedResource();
    
    // Map resource names to file names
    const resourceMap: {[key: string]: string} = {
      'brick': 'tile_brick.svg',
      'wood': 'tile_wood.svg',
      'sheep': 'tile_sheep.svg',
      'wheat': 'tile_wheat.svg',
      'ore': 'tile_ore.svg',
      'desert': 'tile_desert.svg',
      'port': 'tile_maritime.svg',
    };
    
    // Get the matching SVG file or use the resource name directly
    // Use relative path for GitHub Pages compatibility
    return `./assets/${resourceMap[normalizedResource] || `tile_${normalizedResource}.svg`}`;
  }
  
  // Port-specific methods
  
  // Get port ratio display text
  getPortRatio(): string {
    return `${this.portRatio}:1`;
  }
  
  // Check if this is a resource-specific port
  isResourcePort(): boolean {
    return this.isPort && !!this.portResource;
  }
  
  // Get CSS class for the port resource
  getPortResourceClass(): string {
    if (!this.portResource) return '';
    
    const normalizedResource = this.portResource.toLowerCase();
    return `resource-${normalizedResource}`;
  }

  // Get resource icon text for display
  getResourceIconText(): string {
    if (!this.portResource) return '';
    
    const normalizedResource = this.portResource.toLowerCase();
    return normalizedResource.charAt(0).toUpperCase();
  }

  // Get a transform to shift port tiles toward the edge
  getPortTileTransform(): string {
    if (!this.portDirection || !this.isPort) return '';
    
    const size = this.size;
    let x = 0;
    let y = 0;
    
    // Normalize direction to uppercase for consistent handling
    const direction = this.portDirection.toUpperCase();
    
    // Calculate movement along hexagonal axes based on direction
    // In a hexagonal grid, we need to calculate the proper offsets
    switch (direction) {
      case 'EAST':
      case 'E':
        // Move along the x-axis (60° from horizontal)
        x = size;
        break;
      case 'SOUTHEAST':
      case 'SE':
        // Move along the z-axis (120° from horizontal)
        x = size * 0.5;     // cos(120°) = -0.5, but we need positive for SE
        y = size * 0.866;   // sin(120°) = 0.866
        break;
      case 'SOUTHWEST':
      case 'SW':
        // Move along the y-axis (180° from horizontal)
        x = -size * 0.5;    // cos(180°) = -0.5
        y = size * 0.866;   // sin(180°) = 0.866
        break;
      case 'WEST':
      case 'W':
        // Opposite of East
        x = -size;
        break;
      case 'NORTHWEST':
      case 'NW':
        // Opposite of Southeast
        x = -size * 0.5;
        y = -size * 0.866;
        break;
      case 'NORTHEAST':
      case 'NE':
        // Opposite of Southwest
        x = size * 0.5;
        y = -size * 0.866;
        break;
    }
    
    // Apply 50% of the calculated offset for a more subtle effect
    return `translate(${x * 0.5}px, ${y * 0.5}px)`;
  }
  
  // Helper method to get style object for port container
  getPortTranslateStyle() {
    // Keep port indicators centered in their tiles
    return {}; // No translation applied to the indicators
  }
}
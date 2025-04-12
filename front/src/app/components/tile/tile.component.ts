import { Component, Input, EventEmitter, Output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatCardModule } from '@angular/material/card';

@Component({
  selector: 'app-tile',
  standalone: true,
  imports: [CommonModule, MatCardModule],
  template: `
    <div class="tile" 
         [ngStyle]="tileStyle"
         (click)="onClick.emit(coordinate)">
      
      <img [src]="getTileImageSrc()" class="tile-image" [alt]="getNormalizedResource() + ' tile'">
      
      <div *ngIf="number" class="number-token" [ngClass]="{ 'flashing': flashing, 'high-probability': isHighProbability() }">
        <div class="number">{{ number }}</div>
        <div class="pips">{{ numberToPips(number) }}</div>
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
  @Output() onClick = new EventEmitter<any>();
  
  // Constants
  readonly SQRT3 = 1.732;
  
  get tileStyle() {
    const w = this.SQRT3 * this.size;
    const h = 2 * this.size;
    
    const [x, y] = this.tilePixelVector();
    
    return {
      left: `${x - w/2}px`,
      top: `${y - h/2}px`,
      width: `${w}px`,
      height: `${h}px`
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
      'lumber': 'tile_wood.svg',
      'wool': 'tile_sheep.svg',
      'grain': 'tile_wheat.svg',
      'ore': 'tile_ore.svg',
      'desert': 'tile_desert.svg',
      'port': 'tile_maritime.svg',
    };
    
    // Get the matching SVG file or use the resource name directly
    return `assets/${resourceMap[normalizedResource] || `tile_${normalizedResource}.svg`}`;
  }
}
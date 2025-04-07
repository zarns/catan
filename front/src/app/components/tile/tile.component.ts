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
         [ngClass]="resource ? resource.toLowerCase() : ''"
         (click)="onClick.emit(coordinate)">
      
      <img *ngIf="resource && resource !== 'Water'" 
           [src]="getTileImageSrc(resource)" 
           alt="{{ resource }}" 
           class="tile-image">
      
      <div *ngIf="number" class="number-token" [ngClass]="{ 'flashing': flashing }">
        <div>{{ number }}</div>
        <div class="pips">{{ numberToPips(number) }}</div>
      </div>
    </div>
  `,
  styleUrls: ['./tile.component.scss']
})
export class TileComponent {
  @Input() coordinate: any;
  @Input() resource: string = '';
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
  
  // Get tile image based on resource
  getTileImageSrc(resource: string): string {
    switch (resource?.toLowerCase()) {
      case 'brick': return 'assets/tile_brick.svg';
      case 'lumber': return 'assets/tile_wood.svg';
      case 'wool': return 'assets/tile_sheep.svg';
      case 'grain': return 'assets/tile_wheat.svg';
      case 'ore': return 'assets/tile_ore.svg';
      case 'desert': return 'assets/tile_desert.svg';
      case 'water': return 'assets/tile_maritime.svg';
      default: return 'assets/tile_desert.svg';
    }
  }
} 
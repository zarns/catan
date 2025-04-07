import { Component, Input } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatIconModule } from '@angular/material/icon';

interface RobberCoordinate {
  x: number;
  y: number;
  z: number;
}

@Component({
  selector: 'app-robber',
  standalone: true,
  imports: [CommonModule, MatIconModule],
  template: `
    <div class="robber" [ngStyle]="robberStyle">
      <mat-icon>person</mat-icon>
    </div>
  `,
  styleUrls: ['./robber.component.scss']
})
export class RobberComponent {
  @Input() coordinate!: RobberCoordinate;
  @Input() size: number = 60;
  @Input() centerX: number = 0;
  @Input() centerY: number = 0;
  
  // Constants
  readonly SQRT3 = 1.732;
  
  get robberStyle() {
    const w = this.SQRT3 * this.size;
    
    const [tileX, tileY] = this.tilePixelVector();
    const [deltaX, deltaY] = [-w/2 + w/8, 0];
    
    const x = tileX + deltaX;
    const y = tileY + deltaY;
    
    return {
      left: `${x}px`,
      top: `${y}px`
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
} 
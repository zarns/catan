import { Component, Input, EventEmitter, Output } from '@angular/core';
import { CommonModule } from '@angular/common';

interface NodeCoordinate {
  x: number;
  y: number;
  z: number;
}

@Component({
  selector: 'app-node',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="node" 
         [ngClass]="direction"
         [ngStyle]="nodeStyle"
         (click)="onClick.emit(id)">
      <div *ngIf="color" [ngClass]="[color.toLowerCase(), buildingClass]" class="building"></div>
      <div *ngIf="flashing" class="pulse"></div>
    </div>
  `,
  styleUrls: ['./node.component.scss']
})
export class NodeComponent {
  @Input() id: string = '';
  @Input() coordinate!: NodeCoordinate;
  @Input() direction: string = '';
  @Input() building: string | null = null;
  @Input() color: string | null = null;
  @Input() flashing: boolean = false;
  @Input() size: number = 60;
  @Input() centerX: number = 0;
  @Input() centerY: number = 0;
  @Output() onClick = new EventEmitter<string>();
  
  // Constants
  readonly SQRT3 = 1.732;
  
  get buildingClass(): string {
    return this.building === 'city' ? 'city' : 'settlement';
  }
  
  get nodeStyle() {
    const [tileX, tileY] = this.tilePixelVector();
    const [deltaX, deltaY] = this.getNodeDelta();
    
    // Calculate the final position with the delta
    const x = tileX + deltaX;
    const y = tileY + deltaY;
    
    return {
      width: `${this.size * 0.21}px`,
      height: `${this.size * 0.21}px`,
      left: `${x}px`,
      top: `${y}px`,
      transform: 'translateY(-50%) translateX(-50%)',
      'z-index': this.building ? 13 : 3
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
  
  // Calculate the delta position based on the node direction
  getNodeDelta(): [number, number] {
    // Calculate the hex dimensions
    const w = this.SQRT3 * this.size;
    const h = 2 * this.size;
    
    // Handle both full and abbreviated direction formats
    switch(this.direction) {
      case 'NORTH':
      case 'N':
        return [0, -h/2];  // Top point
      
      case 'NORTHEAST':
      case 'NE':
        return [w/2, -h/4]; // Top-right point
      
      case 'SOUTHEAST':
      case 'SE':
        return [w/2, h/4];  // Bottom-right point
      
      case 'SOUTH':
      case 'S':
        return [0, h/2];   // Bottom point
      
      case 'SOUTHWEST':
      case 'SW':
        return [-w/2, h/4]; // Bottom-left point
      
      case 'NORTHWEST':
      case 'NW':
        return [-w/2, -h/4]; // Top-left point
      
      default:
        console.warn(`Node ${this.id} has invalid direction: "${this.direction}"`);
        return [0, 0];
    }
  }
} 
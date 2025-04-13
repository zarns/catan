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
    return this.building === 'City' ? 'city' : 'settlement';
  }
  
  get nodeStyle() {
    const [tileX, tileY] = this.tilePixelVector();
    const [deltaX, deltaY] = this.getNodeDelta();
    
    let offsetMultiplier = 1;
    // Apply a slightly larger offset for north nodes to avoid token overlap
    if (this.direction === 'NORTH') {
      offsetMultiplier = 1.05;
    }
    
    const x = tileX + deltaX * offsetMultiplier;
    const y = tileY + deltaY * offsetMultiplier;
    
    return {
      width: `${this.size * 0.5}px`,
      height: `${this.size * 0.5}px`,
      left: `${x}px`,
      top: `${y}px`,
      transform: 'translateY(-50%) translateX(-50%)',
      'background-color': this.building ? 'transparent' : '',
      'z-index': this.building ? 13 : 3,
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
    const w = this.SQRT3 * this.size;
    const h = 2 * this.size;
    
    switch(this.direction) {
      case 'NORTH':
        return [0, -h/2];
      case 'NORTHEAST':
        return [w/4, -h/4];
      case 'SOUTHEAST':
        return [w/4, h/4];
      case 'SOUTH':
        return [0, h/2];
      case 'SOUTHWEST':
        return [-w/4, h/4];
      case 'NORTHWEST':
        return [-w/4, -h/4];
      default:
        return [0, 0];
    }
  }
} 
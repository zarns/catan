import { Component, Input, EventEmitter, Output } from '@angular/core';
import { CommonModule } from '@angular/common';

interface EdgeCoordinate {
  x: number;
  y: number;
  z: number;
}

@Component({
  selector: 'app-edge',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="edge" 
         [ngClass]="direction"
         [ngStyle]="edgeStyle"
         (click)="onClick.emit(id)">
      <div *ngIf="color" [ngClass]="color.toLowerCase()" class="road"></div>
      <div *ngIf="flashing" class="pulse"></div>
    </div>
  `,
  styleUrls: ['./edge.component.scss']
})
export class EdgeComponent {
  @Input() id: string = '';
  @Input() coordinate!: EdgeCoordinate;
  @Input() direction: string = '';
  @Input() color: string | null = null;
  @Input() flashing: boolean = false;
  @Input() size: number = 60;
  @Input() centerX: number = 0;
  @Input() centerY: number = 0;
  @Output() onClick = new EventEmitter<string>();
  
  // Constants
  readonly SQRT3 = 1.732;
  
  get edgeStyle() {
    const [tileX, tileY] = this.tilePixelVector();
    const transform = this.getEdgeTransform();
    
    // Uniform transparent white for all edges
    const debugColor = this.color ? 'transparent' : 'rgba(255, 255, 255, 0.4)';
    
    return {
      left: `${tileX}px`,
      top: `${tileY}px`,
      width: `${this.size * 0.8}px`,
      height: '8px',
      transform: transform,
      'z-index': this.color ? 16 : 15,
      'background-color': debugColor,
      'border': '1px solid rgba(0, 0, 0, 0.4)',
      'border-radius': '2px'
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
  
  // Get the transform style for the edge based on direction
  getEdgeTransform(): string {
    const distanceToEdge = this.size * 0.865;
    
    // Handle both full and abbreviated direction formats
    switch(this.direction) {
      case 'NORTHEAST':
      case 'NE':
        return `translateX(-50%) translateY(-50%) rotate(30deg) translateY(${-distanceToEdge}px)`;
      
      case 'EAST':
      case 'E':
        return `translateX(-50%) translateY(-50%) rotate(90deg) translateY(${-distanceToEdge}px)`;
      
      case 'SOUTHEAST':
      case 'SE':
        return `translateX(-50%) translateY(-50%) rotate(150deg) translateY(${-distanceToEdge}px)`;
      
      case 'SOUTHWEST':
      case 'SW':
        return `translateX(-50%) translateY(-50%) rotate(210deg) translateY(${-distanceToEdge}px)`;
      
      case 'WEST':
      case 'W':
        return `translateX(-50%) translateY(-50%) rotate(270deg) translateY(${-distanceToEdge}px)`;
      
      case 'NORTHWEST':
      case 'NW':
        return `translateX(-50%) translateY(-50%) rotate(330deg) translateY(${-distanceToEdge}px)`;
      
      default:
        console.warn(`Edge ${this.id} has invalid direction: "${this.direction}"`);
        return `translateX(-50%) translateY(-50%) rotate(0deg) translateY(${-distanceToEdge}px)`;
    }
  }
} 
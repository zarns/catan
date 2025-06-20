import { Component, Input, EventEmitter, Output } from '@angular/core';
import { CommonModule } from '@angular/common';

interface EdgeCoordinate {
  x: number;
  y: number;
  z: number;
}

interface EdgeAbsoluteCoordinate {
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
      [attr.data-edge-id]="id"
      [attr.data-edge-coord]="getCoordinateString()"
      [attr.data-edge-direction]="direction"
      [attr.data-edge-color]="color"
      [attr.title]="getDebugTitle()"
      (click)="onClick.emit(id)">
      @if (color) {
        <div [ngClass]="color.toLowerCase()" class="road"></div>
      }
      @if (flashing) {
        <div class="pulse"></div>
      }
    
      <!-- Debug overlay for development -->
      @if (showDebugInfo) {
        <div class="debug-overlay">
          <div class="debug-id">{{ id }}</div>
          <div class="debug-direction">{{ direction }}</div>
        </div>
      }
    </div>
    `,
  styleUrls: ['./edge.component.scss']
})
export class EdgeComponent {
  @Input() id: string = '';
  @Input() coordinate!: EdgeCoordinate; // DEPRECATED: For backward compatibility
  @Input() node1AbsoluteCoordinate?: EdgeAbsoluteCoordinate; // NEW: Absolute position of node 1
  @Input() node2AbsoluteCoordinate?: EdgeAbsoluteCoordinate; // NEW: Absolute position of node 2
  @Input() direction: string = '';
  @Input() color: string | null = null;
  @Input() flashing: boolean = false;
  @Input() size: number = 60;
  @Input() centerX: number = 0;
  @Input() centerY: number = 0;
  @Input() showDebugInfo: boolean = false;
  @Output() onClick = new EventEmitter<string>();
  
  // Constants
  readonly SQRT3 = 1.732;
  
  get edgeStyle() {
    // Use absolute coordinates if available, otherwise fall back to tile-relative positioning
    if (this.node1AbsoluteCoordinate && this.node2AbsoluteCoordinate) {
      return this.getAbsoluteEdgeStyle();
    } else {
      // Legacy positioning for backward compatibility
      return this.getTileRelativeEdgeStyle();
    }
  }

  private getAbsoluteEdgeStyle() {
    const [x1, y1] = this.absolutePixelVector(this.node1AbsoluteCoordinate!);
    const [x2, y2] = this.absolutePixelVector(this.node2AbsoluteCoordinate!);
    
    // Calculate center point of the edge
    const centerX = (x1 + x2) / 2;
    const centerY = (y1 + y2) / 2;
    
    // Calculate length and angle
    const fullLength = Math.sqrt((x2 - x1) ** 2 + (y2 - y1) ** 2);
    const angle = Math.atan2(y2 - y1, x2 - x1) * (180 / Math.PI);
    
    // Make roads shorter - use 70% of the full distance between nodes
    const roadLength = fullLength * 0.7;
    
    // Safety check: if coordinates are invalid or length is unreasonable, fall back to tile-relative
    if (isNaN(fullLength) || isNaN(centerX) || isNaN(centerY) || fullLength === 0 || fullLength > this.size * 4) {
      console.warn(`Edge ${this.id}: Invalid absolute coordinates (length: ${fullLength}), falling back to tile-relative positioning`);
      return this.getTileRelativeEdgeStyle();
    }
    
    // Debug logging for problematic edges only
    if (fullLength > this.size * 2) {
      console.log(`Edge ${this.id}: Long edge detected - Length: ${fullLength}, connecting [${x1},${y1}] to [${x2},${y2}]`);
    }
    
    // Uniform transparent white for all edges
    const debugColor = this.color ? 'transparent' : 'rgba(255, 255, 255, 0.2)';
    
    return {
      left: `${centerX}px`,
      top: `${centerY}px`,
      width: `${roadLength}px`,
      height: '7px',
      transform: `translateX(-50%) translateY(-50%) rotate(${angle}deg)`,
      'z-index': this.color ? 16 : 15,
      'background-color': debugColor,
      'border': '1px solid rgba(0, 0, 0, 0.4)',
      'border-radius': '2px'
    };
  }

  private getTileRelativeEdgeStyle() {
    const [tileX, tileY] = this.tilePixelVector();
    const transform = this.getEdgeTransform();
    
    // Make roads shorter for tile-relative positioning too
    const roadLength = this.size * 0.75 * 0.7; // 70% of the original 75% size
    
    // Uniform transparent white for all edges
    const debugColor = this.color ? 'transparent' : 'rgba(255, 255, 255, 0.2)';
    
    return {
      left: `${tileX}px`,
      top: `${tileY}px`,
      width: `${roadLength}px`,
      height: '7px',
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

  // Convert absolute coordinates to pixel coordinates
  // This MUST match the calculation in NodeComponent.absolutePixelVector()
  absolutePixelVector(coordinate: EdgeAbsoluteCoordinate): [number, number] {
    const { x, y } = coordinate;
    const size = this.size;
    
    // Use the EXACT same calculation as NodeComponent
    // Scale and center the normalized coordinates
    const pixelX = this.centerX + (x * size);
    const pixelY = this.centerY + (y * size);
    
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
      
      case 'NORTH':
      case 'N':
        return `translateX(-50%) translateY(-50%) rotate(90deg) translateY(${-distanceToEdge}px)`;
      
      case 'SOUTHEAST':
      case 'SE':
        return `translateX(-50%) translateY(-50%) rotate(150deg) translateY(${-distanceToEdge}px)`;
      
      case 'SOUTHWEST':
      case 'SW':
        return `translateX(-50%) translateY(-50%) rotate(210deg) translateY(${-distanceToEdge}px)`;
      
      case 'SOUTH':
      case 'S':
        return `translateX(-50%) translateY(-50%) rotate(270deg) translateY(${-distanceToEdge}px)`;
      
      case 'NORTHWEST':
      case 'NW':
        return `translateX(-50%) translateY(-50%) rotate(330deg) translateY(${-distanceToEdge}px)`;
      
      default:
        console.warn(`Edge ${this.id} has invalid direction: "${this.direction}"`);
        return `translateX(-50%) translateY(-50%) rotate(0deg) translateY(${-distanceToEdge}px)`;
    }
  }
  
  getCoordinateString(): string {
    if (!this.coordinate) {
      return '';
    }
    const { x, y, z } = this.coordinate;
    return `(${x}, ${y}, ${z})`;
  }
  
  getDebugTitle(): string {
    return `Edge ID: ${this.id}, Direction: ${this.direction}, Color: ${this.color}`;
  }
} 
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
    <div
      class="edge"
      [ngClass]="getEdgeClasses()"
      [ngStyle]="edgeStyle"
      [attr.data-edge-id]="id"
      [attr.data-edge-coord]="getCoordinateString()"
      [attr.data-edge-direction]="direction"
      [attr.data-edge-color]="color"
      [attr.title]="getDebugTitle()"
      (click)="onClick.emit(id)"
    >
      <div class="edge-indicator" [ngClass]="getEdgeIndicatorClasses()">
        @if (color) {
          <div [ngClass]="color.toLowerCase()" class="road"></div>
        }
      </div>
    </div>
  `,
  styleUrls: ['./edge.component.scss'],
})
export class EdgeComponent {
  @Input() id: string = '';
  @Input() coordinate!: EdgeCoordinate; // DEPRECATED: For backward compatibility
  @Input() calculatedNodePositions?: { node1: {x: number, y: number} | null, node2: {x: number, y: number} | null }; // Calculated positions
  @Input() direction: string = '';
  @Input() color: string | null = null;
  @Input() flashing: boolean = false;
  @Input() size: number = 60;
  @Input() centerX: number = 0;
  @Input() centerY: number = 0;
  @Output() onClick = new EventEmitter<string>();

  // Constants
  readonly SQRT3 = 1.732050807568877;

  get edgeStyle() {
    // Use calculated positions from hex math
    if (this.calculatedNodePositions?.node1 && this.calculatedNodePositions?.node2) {
      return this.getCalculatedEdgeStyle();
    } else {
      // Legacy positioning for backward compatibility
      return this.getTileRelativeEdgeStyle();
    }
  }

  private getCalculatedEdgeStyle() {
    const node1 = this.calculatedNodePositions!.node1!;
    const node2 = this.calculatedNodePositions!.node2!;
    
    const x1 = node1.x;
    const y1 = node1.y;
    const x2 = node2.x;
    const y2 = node2.y;

    // Calculate edge position and rotation
    const centerX = (x1 + x2) / 2;
    const centerY = (y1 + y2) / 2;
    const deltaX = x2 - x1;
    const deltaY = y2 - y1;
    const angle = Math.atan2(deltaY, deltaX) * (180 / Math.PI);

    // ONLY positioning - let SCSS handle width/height/appearance
    return {
      left: `${centerX}px`,
      top: `${centerY}px`,
      transform: `translateX(-50%) translateY(-50%) rotate(${angle}deg)`,
    };
  }



  private getTileRelativeEdgeStyle() {
    const [tileX, tileY] = this.tilePixelVector();
    const transform = this.getEdgeTransform();

    // ONLY positioning - let SCSS handle width/height/appearance
    return {
      left: `${tileX}px`,
      top: `${tileY}px`,
      transform: transform,
    };
  }

  // Convert cube coordinates to pixel coordinates
  tilePixelVector(): [number, number] {
    if (!this.coordinate) {
      return [0, 0];
    }

    const { x, y, z } = this.coordinate;
    const size = this.size;
    
    // Use backend's coordinate conversion: axial coordinates (q, r) = (x, z)
    const q = x;
    const r = z;
    
    // Match backend's tile center calculation exactly
    const tile_center_x = size * (this.SQRT3 * q + this.SQRT3 / 2.0 * r);
    const tile_center_y = size * (3.0 / 2.0 * r);
    
    const pixelX = this.centerX + tile_center_x;
    const pixelY = this.centerY + tile_center_y;

    return [pixelX, pixelY];
  }



  // Simple edge transform for legacy positioning
  getEdgeTransform(): string {
    const directionMap: { [key: string]: number } = {
      'NORTHEAST': 30, 'NE': 30,
      'NORTH': 90, 'N': 90,
      'SOUTHEAST': 150, 'SE': 150,
      'SOUTHWEST': 210, 'SW': 210,
      'SOUTH': 270, 'S': 270,
      'NORTHWEST': 330, 'NW': 330
    };

    const rotation = directionMap[this.direction] || 0;
    return `translateX(-50%) translateY(-50%) rotate(${rotation}deg) translateY(-50px)`;
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

  getEdgeClasses(): string[] {
    const classes = [];

    if (this.flashing) {
      classes.push('flashing');
    }

    if (this.direction) {
      classes.push(this.direction);
    }

    return classes;
  }

  getEdgeIndicatorClasses(): string[] {
    const classes = [];

    if (this.color) {
      // Has a road - show as occupied
      classes.push('occupied');
    } else {
      // Empty edge - show subtle indicator
      classes.push('empty');
    }

    return classes;
  }
}

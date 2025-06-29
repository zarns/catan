import { Component, Input, EventEmitter, Output } from '@angular/core';
import { CommonModule } from '@angular/common';

interface NodeCoordinate {
  x: number;
  y: number;
  z: number;
}

interface NodeAbsoluteCoordinate {
  x: number;
  y: number;
  z: number;
}

@Component({
  selector: 'app-node',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div
      class="node"
      [ngClass]="getNodeClasses()"
      [ngStyle]="nodeStyle"
      [attr.data-node-id]="id"
      [attr.data-node-coord]="getCoordinateString()"
      [attr.data-node-direction]="direction"
      [attr.data-node-building]="building"
      [attr.data-node-color]="color"
      [attr.title]="getDebugTitle()"
      (click)="onClick.emit(id)"
    >
      <!-- Settlement/City shape - always visible -->
      <div
        class="settlement-shape"
        [ngClass]="getSettlementClasses()"
        [attr.title]="getNodeTitle()"
      ></div>

      <!-- Debug overlay for development -->
      @if (showDebugInfo) {
        <div class="debug-overlay">
          <div class="debug-id">{{ id }}</div>
          <div class="debug-direction">{{ direction }}</div>
          @if (building) {
            <div class="debug-building">{{ building }}</div>
          }
        </div>
      }
    </div>
  `,
  styleUrls: ['./node.component.scss'],
})
export class NodeComponent {
  @Input() id: string = '';
  @Input() coordinate!: NodeCoordinate; // DEPRECATED: For backward compatibility
  @Input() absoluteCoordinate?: NodeAbsoluteCoordinate; // NEW: Absolute positioning
  @Input() direction: string = '';
  @Input() building: string | null = null;
  @Input() color: string | null = null;
  @Input() flashing: boolean = false;
  @Input() size: number = 60;
  @Input() centerX: number = 0;
  @Input() centerY: number = 0;
  @Input() showDebugInfo: boolean = false;
  @Output() onClick = new EventEmitter<string>();

  // Constants
  readonly SQRT3 = 1.732;

  get buildingClass(): string {
    return this.building === 'city' ? 'city' : 'settlement';
  }

  getSettlementClasses(): string {
    const classes = ['settlement-base'];

    if (this.building && this.color) {
      // Has a building - show solid color
      classes.push('occupied');
      classes.push(this.color.toLowerCase());
      if (this.building === 'city') {
        classes.push('city');
      } else {
        classes.push('settlement');
      }
    } else {
      // Empty node - show transparent indicator
      classes.push('empty');
    }

    return classes.join(' ');
  }

  getNodeTitle(): string {
    if (this.building) {
      return `${this.building} (Player: ${this.color})`;
    } else {
      return `Node ${this.id}`;
    }
  }

  get nodeStyle() {
    let x: number, y: number;

    // Use absolute coordinates if available, otherwise fall back to tile-relative positioning
    if (this.absoluteCoordinate) {
      [x, y] = this.absolutePixelVector();
    } else {
      // Legacy positioning for backward compatibility
      const [tileX, tileY] = this.tilePixelVector();
      const [deltaX, deltaY] = this.getNodeDelta();
      x = tileX + deltaX;
      y = tileY + deltaY;
    }

    return {
      width: `${this.size * 0.21}px`,
      height: `${this.size * 0.21}px`,
      left: `${x}px`,
      top: `${y}px`,
      transform: 'translateY(-50%) translateX(-50%)',
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
    const pixelX = this.centerX + width * (x + y / 2);
    const pixelY = this.centerY + height * (3 / 4) * y;

    return [pixelX, pixelY];
  }

  // Convert absolute coordinates to pixel coordinates
  absolutePixelVector(): [number, number] {
    if (!this.absoluteCoordinate) {
      return [0, 0];
    }

    const { x, y } = this.absoluteCoordinate;
    const size = this.size;

    // Scale and center the normalized coordinates
    const pixelX = this.centerX + x * size;
    const pixelY = this.centerY + y * size;

    return [pixelX, pixelY];
  }

  // Calculate the delta position based on the node direction
  getNodeDelta(): [number, number] {
    // Calculate the hex dimensions
    const w = this.SQRT3 * this.size;
    const h = 2 * this.size;

    // Handle both full and abbreviated direction formats
    switch (this.direction) {
      case 'NORTH':
      case 'N':
        return [0, -h / 2]; // Top point

      case 'NORTHEAST':
      case 'NE':
        return [w / 2, -h / 4]; // Top-right point

      case 'SOUTHEAST':
      case 'SE':
        return [w / 2, h / 4]; // Bottom-right point

      case 'SOUTH':
      case 'S':
        return [0, h / 2]; // Bottom point

      case 'SOUTHWEST':
      case 'SW':
        return [-w / 2, h / 4]; // Bottom-left point

      case 'NORTHWEST':
      case 'NW':
        return [-w / 2, -h / 4]; // Top-left point

      default:
        console.warn(`Node ${this.id} has invalid direction: "${this.direction}"`);
        return [0, 0];
    }
  }

  getCoordinateString(): string {
    if (!this.coordinate) {
      return '';
    }
    return `${this.coordinate.x},${this.coordinate.y},${this.coordinate.z}`;
  }

  getDebugTitle(): string {
    if (!this.building) {
      return '';
    }
    return `Node ${this.id} - Building: ${this.building}`;
  }

  getNodeClasses(): string[] {
    const classes = [];

    if (this.flashing) {
      classes.push('flashing');
    }

    if (this.direction) {
      classes.push(this.direction);
    }

    return classes;
  }
}

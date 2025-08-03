import { Component, Input, Output, EventEmitter } from '@angular/core';
import { CommonModule } from '@angular/common';
import { calculateNodePixelPosition, CubeCoordinate } from '../../utils/hex-math';

@Component({
  selector: 'app-node',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div
      class="node"
      [ngClass]="getNodeClasses()"
      [style.--node-x]="nodePosition.x + 'px'"
      [style.--node-y]="nodePosition.y + 'px'"
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
  @Input() tileCoordinate?: CubeCoordinate; // Canonical tile coordinate for position calculation
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

  get nodePosition() {
    // Calculate position using hex math
    if (this.tileCoordinate && this.direction) {
      const position = calculateNodePixelPosition(this.tileCoordinate, this.direction, this.size);
      return {
        x: this.centerX + position.x,
        y: this.centerY + position.y
      };
    } else {
      // Fallback to center position if data is missing
      console.error(`❌ Node ${this.id} missing tileCoordinate or direction - using center position`);
      return {
        x: this.centerX,
        y: this.centerY
      };
    }
  }

  // REMOVED: absolutePixelVector method - no longer needed with hex math calculations

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
    if (!this.tileCoordinate) {
      return '';
    }
    return `${this.tileCoordinate.x},${this.tileCoordinate.y},${this.tileCoordinate.z}`;
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

    if (this.building) {
      classes.push('has-building');
    }

    return classes;
  }
}

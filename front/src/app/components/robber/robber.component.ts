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
      <svg viewBox="0 0 24 24" class="robber-icon">
        <path
          d="M12,2A9,9 0 0,0 3,11C3,14.03 4.53,16.82 7,18.47V22H9V19H11V22H13V19H15V22H17V18.46C19.47,16.81 21,14 21,11A9,9 0 0,0 12,2M8,11A2,2 0 0,1 10,13A2,2 0 0,1 8,15A2,2 0 0,1 6,13A2,2 0 0,1 8,11M16,11A2,2 0 0,1 18,13A2,2 0 0,1 16,15A2,2 0 0,1 14,13A2,2 0 0,1 16,11M12,14L13.5,17H10.5L12,14Z"
        />
      </svg>
    </div>
  `,
  styleUrls: ['./robber.component.scss'],
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

    // The original offset was [-w/2 + w/8, 0], we'll increase the right (x) offset
    const [deltaX, deltaY] = [-w / 1.8 + w / 4, 0]; // Changed from w/8 to w/4 to move it more to the right

    const x = tileX + deltaX;
    const y = tileY + deltaY;

    return {
      left: `${x}px`,
      top: `${y}px`,
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
}

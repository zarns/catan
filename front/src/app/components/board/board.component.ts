// src/app/components/board/board.component.ts
import { Component, Input, Output, EventEmitter, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { GameState, Position, Hex, TerrainType } from '../../models/game-types';
import { SQRT3, CATAN_LAYOUT, tilePixelVector, getHexPoints } from './coordinates';

@Component({
  selector: 'app-board',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './board.component.html',
  styleUrls: ['./board.component.scss']
})
export class BoardComponent implements OnInit {
    @Input() gameState?: GameState;
    @Output() hexClick = new EventEmitter<Position>();

    readonly HEX_SIZE = 40;  // Slightly smaller hexes
    readonly viewBoxSize = 800;  // Larger viewBox to accommodate spacing
    showDebugGrid = true;
    
    TerrainType = TerrainType;

    ngOnInit() {
        console.log('=== BOARD INITIALIZATION ===');
        this.generateDefaultHexPositions();
    }

    generateDefaultHexPositions(): Hex[] {
        console.log('=== GENERATING HEXES ===');
        const centerX = this.viewBoxSize / 2;
        const centerY = this.viewBoxSize / 2;
        
        console.log('Board Settings:', {
            HEX_SIZE: this.HEX_SIZE,
            viewBoxSize: this.viewBoxSize,
            centerX,
            centerY
        });

        console.log('Using Layout:', CATAN_LAYOUT);

        const hexes = CATAN_LAYOUT.map((coordinate, index) => {
            console.log(`Processing Hex ${index}:`, coordinate);
            const position = tilePixelVector(coordinate, this.HEX_SIZE, centerX, centerY);
            return {
                position,
                terrain: TerrainType.Desert,
                hasRobber: false
            };
        });

        console.log('Final Hex Positions:', hexes);
        return hexes;
    }

    getViewBox(): string {
        return `-${this.viewBoxSize/2} -${this.viewBoxSize/2} ${this.viewBoxSize} ${this.viewBoxSize}`;
    }

    getDebugGridLines() {
        const lines = [];
        const size = this.viewBoxSize/2;
        for (let i = -size; i <= size; i += 50) {
            lines.push(
                {x1: -size, y1: i, x2: size, y2: i},
                {x1: i, y1: -size, x2: i, y2: size}
            );
        }
        return lines;
    }

    getHexPoints(pos: Position): string {
        return getHexPoints(pos, this.HEX_SIZE);
    }

    getHexCenter(pos: Position): Position {
        return pos;
    }

    getTokenDots(number: number): string {
        switch (number) {
            case 2: case 12: return '•';
            case 3: case 11: return '••';
            case 4: case 10: return '•••';
            case 5: case 9: return '••••';
            case 6: case 8: return '•••••';
            default: return '';
        }
    }

    getHexClass(terrain: TerrainType): string {
        return terrain.toLowerCase();
    }

    onHexClick(hex: Hex): void {
        this.hexClick.emit(hex.position);
    }
}
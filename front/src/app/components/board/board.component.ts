import { Component, Input, Output, EventEmitter, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { GameState, Position, Hex, TerrainType } from '../../models/game-types';
import { CATAN_LAYOUT } from './coordinates';

@Component({
  selector: 'app-board',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './board.component.html',
  styleUrls: ['./board.component.scss']
})
export class BoardComponent implements OnInit {
  @Input() gameState: GameState | null | undefined;
  @Input() isCurrentPlayer = false;
  @Output() hexClick = new EventEmitter<Position>();
  @Output() settlementClick = new EventEmitter<Position>();
  @Output() roadClick = new EventEmitter<{start: Position, end: Position}>();

  readonly HEX_SIZE = 40;
  readonly viewBoxSize = 800;
  showDebugGrid = false;
  
  TerrainType = TerrainType;

  processedSettlements = new Set<string>();
  processedRoads = new Set<string>();
  validSettlementPositions = new Set<string>();

  hexes: Hex[] = [];

  ngOnInit() {
    this.hexes = this.generateDefaultHexPositions();
  }

  generateDefaultHexPositions(): Hex[] {
    this.processedSettlements.clear();
    this.processedRoads.clear();

    const terrains: TerrainType[] = [
      TerrainType.Fields, TerrainType.Hills, TerrainType.Pasture, TerrainType.Forest, TerrainType.Mountains,
      TerrainType.Fields, TerrainType.Hills, TerrainType.Pasture, TerrainType.Forest, TerrainType.Mountains,
      TerrainType.Fields, TerrainType.Hills, TerrainType.Pasture, TerrainType.Forest, TerrainType.Mountains,
      TerrainType.Fields, TerrainType.Pasture, TerrainType.Desert, TerrainType.Forest
    ];

    const tokens: (number | null)[] = [
      5, 2, 6, 3, 8,
      10, 9, 12, 11, 4,
      8, 10, 9, 4, 5,
      6, 3, null, 11
    ];

    return CATAN_LAYOUT.map((coordinate, index) => ({
      position: this.hexToPixel(coordinate),
      terrain: terrains[index],
      hasRobber: terrains[index] === TerrainType.Desert,
      token: tokens[index]
    }));
  }

  hexToPixel(hex: { q: number; r: number }): Position {
    const x = this.HEX_SIZE * (Math.sqrt(3) * hex.q + Math.sqrt(3)/2 * hex.r);
    const y = this.HEX_SIZE * ((3/2) * hex.r);
    return { x, y };
  }

  getViewBox(): string {
    return `-${this.viewBoxSize / 2} -${this.viewBoxSize / 2} ${this.viewBoxSize} ${this.viewBoxSize}`;
  }

  getHexPoints(pos: Position): string {
    const points = [];
    const size = this.HEX_SIZE;
    for (let i = 0; i < 6; i++) {
      const angle_deg = 60 * i + 30;
      const angle_rad = Math.PI / 180 * angle_deg;
      const x = pos.x + size * Math.cos(angle_rad);
      const y = pos.y + size * Math.sin(angle_rad);
      points.push(`${x},${y}`);
    }
    return points.join(' ');
  }

  getHexClass(terrain: TerrainType): string {
    return terrain.toLowerCase();
  }

  onHexClick(hex: Hex): void {
    this.hexClick.emit(hex.position);
  }

  getHexCorners(center: Position): Position[] {
    const corners: Position[] = [];
    const size = this.HEX_SIZE;
    for (let i = 0; i < 6; i++) {
      const angle_deg = 60 * i + 30;
      const angle_rad = Math.PI / 180 * angle_deg;
      const x = center.x + size * Math.cos(angle_rad);
      const y = center.y + size * Math.sin(angle_rad);
      corners.push({ x, y });
    }
    return corners;
  }

  getHexEdges(center: Position): { start: Position; end: Position }[] {
    const corners = this.getHexCorners(center);
    return corners.map((start, i) => ({
      start,
      end: corners[(i + 1) % 6]
    }));
  }

  getUniqueSettlementPositions(hex: Hex): Position[] {
    const settlements = this.getHexCorners(hex.position);
    return settlements.filter(pos => {
      const key = this.getPositionKey(pos);
      if (this.processedSettlements.has(key)) return false;
      this.processedSettlements.add(key);
      return true;
    });
  }

  getUniqueRoadPositions(hex: Hex): { start: Position; end: Position }[] {
    const edges = this.getHexEdges(hex.position);
    return edges.filter(edge => {
      const startKey = this.getPositionKey(edge.start);
      const endKey = this.getPositionKey(edge.end);
      const key = startKey < endKey ? `${startKey}-${endKey}` : `${endKey}-${startKey}`;
      if (this.processedRoads.has(key)) return false;
      this.processedRoads.add(key);
      return true;
    });
  }

  getHexCenter(pos: Position): Position {
    return pos;
  }

  getTokenDots(number: number): string {
    const dots: Record<number, string> = {
      2: '•', 12: '•',
      3: '••', 11: '••',
      4: '•••', 10: '•••',
      5: '••••', 9: '••••',
      6: '•••••', 8: '•••••'
    };
    return dots[number] || '';
  }

  getRoadPoints(start: Position, end: Position): string {
    const roadWidth = 6;
    const angle = Math.atan2(end.y - start.y, end.x - start.x);
    const offsetX = (roadWidth / 2) * Math.sin(angle);
    const offsetY = (roadWidth / 2) * -Math.cos(angle);

    return [
      `${start.x + offsetX},${start.y + offsetY}`,
      `${start.x - offsetX},${start.y - offsetY}`,
      `${end.x - offsetX},${end.y - offsetY}`,
      `${end.x + offsetX},${end.y + offsetY}`
    ].join(' ');
  }

  getSettlementPoints(pos: Position): string {
    const size = 8;
    const halfSize = size / 2;
    
    return [
      `${pos.x - halfSize},${pos.y - halfSize}`,
      `${pos.x + halfSize},${pos.y - halfSize}`,
      `${pos.x + halfSize},${pos.y + halfSize}`,
      `${pos.x - halfSize},${pos.y + halfSize}`
    ].join(' ');
  }

  private getPositionKey(pos: Position): string {
    return `${pos.x.toFixed(2)},${pos.y.toFixed(2)}`;
  }

  onSettlementClick(position: Position): void {
    this.settlementClick.emit(position);
  }

  onRoadClick(road: {start: Position, end: Position}): void {
    this.roadClick.emit(road);
  }

  getSettlementClasses(pos: Position): string {
    const key = this.getPositionKey(pos);
    const isValid = this.validSettlementPositions.has(key);
    const isPlaced = this.gameState?.board.settlements.has(pos);
    
    return `settlement ${isValid ? 'valid' : ''} ${isPlaced ? 'placed' : ''}`;
  }
}
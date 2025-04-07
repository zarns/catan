import { Component, Input, OnInit, CUSTOM_ELEMENTS_SCHEMA } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatSidenavModule } from '@angular/material/sidenav';
import { MatDividerModule } from '@angular/material/divider';
import { MatCardModule } from '@angular/material/card';
import { PlayerStateBoxComponent } from '../player-state-box/player-state-box.component';

@Component({
  selector: 'app-left-drawer',
  standalone: true,
  imports: [
    CommonModule,
    MatSidenavModule,
    MatDividerModule,
    MatCardModule,
    PlayerStateBoxComponent
  ],
  schemas: [CUSTOM_ELEMENTS_SCHEMA],
  template: `
    <div class="left-drawer" [class.mobile]="isMobile" [class.open]="isOpen">
      <div class="drawer-content">
        <!-- Player sections -->
        <ng-container *ngIf="gameState && gameState.game">
          <div *ngFor="let player of gameState.game.players; let i = index" 
               class="player-section" 
               [ngClass]="{'current-player': i === gameState.game.current_player_index}">
            <app-player-state-box
              [playerState]="gameState.player_state"
              [playerKey]="player.color.toLowerCase()"
              [color]="player.color">
            </app-player-state-box>
            <mat-divider></mat-divider>
          </div>
        </ng-container>
        
        <!-- Action log -->
        <div class="log" *ngIf="gameState && gameState.actions">
          <div 
            *ngFor="let action of getReversedActions()"
            class="action foreground"
            [ngClass]="action[0]?.toLowerCase()">
            {{ humanizeAction(action) }}
          </div>
        </div>
      </div>
    </div>
  `,
  styleUrls: ['./left-drawer.component.scss']
})
export class LeftDrawerComponent implements OnInit {
  @Input() gameState: any;
  @Input() isOpen: boolean = true;
  @Input() isMobile: boolean = false;

  constructor() { }

  ngOnInit(): void {
  }

  getReversedActions(): any[] {
    if (!this.gameState || !this.gameState.actions) {
      return [];
    }
    return [...this.gameState.actions].reverse();
  }

  humanizeAction(action: any[]): string {
    if (!this.gameState) return '';

    const botColors = this.gameState.bot_colors || [];
    const player = botColors.includes(action[0]) ? "BOT" : "YOU";
    
    switch (action[1]) {
      case "ROLL":
        return `${player} ROLLED A ${action[2][0] + action[2][1]}`;
      case "DISCARD":
        return `${player} DISCARDED`;
      case "BUY_DEVELOPMENT_CARD":
        return `${player} BOUGHT DEVELOPMENT CARD`;
      case "BUILD_SETTLEMENT":
      case "BUILD_CITY": {
        const parts = action[1].split("_");
        const building = parts[parts.length - 1];
        const tileId = action[2];
        const tiles = this.gameState.adjacent_tiles?.[tileId] || [];
        const tileString = tiles.map((t: any) => this.getShortTileString(t)).join("-");
        return `${player} BUILT ${building} ON ${tileString}`;
      }
      case "BUILD_ROAD": {
        const edge = action[2];
        if (!this.gameState.adjacent_tiles) return `${player} BUILT ROAD`;
        
        const a = this.gameState.adjacent_tiles[edge[0]]?.map((t: any) => t.id) || [];
        const b = this.gameState.adjacent_tiles[edge[1]]?.map((t: any) => t.id) || [];
        const intersection = a.filter((t: string) => b.includes(t));
        
        if (intersection.length === 0 || !this.gameState.tiles) {
          return `${player} BUILT ROAD`;
        }
        
        const tiles = intersection.map(
          (tileId: string) => this.findTileById(this.gameState, tileId)?.tile
        ).filter(Boolean);
        
        const edgeString = tiles.map((tile: any) => this.getShortTileString(tile)).join("-");
        return `${player} BUILT ROAD ON ${edgeString}`;
      }
      case "PLAY_KNIGHT_CARD": {
        return `${player} PLAYED KNIGHT CARD`;
      }
      case "PLAY_ROAD_BUILDING": {
        return `${player} PLAYED ROAD BUILDING`;
      }
      case "PLAY_MONOPOLY": {
        return `${player} MONOPOLIZED ${action[2]}`;
      }
      case "PLAY_YEAR_OF_PLENTY": {
        const firstResource = action[2][0];
        const secondResource = action[2][1];
        if (secondResource) {
          return `${player} PLAYED YEAR OF PLENTY. CLAIMED ${firstResource} AND ${secondResource}`;
        } else {
          return `${player} PLAYED YEAR OF PLENTY. CLAIMED ${firstResource}`;
        }
      }
      case "MOVE_ROBBER": {
        const tile = this.findTileByCoordinate(this.gameState, action[2][0]);
        const tileString = this.getTileString(tile);
        const stolenResource = action[2][2] ? ` (STOLE ${action[2][2]})` : '';
        return `${player} ROBBED ${tileString}${stolenResource}`;
      }
      case "MARITIME_TRADE": {
        const label = this.humanizeTradeAction(action);
        return `${player} TRADED ${label}`;
      }
      case "END_TURN":
        return `${player} ENDED TURN`;
      default:
        return `${player} ${action.slice(1)}`;
    }
  }

  humanizeTradeAction(action: any[]): string {
    if (!action[2] || !Array.isArray(action[2])) return '';
    const out = action[2].slice(0, 4).filter((resource: string) => resource !== null);
    return `${out.length} ${out[0]} => ${action[2][4]}`;
  }

  findTileByCoordinate(gameState: any, coordinate: any): any {
    if (!gameState.tiles) return null;
    
    for (const tile of Object.values(gameState.tiles)) {
      if (JSON.stringify((tile as any).coordinate) === JSON.stringify(coordinate)) {
        return tile;
      }
    }
    return null;
  }

  findTileById(gameState: any, tileId: string): any {
    if (!gameState.tiles) return null;
    return gameState.tiles[tileId];
  }

  getTileString(tile: any): string {
    if (!tile || !tile.tile) return '';
    const { number = "THE", resource = "DESERT" } = tile.tile;
    return `${number} ${resource}`;
  }

  getShortTileString(tileTile: any): string {
    if (!tileTile) return '';
    return tileTile.number || tileTile.type;
  }
} 
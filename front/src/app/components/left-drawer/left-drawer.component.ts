import { Component, Input, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { PlayerStateBoxComponent } from '../player-state-box/player-state-box.component';
import { GameLogComponent } from '../game-log/game-log.component';

@Component({
  selector: 'app-left-drawer',
  standalone: true,
  imports: [CommonModule, PlayerStateBoxComponent, GameLogComponent],

  template: `
    <div
      class="left-drawer"
      [class.mobile]="isMobile"
      [class.desktop]="!isMobile"
      [class.open]="isOpen"
    >
      <div class="drawer-content">
        <!-- Player sections -->
        @if (gameState && gameState.game) {
          @for (player of gameState.game.players; track player.id; let i = $index) {
            <div
              class="player-section"
              [ngClass]="{ 'current-player': i === gameState.game.current_player_index }"
            >
              <app-player-state-box
                [playerState]="gameState"
                [playerKey]="player.color.toLowerCase()"
                [color]="player.color"
                [isCurrentPlayer]="i === gameState.game.current_player_index"
                [isBot]="isPlayerBot(player.color)"
              >
              </app-player-state-box>
              <div class="divider"></div>
            </div>
          }
        }

        <!-- Enhanced Game Log Component -->
        <app-game-log [gameState]="gameState"></app-game-log>
      </div>
    </div>
  `,
  styleUrls: ['./left-drawer.component.scss'],
})
export class LeftDrawerComponent implements OnInit {
  @Input() gameState: any;
  @Input() isOpen: boolean = true;
  @Input() isMobile: boolean = false;

  constructor() {}

  ngOnInit(): void {}

  isPlayerBot(playerColor: string): boolean {
    if (!this.gameState || !this.gameState.bot_colors) {
      return false;
    }
    return this.gameState.bot_colors.includes(playerColor);
  }
}

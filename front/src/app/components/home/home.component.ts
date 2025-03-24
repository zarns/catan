import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Router } from '@angular/router';
import { MatButtonModule } from '@angular/material/button';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';

enum GameMode {
  HUMAN_VS_CATANATRON = 'HUMAN_VS_CATANATRON',
  RANDOM_BOTS = 'RANDOM_BOTS',
  CATANATRON_BOTS = 'CATANATRON_BOTS'
}

@Component({
  selector: 'app-home',
  standalone: true,
  imports: [CommonModule, MatButtonModule, MatProgressSpinnerModule],
  template: `
    <div class="home-container">
      <h1 class="logo">CATANATRON</h1>

      <div class="switchable">
        <ng-container *ngIf="!loading; else loadingTemplate">
          <div class="game-rules">
            <div>OPEN HAND</div>
            <div>NO CHOICE DURING DISCARD</div>
          </div>
          
          <div class="player-count-section">
            <div class="section-title">Number of Players</div>
            <div class="button-group">
              <button class="player-button" [class.selected]="numPlayers === 2" (click)="setNumPlayers(2)">
                2 PLAYERS
              </button>
              <button class="player-button" [class.selected]="numPlayers === 3" (click)="setNumPlayers(3)">
                3 PLAYERS
              </button>
              <button class="player-button" [class.selected]="numPlayers === 4" (click)="setNumPlayers(4)">
                4 PLAYERS
              </button>
            </div>
          </div>

          <div class="action-buttons">
            <button class="action-button" (click)="startGame(GameMode.HUMAN_VS_CATANATRON)">
              PLAY AGAINST CATANATRON
            </button>

            <button class="action-button" (click)="startGame(GameMode.RANDOM_BOTS)">
              WATCH RANDOM BOTS
            </button>
          </div>
        </ng-container>

        <ng-template #loadingTemplate>
          <mat-spinner diameter="60"></mat-spinner>
        </ng-template>
      </div>
    </div>
  `,
  styles: [`
    :host {
      display: block;
      height: 100vh;
      background-color: #000000;
    }

    .home-container {
      display: flex;
      flex-direction: column;
      align-items: center;
      padding-top: 10vh;
      gap: 2rem;
    }

    .logo {
      font-family: 'Bungee Inline', cursive;
      font-size: 4rem;
      margin: 0;
      color: #5643fd;
      text-shadow: 0 0 20px #ff00ff, 0 0 30px #ff00ff80;
      letter-spacing: 4px;
    }

    .game-rules {
      font-family: 'Roboto Mono', monospace;
      color: white;
      text-align: center;
      margin: 2rem 0;
      font-size: 1rem;
      letter-spacing: 1px;
      line-height: 1.5;
    }

    .section-title {
      font-family: 'Roboto Mono', monospace;
      color: white;
      font-size: 1.2rem;
      margin-bottom: 1rem;
      text-align: center;
    }

    .button-group {
      display: flex;
      gap: 1rem;
      margin-bottom: 2rem;
    }

    .player-button {
      font-family: 'Roboto Mono', monospace;
      padding: 0.5rem 1rem;
      min-width: 120px;
      background-color: #1e0933;
      color: white;
      border: 1px solid #7d33cc;
      cursor: pointer;
      font-size: 1rem;
      box-shadow: 0 0 10px rgba(255, 0, 255, 0.3);
      transition: all 0.2s ease;
    }

    .player-button:hover {
      box-shadow: 0 0 15px rgba(255, 0, 255, 0.5);
      background-color: #2a0f47;
    }

    .player-button.selected {
      background-color: #4a0fab;
      border-color: #ff00ff;
      box-shadow: 0 0 15px rgba(255, 0, 255, 0.5), inset 0 0 10px rgba(255, 0, 255, 0.3);
    }

    .action-buttons {
      display: flex;
      flex-direction: column;
      gap: 1rem;
      align-items: center;
    }

    .action-button {
      font-family: 'Roboto Mono', monospace;
      padding: 0.75rem 1.5rem;
      min-width: 300px;
      background-color: transparent;
      color: white;
      border: 2px solid #ff00ff;
      cursor: pointer;
      font-size: 1rem;
      box-shadow: 0 0 15px rgba(255, 0, 255, 0.3);
      transition: all 0.3s ease;
    }

    .action-button:hover {
      box-shadow: 0 0 20px rgba(255, 0, 255, 0.5);
      background-color: rgba(255, 0, 255, 0.15);
      text-shadow: 0 0 5px rgba(255, 255, 255, 0.7);
    }

    mat-spinner ::ng-deep circle {
      stroke: #ff00ff;
    }
  `]
})
export class HomeComponent {
  GameMode = GameMode;
  loading = false;
  numPlayers = 2;

  constructor(private router: Router) {}

  setNumPlayers(count: number) {
    this.numPlayers = count;
  }

  async startGame(gameMode: GameMode) {
    this.loading = true;
    // TODO: Implement WebSocket connection and game creation
    // For now, just navigate to a placeholder route
    await new Promise(resolve => setTimeout(resolve, 1000)); // Simulate API call
    this.router.navigate(['/game']);
    this.loading = false;
  }
} 
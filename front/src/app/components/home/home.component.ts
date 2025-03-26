import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatButtonModule } from '@angular/material/button';
import { MatCardModule } from '@angular/material/card';
import { Router } from '@angular/router';
import { GameService } from '../../services/game.service';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';

// Define GameMode enum locally in the component
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
            <button class="action-button primary" (click)="startGame(GameMode.HUMAN_VS_CATANATRON)">
              PLAY AGAINST CATANATRON
            </button>

            <button class="action-button secondary" (click)="startGame(GameMode.RANDOM_BOTS)">
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
      background-color: #0a1c0a;
      background-image: 
        radial-gradient(rgba(30, 90, 90, 0.3) 1px, transparent 1px), 
        radial-gradient(rgba(30, 90, 90, 0.2) 1px, transparent 1px);
      background-size: 40px 40px;
      background-position: 0 0, 20px 20px;
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
      color: #f3ea15;
      text-shadow: 0 0 15px rgba(243, 234, 21, 0.5), 0 0 20px rgba(243, 234, 21, 0.2);
      letter-spacing: 4px;
      transform: skew(-5deg);
    }

    .game-rules {
      font-family: 'Roboto Mono', monospace;
      color: #f3ea15;
      text-align: center;
      margin: 2rem 0;
      font-size: 1rem;
      letter-spacing: 1px;
      line-height: 1.5;
    }

    .section-title {
      font-family: 'Roboto Mono', monospace;
      color: #f3ea15;
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
      background-color: #1e5a5a;
      color: white;
      border: 1px solid #63a375;
      cursor: pointer;
      font-size: 1rem;
      box-shadow: 0 0 10px rgba(99, 163, 117, 0.3);
      transition: all 0.2s ease;
    }

    .player-button:hover {
      box-shadow: 0 0 15px rgba(99, 163, 117, 0.5);
      background-color: #2a6b6b;
    }

    .player-button.selected {
      background-color: #63a375;
      border-color: #f3ea15;
      color: #0a1c0a;
      font-weight: bold;
      box-shadow: 0 0 15px rgba(243, 234, 21, 0.5), inset 0 0 10px rgba(243, 234, 21, 0.3);
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
      border: 2px solid;
      cursor: pointer;
      font-size: 1rem;
      transition: all 0.3s ease;
      position: relative;
      overflow: hidden;
    }

    .action-button.primary {
      border-color: #f3ea15;
      box-shadow: 0 0 15px rgba(243, 234, 21, 0.3);
    }

    .action-button.secondary {
      border-color: #1e5a5a;
      box-shadow: 0 0 15px rgba(30, 90, 90, 0.3);
    }

    .action-button:hover {
      transform: translateY(-2px);
    }

    .action-button.primary:hover {
      box-shadow: 0 0 20px rgba(243, 234, 21, 0.5);
      background-color: rgba(243, 234, 21, 0.15);
      color: #f3ea15;
    }

    .action-button.secondary:hover {
      box-shadow: 0 0 20px rgba(30, 90, 90, 0.5);
      background-color: rgba(30, 90, 90, 0.15);
      color: #63a375;
    }

    .action-button::after {
      content: '';
      position: absolute;
      top: -50%;
      left: -50%;
      width: 200%;
      height: 200%;
      background: linear-gradient(
        to bottom right,
        rgba(243, 234, 21, 0) 0%,
        rgba(243, 234, 21, 0.1) 50%,
        rgba(243, 234, 21, 0) 100%
      );
      transform: rotate(45deg);
      opacity: 0;
      transition: opacity 0.3s;
    }

    .action-button:hover::after {
      opacity: 1;
      animation: shine 1.5s infinite;
    }

    @keyframes shine {
      0% {
        top: -50%;
        left: -50%;
      }
      100% {
        top: 150%;
        left: 150%;
      }
    }

    mat-spinner ::ng-deep circle {
      stroke: #f3ea15;
    }
  `]
})
export class HomeComponent implements OnInit {
  GameMode = GameMode;
  loading = false;
  numPlayers = 2;

  constructor(
    private router: Router,
    private gameService: GameService
  ) {}

  ngOnInit() {}

  setNumPlayers(count: number) {
    this.numPlayers = count;
  }

  async startGame(gameMode: GameMode) {
    this.loading = true;
    
    // Map local enum to service enum
    const serviceGameMode = this.mapGameMode(gameMode);
    
    // Call the backend API to create a game
    this.gameService.createGame({
      mode: serviceGameMode,
      num_players: this.numPlayers
    }).subscribe({
      next: (gameState) => {
        console.log('Game created:', gameState);
        this.router.navigate(['/game', gameState.id]);
        this.loading = false;
      },
      error: (error) => {
        console.error('Error creating game:', error);
        this.loading = false;
      }
    });
  }

  // Map local GameMode enum to service string literals
  private mapGameMode(mode: GameMode): 'HUMAN_VS_CATANATRON' | 'RANDOM_BOTS' | 'CATANATRON_BOTS' {
    switch (mode) {
      case GameMode.HUMAN_VS_CATANATRON:
        return 'HUMAN_VS_CATANATRON';
      case GameMode.RANDOM_BOTS:
        return 'RANDOM_BOTS';
      case GameMode.CATANATRON_BOTS:
        return 'CATANATRON_BOTS';
      default:
        return 'RANDOM_BOTS';
    }
  }
} 
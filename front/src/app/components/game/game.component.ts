import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatCardModule } from '@angular/material/card';
import { ActivatedRoute } from '@angular/router';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { GameService, GameState } from '../../services/game.service';

@Component({
  selector: 'app-game',
  standalone: true,
  imports: [CommonModule, MatCardModule, MatProgressSpinnerModule],
  template: `
    <div class="game-container">
      <ng-container *ngIf="gameState; else loading">
        <div class="game-board">
          <h2>Game ID: {{ gameState.id }}</h2>
          <p>Status: {{ gameState.status }}</p>
          <!-- Game board will go here -->
        </div>
        <div class="game-info">
          <h2>Game Info</h2>
          <!-- Player info, resources, etc. will go here -->
        </div>
      </ng-container>

      <ng-template #loading>
        <div class="loading-container">
          <mat-spinner diameter="60"></mat-spinner>
          <p>Loading game...</p>
        </div>
      </ng-template>
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

    .game-container {
      display: grid;
      grid-template-columns: 1fr 300px;
      gap: 2rem;
      padding: 2rem;
      height: 100%;
      color: #f3ea15;
      font-family: 'Roboto Mono', monospace;
    }

    .game-board {
      background: rgba(30, 90, 90, 0.2);
      border: 1px solid #63a375;
      box-shadow: 0 0 15px rgba(99, 163, 117, 0.3);
      padding: 2rem;
      border-radius: 4px;
    }

    .game-info {
      background: rgba(30, 90, 90, 0.2);
      border: 1px solid #63a375;
      box-shadow: 0 0 15px rgba(99, 163, 117, 0.3);
      padding: 2rem;
      border-radius: 4px;
    }

    h2 {
      color: #f3ea15;
      margin-top: 0;
      font-family: 'Roboto Mono', monospace;
      letter-spacing: 1px;
    }

    .loading-container {
      position: absolute;
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%);
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 2rem;
    }

    .loading-container p {
      color: #f3ea15;
      font-family: 'Roboto Mono', monospace;
    }

    mat-spinner ::ng-deep circle {
      stroke: #f3ea15;
    }
  `]
})
export class GameComponent implements OnInit {
  gameState: GameState | null = null;
  gameId: string | null = null;

  constructor(
    private route: ActivatedRoute,
    private gameService: GameService
  ) {}

  ngOnInit() {
    // Get the game ID from route parameters
    this.route.paramMap.subscribe(params => {
      this.gameId = params.get('id');
      
      if (this.gameId) {
        // Fetch game state if we have an ID
        this.gameService.getGameState(this.gameId).subscribe({
          next: (state) => {
            this.gameState = state;
            console.log('Game state loaded:', state);
          },
          error: (error) => {
            console.error('Error loading game state:', error);
          }
        });
      } else {
        // Create a placeholder for now if we don't have an ID
        this.gameState = {
          id: 'placeholder',
          status: 'waiting'
        };
      }
    });
  }
} 
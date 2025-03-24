import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatCardModule } from '@angular/material/card';

@Component({
  selector: 'app-game',
  standalone: true,
  imports: [CommonModule, MatCardModule],
  template: `
    <div class="game-container">
      <mat-card class="game-board">
        <mat-card-content>
          <h2>Game Board</h2>
          <!-- Game board will go here -->
        </mat-card-content>
      </mat-card>
      <mat-card class="game-info">
        <mat-card-content>
          <h2>Game Info</h2>
          <!-- Player info, resources, etc. will go here -->
        </mat-card-content>
      </mat-card>
    </div>
  `,
  styles: [`
    .game-container {
      display: grid;
      grid-template-columns: 1fr 300px;
      gap: 2rem;
      padding: 2rem;
      height: 100%;
    }

    .game-board {
      background: #f0f0f0;
    }

    .game-info {
      background: #e0e0e0;
    }

    mat-card {
      height: 100%;
    }
  `]
})
export class GameComponent implements OnInit {
  constructor() {}

  ngOnInit() {
    // Initialize game state and WebSocket connection
  }
} 
import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatCardModule } from '@angular/material/card';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatBadgeModule } from '@angular/material/badge';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { ActivatedRoute } from '@angular/router';
import { GameService, GameState, Player } from '../../services/game.service';
import { WebsocketService } from '../../services/websocket.service';
import { Subscription } from 'rxjs';

@Component({
  selector: 'app-game',
  standalone: true,
  imports: [
    CommonModule, 
    MatCardModule, 
    MatProgressSpinnerModule,
    MatBadgeModule,
    MatButtonModule,
    MatIconModule
  ],
  template: `
    <div class="game-container">
      <ng-container *ngIf="gameState; else loading">
        <div class="game-board">
          <div class="game-header">
            <h2>Game: {{ gameState.id }}</h2>
            <div class="game-status" [ngClass]="gameState.status">
              Status: {{ gameState.status | titlecase }}
            </div>
          </div>
          
          <div class="board-visualization">
            <!-- This will be replaced with an actual board visualization later -->
            <div class="placeholder-board">
              <div *ngIf="gameState.game" class="game-turn">
                Turn: {{ gameState.game.turns }}
              </div>
              <div class="board-placeholder">
                Game Board Visualization
                <div class="dice-area" *ngIf="gameState.game">
                  <div *ngIf="gameState.game.dice_rolled" class="dice rolled">
                    Dice Rolled
                  </div>
                  <div *ngIf="!gameState.game.dice_rolled" class="dice not-rolled">
                    Waiting for Dice Roll
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
        
        <div class="game-info">
          <h2>Players</h2>
          <div class="players-list" *ngIf="gameState.game">
            <div *ngFor="let player of gameState.game.players; let i = index" 
                 class="player-card"
                 [class.active]="i === gameState.game.current_player_index"
                 [style.border-color]="player.color">
              <div class="player-name">{{ player.name }}</div>
              <div class="player-stats">
                <div class="stat">
                  <mat-icon>stars</mat-icon>
                  <span>{{ player.victory_points }}</span>
                </div>
                <div class="stat">
                  <mat-icon>security</mat-icon>
                  <span>{{ player.knights_played }}</span>
                </div>
              </div>
              <div class="player-resources" *ngIf="player.resources">
                <div class="resource" *ngFor="let resource of getResourceEntries(player)">
                  <div class="resource-icon {{ resource.name }}"></div>
                  <div class="resource-count">{{ resource.count }}</div>
                </div>
              </div>
              <div class="player-achievement" *ngIf="player.longest_road">
                <mat-icon>timeline</mat-icon> Longest Road
              </div>
              <div class="player-achievement" *ngIf="player.largest_army">
                <mat-icon>military_tech</mat-icon> Largest Army
              </div>
            </div>
          </div>
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
      display: flex;
      flex-direction: column;
    }
    
    .game-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 1rem;
    }
    
    .game-status {
      padding: 0.5rem 1rem;
      border-radius: 4px;
      text-transform: uppercase;
      font-size: 0.8rem;
      letter-spacing: 1px;
    }
    
    .game-status.waiting {
      background-color: #2a6b6b;
      border: 1px solid #63a375;
    }
    
    .game-status.in_progress {
      background-color: #1b5e20;
      border: 1px solid #f3ea15;
    }
    
    .game-status.finished {
      background-color: #7d33cc;
      border: 1px solid #ff00ff;
    }
    
    .board-visualization {
      flex: 1;
      display: flex;
      align-items: center;
      justify-content: center;
    }
    
    .placeholder-board {
      width: 100%;
      height: 100%;
      display: flex;
      flex-direction: column;
    }
    
    .game-turn {
      text-align: center;
      margin-bottom: 1rem;
      font-size: 1.2rem;
    }
    
    .board-placeholder {
      flex: 1;
      border: 2px dashed #63a375;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      font-size: 1.5rem;
      color: rgba(243, 234, 21, 0.7);
      position: relative;
    }
    
    .dice-area {
      position: absolute;
      bottom: 2rem;
      padding: 1rem;
      border-radius: 4px;
    }
    
    .dice {
      padding: 0.5rem 1rem;
      border-radius: 4px;
      text-align: center;
    }
    
    .dice.rolled {
      background-color: #1b5e20;
      border: 1px solid #f3ea15;
    }
    
    .dice.not-rolled {
      background-color: #7d33cc;
      border: 1px solid #ff00ff;
    }

    .game-info {
      background: rgba(30, 90, 90, 0.2);
      border: 1px solid #63a375;
      box-shadow: 0 0 15px rgba(99, 163, 117, 0.3);
      padding: 2rem;
      border-radius: 4px;
      overflow-y: auto;
    }

    h2 {
      color: #f3ea15;
      margin-top: 0;
      font-family: 'Roboto Mono', monospace;
      letter-spacing: 1px;
      margin-bottom: 1.5rem;
    }
    
    .players-list {
      display: flex;
      flex-direction: column;
      gap: 1rem;
    }
    
    .player-card {
      background-color: rgba(10, 28, 10, 0.8);
      border: 2px solid;
      border-radius: 4px;
      padding: 1rem;
      transition: all 0.3s ease;
    }
    
    .player-card.active {
      box-shadow: 0 0 15px #f3ea15;
      transform: scale(1.02);
    }
    
    .player-name {
      font-weight: bold;
      margin-bottom: 0.5rem;
      font-size: 1.1rem;
    }
    
    .player-stats {
      display: flex;
      gap: 1rem;
      margin-bottom: 0.5rem;
    }
    
    .stat {
      display: flex;
      align-items: center;
      gap: 0.25rem;
    }
    
    .player-resources {
      display: flex;
      flex-wrap: wrap;
      gap: 0.5rem;
      margin-top: 1rem;
    }
    
    .resource {
      display: flex;
      align-items: center;
      gap: 0.25rem;
      background-color: rgba(30, 90, 90, 0.3);
      padding: 0.25rem 0.5rem;
      border-radius: 4px;
      min-width: 60px;
    }
    
    .resource-icon {
      width: 16px;
      height: 16px;
      border-radius: 50%;
    }
    
    .resource-icon.brick {
      background-color: #d32f2f;
    }
    
    .resource-icon.lumber {
      background-color: #388e3c;
    }
    
    .resource-icon.wool {
      background-color: #9e9e9e;
    }
    
    .resource-icon.grain {
      background-color: #fdd835;
    }
    
    .resource-icon.ore {
      background-color: #616161;
    }
    
    .player-achievement {
      margin-top: 0.5rem;
      display: flex;
      align-items: center;
      gap: 0.25rem;
      font-size: 0.8rem;
      color: #f3ea15;
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
export class GameComponent implements OnInit, OnDestroy {
  gameState: GameState | null = null;
  error: string | null = null;
  loading = true;
  gameId: string | null = null;
  greeting: string = '';
  
  private subscription: Subscription = new Subscription();
  private webSocketSubscription: Subscription = new Subscription();

  constructor(
    private route: ActivatedRoute,
    private gameService: GameService,
    private websocketService: WebsocketService
  ) {}

  ngOnInit(): void {
    // Get game ID from route
    this.route.paramMap.subscribe(params => {
      this.gameId = params.get('id');
      if (this.gameId) {
        // Get initial game state
        this.gameService.getGameState(this.gameId).subscribe({
          next: (gameState) => {
            this.gameState = gameState;
            this.loading = false;
            
            // Connect to WebSocket for live updates
            this.websocketService.connect(this.gameId as string);
            
            // Subscribe to greeting messages
            this.webSocketSubscription.add(
              this.websocketService.lastGreeting$.subscribe(greeting => {
                if (greeting) {
                  this.greeting = greeting;
                  console.log('Received greeting:', greeting);
                }
              })
            );
            
            // Also subscribe to all WebSocket messages
            this.webSocketSubscription.add(
              this.websocketService.messages$.subscribe(message => {
                console.log('Received message:', message);
                if (message.type === 'game_state') {
                  this.gameState = message.data;
                }
              })
            );
          },
          error: (error) => {
            console.error('Error fetching game:', error);
            this.loading = false;
            this.error = 'Game not found or could not be loaded.';
          }
        });
      }
    });
  }

  ngOnDestroy(): void {
    // Clean up subscriptions
    this.subscription.unsubscribe();
    this.webSocketSubscription.unsubscribe();
    
    // Disconnect WebSocket
    this.websocketService.disconnect();
  }

  // Helper method to format player resources for display
  getResourceEntries(player: Player): { name: string, count: number }[] {
    if (!player.resources) {
      return [];
    }

    return Object.entries(player.resources).map(([resource, count]) => {
      return {
        name: resource.toLowerCase(),
        count: count
      };
    });
  }
} 
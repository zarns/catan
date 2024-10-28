import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { WebSocketService } from '../../services/websocket.service';
import { Subscription } from 'rxjs';

// Import interfaces and types
interface Position {
  x: number;
  y: number;
}

interface Player {
  id: string;
  name: string;
  resources: Record<string, number>;
  development_cards: string[];
  roads: string[];
  settlements: string[];
  cities: string[];
  knights_played: number;
}

interface Board {
  hexes: Array<{
    position: Position;
    terrain: string;
    token: number | null;
    has_robber: boolean;
  }>;
  harbors: Array<{
    position: Position;
    harbor_type: string;
  }>;
  roads: Record<string, string>;
  settlements: Record<string, string>;
}

interface GameState {
  players: Record<string, Player>;
  board: Board;
  current_turn: string;
  dice_value: number | null;
  phase: {
    Setup?: 'PlacingFirstSettlement' | 'PlacingFirstRoad' | 'PlacingSecondSettlement' | 'PlacingSecondRoad';
  };
  robber_position: Position;
}

interface GameJoinedResponse {
  GameJoined: {
    player_id: string;
    game_state: GameState;
  };
}

@Component({
  selector: 'app-game',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="game-container">
      <div class="header">
        <h2>Catan Game</h2>
        <p [class]="isConnected ? 'status-connected' : 'status-disconnected'">
          Status: {{ isConnected ? 'Connected' : 'Disconnected' }}
        </p>
      </div>

      <div class="controls" *ngIf="!gameState">
        <input 
          #playerName
          type="text" 
          placeholder="Enter your name"
          class="input-field"
        >
        <button 
          (click)="joinGame(playerName.value)"
          class="btn-primary"
        >
          Join Game
        </button>
      </div>

      <div class="game-state" *ngIf="gameState">
        <div class="phase-info">
          <h3>Game Phase</h3>
          <p>{{ gameState.phase | json }}</p>
        </div>

        <div class="players-list">
          <h3>Players</h3>
          <div *ngFor="let player of gameState.players | keyvalue" 
               [class.current-turn]="player.key === gameState.current_turn"
               class="player-info">
            <p>{{ player.value.name }}</p>
            <p *ngIf="player.key === playerId">(You)</p>
          </div>
        </div>

        <div class="board-info">
          <h3>Game Board</h3>
          <!-- Board visualization will go here -->
        </div>
      </div>

      <div class="debug-info">
        <h3>Debug Messages:</h3>
        <div class="message-list">
          <div *ngFor="let message of messages" 
               class="message-item">
            {{ message | json }}
          </div>
        </div>
      </div>
    </div>
  `,
  styles: [`
    .game-container {
      @apply container mx-auto p-4;
    }

    .header {
      @apply mb-4;
      
      h2 {
        @apply text-2xl font-bold mb-2;
      }
    }

    .status-connected {
      @apply text-green-600;
    }

    .status-disconnected {
      @apply text-red-600;
    }

    .controls {
      @apply mb-4 flex gap-2;
    }

    .input-field {
      @apply border p-2 rounded;
      
      &:focus {
        @apply outline-none ring-2 ring-blue-500;
      }
    }

    .btn-primary {
      @apply bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600 transition-colors;
      
      &:disabled {
        @apply opacity-50 cursor-not-allowed;
      }
    }

    .messages {
      @apply mt-4;
      
      h3 {
        @apply text-xl font-bold mb-2;
      }
    }

    .message-list {
      @apply space-y-2;
    }

    .message-item {
      @apply p-2 bg-gray-100 rounded;
      
      &:hover {
        @apply bg-gray-200;
      }
    }

    .player-info {
      @apply p-2 border rounded mb-2;
    }

    .current-turn {
      @apply bg-blue-100;
    }
  `]
})
export class GameComponent implements OnInit, OnDestroy {
  messages: any[] = [];
  isConnected = false;
  gameState: GameState | null = null;
  playerId: string | null = null;
  private subscriptions: Subscription[] = [];

  constructor(private wsService: WebSocketService) {}

  ngOnInit() {
    this.wsService.connect();
    
    this.subscriptions.push(
      this.wsService.isConnected().subscribe(
        connected => {
          console.log('Connection status changed:', connected);
          this.isConnected = connected;
        }
      )
    );

    this.subscriptions.push(
      this.wsService.getMessages().subscribe(
        message => {
          if (message) {
            console.log('Received message:', message);
            this.messages.push(message);
            this.handleMessage(message);
          }
        }
      )
    );
  }

  joinGame(playerName: string) {
    if (!playerName.trim()) {
      alert('Please enter a player name');
      return;
    }
    
    if (!this.wsService.getConnectionStatus()) {
      alert('Not connected to server. Please wait...');
      return;
    }

    this.wsService.sendMessage('JoinGame', {
      player_name: playerName.trim()
    });
  }

  private handleMessage(message: any) {
    console.log('Handling message:', message);
    
    if ('GameJoined' in message) {
      const response = message as GameJoinedResponse;
      this.playerId = response.GameJoined.player_id;
      this.gameState = response.GameJoined.game_state;
      this.handleGameState();
    } else if ('Error' in message) {
      console.error('Error from server:', message.Error);
    }
  }

  private handleGameState() {
    if (!this.gameState) return;

    const isMyTurn = this.gameState.current_turn === this.playerId;
    
    if (this.gameState.phase.Setup === 'PlacingFirstSettlement') {
      console.log('Player should place their first settlement');
      // Enable settlement placement UI
    }
  }

  placeSettlement(position: Position) {
    if (!this.gameState?.phase.Setup) return;
    
    this.wsService.sendMessage('PlaceSettlement', {
      position: position
    });
  }

  placeRoad(start: Position, end: Position) {
    if (!this.gameState?.phase.Setup) return;
    
    this.wsService.sendMessage('PlaceRoad', {
      start: start,
      end: end
    });
  }

  ngOnDestroy() {
    this.subscriptions.forEach(sub => sub.unsubscribe());
    this.wsService.disconnect();
  }
}
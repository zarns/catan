import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { WebSocketService } from '../../services/websocket.service';
import { GameMessage, GameMessageType } from '../../models/game-message';
import { Subscription } from 'rxjs';

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

      <div class="controls">
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

      <div class="messages">
        <h3>Game Messages:</h3>
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
  `]
})
export class GameComponent implements OnInit, OnDestroy {
  messages: GameMessage[] = [];
  isConnected = false;
  private subscription: Subscription | null = null;

  constructor(private wsService: WebSocketService) {}

  ngOnInit() {
    this.wsService.connect();
    this.subscription = this.wsService.getMessages().subscribe(message => {
      if (message) {
        this.messages.push(message);
        this.handleMessage(message);
      }
    });
  }

  joinGame(playerName: string) {
    if (!playerName.trim()) {
      alert('Please enter a player name');
      return;
    }
    
    this.wsService.sendMessage(
      GameMessageType.JOIN_GAME,
      { playerName: playerName.trim() }
    );
  }

  private handleMessage(message: GameMessage) {
    switch (message.type) {
      case GameMessageType.GAME_STATE:
        // Handle game state update
        break;
      case GameMessageType.ERROR:
        // Handle error message
        break;
    }
  }

  ngOnDestroy() {
    this.subscription?.unsubscribe();
    this.wsService.disconnect();
  }
}
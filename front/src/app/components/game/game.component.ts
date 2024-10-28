// src/app/components/game/game.component.ts
import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { BoardComponent } from '../board/board.component';
import { WebSocketService } from '../../services/websocket.service';
import { GameState, Position } from '../../models/game-types';

@Component({
  selector: 'app-game',
  standalone: true,
  imports: [CommonModule, BoardComponent],
  template: `
    <div class="min-h-screen p-4 flex flex-col bg-gray-100">
      <!-- Header -->
      <div class="mb-4">
        <h2 class="text-2xl font-bold mb-2">Catan Game</h2>
        <p [class]="isConnected ? 'text-green-600' : 'text-red-600'">
          Status: {{ isConnected ? 'Connected' : 'Disconnected' }}
        </p>
      </div>

      <!-- Join Game Form (show only when no gameState) -->
      <div *ngIf="!gameState" class="flex gap-4 my-4">
        <input 
          #playerName
          type="text" 
          placeholder="Enter your name"
          class="px-4 py-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
        <button 
          (click)="joinGame(playerName.value)"
          class="px-6 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
        >
          Join Game
        </button>
      </div>

      <!-- Game Board (show only when gameState exists) -->
      <div *ngIf="gameState" class="flex-grow">
        <app-board
          [gameState]="gameState"
          (hexClick)="onHexClick($event)">
        </app-board>
      </div>
    </div>
  `
})
export class GameComponent implements OnInit {
  gameState?: GameState;
  isConnected = false;

  constructor(private wsService: WebSocketService) {}

  ngOnInit() {
    this.wsService.connect();
    
    this.wsService.isConnected().subscribe(
      connected => {
        console.log('Connection status:', connected);
        this.isConnected = connected;
      }
    );

    this.wsService.getMessages().subscribe(
      message => {
        if (message) {
          console.log('Received message:', message);
          if ('GameJoined' in message) {
            this.gameState = message.GameJoined.game_state;
          } else if ('GameStateUpdate' in message) {
            this.gameState = message.GameStateUpdate.game_state;
          }
        }
      }
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

    console.log('Joining game with name:', playerName);
    this.wsService.sendMessage('JoinGame', {
      player_name: playerName.trim()
    });
  }

  onHexClick(position: Position) {
    console.log('Hex clicked:', position);
  }
}
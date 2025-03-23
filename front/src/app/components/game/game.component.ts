import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Router } from '@angular/router';
import { BoardComponent } from '../board/board.component';
import { WebSocketService } from '../../services/websocket.service';
import { GameState, Position } from '../../models/game-types';
import { Subscription } from 'rxjs';

@Component({
  selector: 'app-game',
  standalone: true,
  imports: [CommonModule, BoardComponent],
  templateUrl: './game.component.html',
  styleUrls: ['./game.component.scss']
})
export class GameComponent implements OnInit, OnDestroy {
  gameState: GameState | null = null;
  playerId: string | null = null;
  private subscriptions: Subscription[] = [];
  private isProcessingAction = false;

  constructor(
    private wsService: WebSocketService,
    private router: Router
  ) {}

  get isCurrentTurn(): boolean {
    return !!this.gameState && 
           !!this.playerId && 
           this.gameState.currentTurn === this.playerId;
  }

  ngOnInit() {
    document.addEventListener('visibilitychange', () => {
      if (document.hidden) {
        console.log('Tab hidden - disconnecting WebSocket');
        this.wsService.disconnect();
      } else {
        console.log('Tab visible - connecting WebSocket');
        this.connectToGame();
      }
    });

    window.addEventListener('beforeunload', () => {
      this.wsService.disconnect();
    });

    this.connectToGame();
  }

  private connectToGame() {
    this.wsService.connect();

    this.subscriptions.push(
      this.wsService.getMessages().subscribe(message => {
        if (message) {
          this.handleGameMessage(message);
        }
      })
    );

    this.subscriptions.push(
      this.wsService.isConnected().subscribe(connected => {
        if (connected) {
          this.wsService.sendMessage('JoinGame', {
            player_name: 'Player'
          });
        }
      })
    );
  }

  private handleGameMessage(message: any) {
    this.isProcessingAction = false;
    
    if ('GameJoined' in message) {
      this.playerId = message.GameJoined.player_id;
      this.gameState = message.GameJoined.game_state;
    } else if ('GameStateUpdate' in message) {
      this.gameState = message.GameStateUpdate.game_state;
    } else if ('Error' in message) {
      console.error('Game error:', message.Error);
    }
  }

  onSettlementPlaced(position: Position) {
    if (!this.isCurrentTurn || this.isProcessingAction) return;
    
    this.isProcessingAction = true;
    this.wsService.sendMessage('BuildSettlement', { position });

    setTimeout(() => {
      this.isProcessingAction = false;
    }, 5000);
  }

  onRoadPlaced(road: {start: Position, end: Position}) {
    if (!this.isCurrentTurn || this.isProcessingAction) return;
    
    this.isProcessingAction = true;
    this.wsService.sendMessage('BuildRoad', road);

    setTimeout(() => {
      this.isProcessingAction = false;
    }, 5000);
  }

  exitGame() {
    this.wsService.disconnect();
    this.router.navigate(['/home']);
  }

  ngOnDestroy() {
    this.subscriptions.forEach(sub => sub.unsubscribe());
    this.wsService.disconnect();
  }
}
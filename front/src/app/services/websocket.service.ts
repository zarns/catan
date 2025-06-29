import { Injectable } from '@angular/core';
import { Observable, Subject, BehaviorSubject } from 'rxjs';
import { environment } from '../../environments/environment';

export type WsMessageType =
  | 'game_state'
  | 'game_updated'
  | 'error'
  | 'greeting'
  | 'player_action'
  | 'get_game_state'
  | 'bot_thinking'
  | 'action_result'
  | 'create_game'
  | 'game_created';

export interface WsMessage {
  type: WsMessageType;
  data?: any;
  game?: any;
  game_id?: string;
  action?: any;
  message?: string;
  success?: boolean;
  events?: any[];
}

@Injectable({
  providedIn: 'root',
})
export class WebsocketService {
  private socket: WebSocket | null = null;
  private messagesSubject = new Subject<WsMessage>();
  private connectionStatusSubject = new BehaviorSubject<boolean>(false);
  private lastGreeting = new BehaviorSubject<string>('');

  public messages$ = this.messagesSubject.asObservable();
  public connectionStatus$ = this.connectionStatusSubject.asObservable();
  public lastGreeting$ = this.lastGreeting.asObservable();

  constructor() {}

  // Connect to a specific game
  public connect(gameId: string): Observable<boolean> {
    // Close existing connection if any
    this.disconnect();

    const wsUrl = `${environment.wsUrl}/games/${gameId}`;

    this.socket = new WebSocket(wsUrl);

    this.socket.onopen = () => {
      console.log('✅ WebSocket connection established successfully');
      this.connectionStatusSubject.next(true);
    };

    this.socket.onmessage = event => {
      try {
        const message = JSON.parse(event.data) as WsMessage;
        console.log('📨 Received WebSocket message:', message);

        // Handle greeting messages specifically
        if (message.type === 'greeting') {
          this.lastGreeting.next(message.message || message.data);
        }

        this.messagesSubject.next(message);
      } catch (error) {
        console.error('❌ Error parsing WebSocket message:', error);
      }
    };

    this.socket.onclose = event => {
      console.log('🔌 WebSocket connection closed. Code:', event.code, 'Reason:', event.reason);
      this.connectionStatusSubject.next(false);
    };

    this.socket.onerror = error => {
      console.error('🚫 WebSocket error:', error);
      this.connectionStatusSubject.next(false);
    };

    return this.connectionStatus$;
  }

  public disconnect(): void {
    if (this.socket) {
      this.socket.close();
      this.socket = null;
      this.connectionStatusSubject.next(false);
    }
  }

  public sendMessage(message: any) {
    if (this.socket && this.socket.readyState === WebSocket.OPEN) {
      console.debug('📤 Sending WebSocket message:', message.type);
      this.socket.send(JSON.stringify(message));
    } else {
      console.error('Cannot send message, WebSocket is not connected');
    }
  }

  // Helper methods for specific message types
  public sendPlayerAction(gameId: string, action: any): void {
    this.sendMessage({
      type: 'player_action',
      action: action,
    });
  }

  // ✅ REMOVED: sendBotAction() - Frontend should never trigger bot actions
  // Bot actions are handled automatically by the backend

  public requestGameState(gameId: string): void {
    this.sendMessage({
      type: 'get_game_state',
    });
  }

  public createGame(mode: string, numPlayers: number): void {
    this.sendMessage({
      type: 'create_game',
      mode: mode,
      num_players: numPlayers,
    });
  }
}

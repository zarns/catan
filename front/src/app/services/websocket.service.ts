import { Injectable } from '@angular/core';
import { BehaviorSubject, Observable } from 'rxjs';
import { environment } from '../../environments/environment';

// Define types if not already in a separate file
export interface GameMessage {
  type: GameMessageType;
  payload: any;
}

export enum GameMessageType {
  JOIN_GAME = 'JoinGame',
  CREATE_GAME = 'CreateGame',
  GAME_STATE = 'GameState',
  PLAYER_ACTION = 'PlayerAction',
  ERROR = 'Error'
}

@Injectable({
  providedIn: 'root'
})
export class WebSocketService {
  private socket: WebSocket | null = null;
  private messageSubject = new BehaviorSubject<GameMessage | null>(null);
  private readonly wsUrl = environment.wsUrl;

  connect(): void {
    if (this.socket?.readyState !== WebSocket.OPEN) {
      console.log(`Connecting to WebSocket at ${this.wsUrl}`);
      this.socket = new WebSocket(this.wsUrl);
      
      this.socket.onopen = () => {
        console.log(`Connected to ${this.wsUrl}`);
      };

      this.socket.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data);
          console.log('Received message:', message);
          this.messageSubject.next(message);
        } catch (error) {
          console.error('Error parsing message:', error);
        }
      };

      this.socket.onclose = () => {
        console.log('WebSocket disconnected');
        // Optional: Implement reconnection logic
        if (!environment.production) {
          setTimeout(() => this.connect(), 5000);
        }
      };

      this.socket.onerror = (error) => {
        console.error('WebSocket error:', error);
      };
    }
  }

  sendMessage(type: GameMessageType, payload: any): void {
    if (this.socket?.readyState === WebSocket.OPEN) {
      // Format the message to match Rust's expected format
      const message = {
        [type]: payload
      };
      console.log('Sending message:', message);
      this.socket.send(JSON.stringify(message));
    } else {
      console.error('WebSocket is not connected');
    }
  }

  getMessages(): Observable<GameMessage | null> {
    return this.messageSubject.asObservable();
  }

  disconnect(): void {
    if (this.socket) {
      this.socket.close();
      this.socket = null;
    }
  }
}
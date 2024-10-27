import { Injectable } from '@angular/core';
import { BehaviorSubject, Observable } from 'rxjs';
import { GameMessage, GameMessageType } from '../models/game-message';

@Injectable({
  providedIn: 'root'
})
export class WebSocketService {
  private socket: WebSocket | null = null;
  private messageSubject = new BehaviorSubject<GameMessage | null>(null);
  private readonly WS_URL = window.location.hostname === 'localhost' 
    ? 'ws://127.0.0.1:8000/ws'
    : 'wss://catan.shuttleapp.rs/ws';

  connect(): void {
    if (this.socket?.readyState !== WebSocket.OPEN) {
      this.socket = new WebSocket(this.WS_URL);
      
      this.socket.onopen = () => {
        console.log('WebSocket connected');
      };

      this.socket.onmessage = (event) => {
        try {
          const message: GameMessage = JSON.parse(event.data);
          this.messageSubject.next(message);
        } catch (error) {
          console.error('Error parsing message:', error);
        }
      };

      this.socket.onclose = () => {
        console.log('WebSocket disconnected');
        // Attempt to reconnect after 5 seconds
        setTimeout(() => this.connect(), 5000);
      };

      this.socket.onerror = (error) => {
        console.error('WebSocket error:', error);
      };
    }
  }

  sendMessage(type: GameMessageType, payload: any): void {
    if (this.socket?.readyState === WebSocket.OPEN) {
      const message: GameMessage = { type, payload };
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
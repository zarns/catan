import { Injectable } from '@angular/core';
import { Observable, Subject, BehaviorSubject } from 'rxjs';
import { environment } from '../../environments/environment';

export type WsMessageType = 'game_state' | 'error' | 'greeting' | 'player_action' | 'bot_action' | 'action_result';

export interface WsMessage {
  type: WsMessageType;
  data: any;
}

@Injectable({
  providedIn: 'root'
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
    console.log(`Connecting to WebSocket at ${wsUrl}`);
    
    this.socket = new WebSocket(wsUrl);

    this.socket.onopen = () => {
      console.log('WebSocket connection established');
      this.connectionStatusSubject.next(true);
    };

    this.socket.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data) as WsMessage;
        console.log('Received WebSocket message:', message);
        
        // Handle greeting messages specifically
        if (message.type === 'greeting') {
          this.lastGreeting.next(message.data);
        }
        
        this.messagesSubject.next(message);
      } catch (error) {
        console.error('Error parsing WebSocket message:', error);
      }
    };

    this.socket.onclose = () => {
      console.log('WebSocket connection closed');
      this.connectionStatusSubject.next(false);
    };

    this.socket.onerror = (error) => {
      console.error('WebSocket error:', error);
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
      this.socket.send(JSON.stringify(message));
    } else {
      console.error('Cannot send message, WebSocket is not connected');
    }
  }
} 
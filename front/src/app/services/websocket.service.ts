import { Injectable } from '@angular/core';
import { BehaviorSubject, Observable } from 'rxjs';
import { environment } from '../../environments/environment';

@Injectable({
  providedIn: 'root'
})
export class WebSocketService {
  private socket: WebSocket | null = null;
  private messageSubject = new BehaviorSubject<any>(null);
  private connectionStatusSubject = new BehaviorSubject<boolean>(false);
  private readonly wsUrl = environment.wsUrl;

  connect(): void {
    if (this.socket?.readyState !== WebSocket.OPEN) {
      console.log(`Connecting to WebSocket at ${this.wsUrl}`);
      this.socket = new WebSocket(this.wsUrl);
      
      this.socket.onopen = () => {
        console.log(`Connected to ${this.wsUrl}`);
        this.connectionStatusSubject.next(true);
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
        this.connectionStatusSubject.next(false);
        // Optional: Implement reconnection logic
        if (!environment.production) {
          setTimeout(() => this.connect(), 5000);
        }
      };

      this.socket.onerror = (error) => {
        console.error('WebSocket error:', error);
        this.connectionStatusSubject.next(false);
      };
    }
  }

  isConnected(): Observable<boolean> {
    return this.connectionStatusSubject.asObservable();
  }

  getConnectionStatus(): boolean {
    return this.socket?.readyState === WebSocket.OPEN;
  }

  sendMessage(type: string, payload: any): void {
    if (this.socket?.readyState === WebSocket.OPEN) {
      const message = {
        [type]: payload
      };
      console.log('Sending message:', message);
      this.socket.send(JSON.stringify(message));
    } else {
      console.error('WebSocket is not connected');
    }
  }

  getMessages(): Observable<any> {
    return this.messageSubject.asObservable();
  }

  disconnect(): void {
    if (this.socket) {
      this.socket.close();
      this.socket = null;
      this.connectionStatusSubject.next(false);
    }
  }
}
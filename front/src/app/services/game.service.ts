import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, of } from 'rxjs';
import { catchError } from 'rxjs/operators';

export enum GameMode {
  HUMAN_VS_CATANATRON = 'HUMAN_VS_CATANATRON',
  RANDOM_BOTS = 'RANDOM_BOTS',
  CATANATRON_BOTS = 'CATANATRON_BOTS'
}

export interface GameConfig {
  mode: GameMode;
  numPlayers: number;
}

export interface GameState {
  id: string;
  status: 'waiting' | 'in_progress' | 'finished';
  // Additional game state properties will be added here
}

@Injectable({
  providedIn: 'root'
})
export class GameService {
  private apiUrl = 'http://localhost:8000'; // Default Rust backend URL

  constructor(private http: HttpClient) {}

  /**
   * Creates a new game with the specified configuration
   */
  createGame(config: GameConfig): Observable<GameState> {
    return this.http.post<GameState>(`${this.apiUrl}/games`, config)
      .pipe(
        catchError(this.handleError<GameState>('createGame'))
      );
  }

  /**
   * Gets the current state of a game
   */
  getGameState(gameId: string): Observable<GameState> {
    return this.http.get<GameState>(`${this.apiUrl}/games/${gameId}`)
      .pipe(
        catchError(this.handleError<GameState>('getGameState'))
      );
  }

  /**
   * Error handler for HTTP requests
   */
  private handleError<T>(operation = 'operation', result?: T) {
    return (error: any): Observable<T> => {
      console.error(`${operation} failed: ${error.message}`);
      
      // Let the app keep running by returning an empty result
      return of(result as T);
    };
  }
} 
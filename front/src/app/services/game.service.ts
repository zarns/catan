import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, tap, catchError, throwError } from 'rxjs';
import { environment } from '../../environments/environment';
import { WebsocketService } from './websocket.service';

// Game-related types
export interface Resource {
  brick?: number;
  lumber?: number;
  wool?: number;
  grain?: number;
  ore?: number;
}

export interface Player {
  id: string;
  name: string;
  color: string;
  resources: Resource;
  dev_cards: string[];
  knights_played: number;
  victory_points: number;
  longest_road: boolean;
  largest_army: boolean;
}

export interface Game {
  id: string;
  players: Player[];
  current_player_index: number;
  dice_rolled: boolean;
  winner: string | null;
  turns: number;
}

export interface GameState {
  id: string;
  status: 'waiting' | 'in_progress' | 'finished';
  game?: Game;
}

export interface GameConfig {
  mode: 'HUMAN_VS_CATANATRON' | 'RANDOM_BOTS' | 'CATANATRON_BOTS';
  num_players: number;
}

@Injectable({
  providedIn: 'root'
})
export class GameService {
  private apiUrl = environment.apiUrl;

  constructor(
    private http: HttpClient,
    private websocketService: WebsocketService
  ) {}

  createGame(config: GameConfig): Observable<GameState> {
    return this.http.post<GameState>(`${this.apiUrl}/games`, config)
      .pipe(
        tap(gameState => console.log('Game created:', gameState)),
        catchError(error => {
          console.error('Error creating game:', error);
          return throwError(() => new Error('Failed to create game. Please try again.'));
        })
      );
  }

  getGameState(gameId: string): Observable<GameState> {
    return this.http.get<GameState>(`${this.apiUrl}/games/${gameId}`)
      .pipe(
        tap(gameState => console.log('Game state retrieved:', gameState)),
        catchError(error => {
          console.error('Error retrieving game state:', error);
          return throwError(() => new Error('Failed to load game state. Please try again.'));
        })
      );
  }
} 
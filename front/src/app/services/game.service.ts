import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, BehaviorSubject, Subject, tap, catchError, throwError, map } from 'rxjs';
import { environment } from '../../environments/environment';
import { WebsocketService, WsMessage } from './websocket.service';

// Game state interfaces matching backend structure
export interface Coordinate {
  x: number;
  y: number;
  z: number;
}

export interface Tile {
  resource: string;
  number?: number;
}

export interface TilePosition {
  coordinate: Coordinate;
  tile: Tile;
}

export interface Node {
  id: string;
  building?: string; // 'settlement' or 'city'
  color?: string;
}

export interface Edge {
  id: string;
  color?: string;
}

export interface GameBoard {
  tiles: TilePosition[];
  ports: { coordinate: Coordinate, port: { resource: string | null, ratio: number, direction: string } }[];
  nodes: { [nodeId: string]: Node };
  edges: { [edgeId: string]: Edge };
  robber_coordinate: Coordinate;
}

export interface ResourceMap {
  [key: string]: number;
}

export interface DevelopmentCard {
  type: string;
}

export interface Player {
  id: string;
  name: string;
  color: string;
  resources: ResourceMap;
  dev_cards: DevelopmentCard[];
  knights_played: number;
  victory_points: number;
  longest_road: boolean;
  largest_army: boolean;
  settlements_left: number;
  cities_left: number;
  roads_left: number;
  development_cards?: DevelopmentCard[];
  achievements?: string[];
}

export interface Game {
  id: string;
  players: Player[];
  board: GameBoard;
  current_player_index: number;
  dice_rolled: boolean;
  winner?: string;
  turns: number;
  current_dice_roll?: [number, number];
}

export interface GameState {
  id: string;
  status: 'waiting' | 'in_progress' | 'finished';
  game?: Game;
  current_color?: string;
  current_prompt?: 'PLAY_TURN' | 'DISCARD' | 'MOVE_ROBBER';
  bot_colors: string[];
  winning_color?: string;
}

export interface GameConfig {
  mode: 'HUMAN_VS_CATANATRON' | 'RANDOM_BOTS' | 'CATANATRON_BOTS';
  num_players: number;
}

// Game state actions similar to React UI
export enum GameAction {
  SET_GAME_STATE = 'SET_GAME_STATE',
  TOGGLE_BUILDING_ROAD = 'TOGGLE_BUILDING_ROAD',
  SET_IS_BUILDING_SETTLEMENT = 'SET_IS_BUILDING_SETTLEMENT',
  SET_IS_BUILDING_CITY = 'SET_IS_BUILDING_CITY',
  SET_IS_PLAYING_MONOPOLY = 'SET_IS_PLAYING_MONOPOLY',
  CANCEL_MONOPOLY = 'CANCEL_MONOPOLY',
  SET_IS_PLAYING_YEAR_OF_PLENTY = 'SET_IS_PLAYING_YEAR_OF_PLENTY',
  CANCEL_YEAR_OF_PLENTY = 'CANCEL_YEAR_OF_PLENTY',
  PLAY_ROAD_BUILDING = 'PLAY_ROAD_BUILDING',
  SET_IS_MOVING_ROBBER = 'SET_IS_MOVING_ROBBER'
}

// State management
interface GameUIState {
  gameState: GameState | null;
  isBuildingRoad: boolean;
  isBuildingSettlement: boolean;
  isBuildingCity: boolean;
  isPlayingMonopoly: boolean;
  isPlayingYearOfPlenty: boolean;
  isMovingRobber: boolean;
}

@Injectable({
  providedIn: 'root'
})
export class GameService {
  private apiUrl = environment.apiUrl;
  
  // Game state with UI state similar to React Redux store
  private gameUIState = new BehaviorSubject<GameUIState>({
    gameState: null,
    isBuildingRoad: false,
    isBuildingSettlement: false,
    isBuildingCity: false,
    isPlayingMonopoly: false,
    isPlayingYearOfPlenty: false,
    isMovingRobber: false
  });

  // Expose as observable
  gameUIState$ = this.gameUIState.asObservable();
  
  constructor(
    private http: HttpClient,
    private websocketService: WebsocketService
  ) {
    // Listen for WebSocket messages to update game state
    this.websocketService.messages$.subscribe((message: WsMessage) => {
      console.log('🎮 GameService processing WebSocket message:', message.type, message);
      
      if (message.type === 'game_state' || message.type === 'game_updated') {
        // WebSocket sends {type: 'game_state', game: Game}, so message.game contains the Game object
        const game = message.game;
        console.log('🎲 Extracting game from message:', game);
        
        if (game) {
          const gameState: GameState = {
            id: game.id,
            status: 'in_progress',
            game: game,
            bot_colors: []
          };
          
          console.log('🔄 Dispatching SET_GAME_STATE with:', gameState);
          this.dispatch({
            type: GameAction.SET_GAME_STATE,
            payload: gameState
          });
          
          console.log('✅ Game state updated via WebSocket');
        } else {
          console.warn('⚠️ No game object found in WebSocket message');
        }
      } else if (message.type === 'bot_thinking') {
        console.log('🤖 Bot is thinking:', message);
        // Could add bot thinking state management here if needed
      }
    });
  }

  // Dispatch actions similar to Redux
  dispatch(action: { type: GameAction, payload?: any }) {
    const currentState = this.gameUIState.getValue();
    let newState: GameUIState = { ...currentState };

    switch (action.type) {
      case GameAction.SET_GAME_STATE:
        newState.gameState = action.payload;
        break;
      case GameAction.TOGGLE_BUILDING_ROAD:
        newState.isBuildingRoad = !currentState.isBuildingRoad;
        // Reset other building states
        newState.isBuildingSettlement = false;
        newState.isBuildingCity = false;
        break;
      case GameAction.SET_IS_BUILDING_SETTLEMENT:
        newState.isBuildingSettlement = action.payload;
        // Reset other building states
        newState.isBuildingRoad = false;
        newState.isBuildingCity = false;
        break;
      case GameAction.SET_IS_BUILDING_CITY:
        newState.isBuildingCity = action.payload;
        // Reset other building states
        newState.isBuildingRoad = false;
        newState.isBuildingSettlement = false;
        break;
      case GameAction.SET_IS_PLAYING_MONOPOLY:
        newState.isPlayingMonopoly = action.payload;
        break;
      case GameAction.CANCEL_MONOPOLY:
        newState.isPlayingMonopoly = false;
        break;
      case GameAction.SET_IS_PLAYING_YEAR_OF_PLENTY:
        newState.isPlayingYearOfPlenty = action.payload;
        break;
      case GameAction.CANCEL_YEAR_OF_PLENTY:
        newState.isPlayingYearOfPlenty = false;
        break;
      case GameAction.SET_IS_MOVING_ROBBER:
        newState.isMovingRobber = action.payload;
        break;
    }

    this.gameUIState.next(newState);
  }

  // API methods
  createGame(config: GameConfig): Observable<GameState> {
    return this.http.post<Game>(`${this.apiUrl}/games`, config).pipe(
      tap(game => {
        console.log('Game created:', game);
        // HTTP API returns Game object directly, wrap it as GameState
        const gameState: GameState = {
          id: game.id,
          status: 'in_progress',
          game: game,
          bot_colors: []
        };
        this.dispatch({
          type: GameAction.SET_GAME_STATE,
          payload: gameState
        });
      }),
      map(game => ({
        id: game.id,
        status: 'in_progress' as const,
        game: game,
        bot_colors: []
      })),
      catchError(error => {
        console.error('Error creating game:', error);
        return throwError(() => new Error('Failed to create game'));
      })
    );
  }

  getGameState(gameId: string): Observable<GameState> {
    return this.http.get<Game>(`${this.apiUrl}/games/${gameId}`).pipe(
      tap(game => {
        console.log('Game state retrieved:', game);
        // HTTP API returns Game object directly, wrap it as GameState
        const gameState: GameState = {
          id: game.id,
          status: 'in_progress',
          game: game,
          bot_colors: []
        };
        this.dispatch({
          type: GameAction.SET_GAME_STATE,
          payload: gameState
        });
      }),
      map(game => ({
        id: game.id,
        status: 'in_progress' as const,
        game: game,
        bot_colors: []
      })),
      catchError(error => {
        console.error('Error retrieving game state:', error);
        return throwError(() => new Error('Failed to retrieve game state'));
      })
    );
  }

  // Build a road at an edge
  buildRoad(gameId: string, edgeId: string): Observable<GameState> {
    return this.http.post<GameState>(
      `${this.apiUrl}/games/${gameId}/actions`,
      { action: 'build_road', edge_id: edgeId }
    ).pipe(
      tap(gameState => {
        this.dispatch({
          type: GameAction.SET_GAME_STATE,
          payload: gameState
        });
        this.dispatch({
          type: GameAction.TOGGLE_BUILDING_ROAD,
        });
      }),
      catchError(error => {
        console.error('Error building road:', error);
        return throwError(() => new Error('Failed to build road'));
      })
    );
  }

  // Build a settlement at a node
  buildSettlement(gameId: string, nodeId: string): Observable<GameState> {
    return this.http.post<GameState>(
      `${this.apiUrl}/games/${gameId}/actions`,
      { action: 'build_settlement', node_id: nodeId }
    ).pipe(
      tap(gameState => {
        this.dispatch({
          type: GameAction.SET_GAME_STATE,
          payload: gameState
        });
        this.dispatch({
          type: GameAction.SET_IS_BUILDING_SETTLEMENT,
          payload: false
        });
      }),
      catchError(error => {
        console.error('Error building settlement:', error);
        return throwError(() => new Error('Failed to build settlement'));
      })
    );
  }

  // Build a city at a node
  buildCity(gameId: string, nodeId: string): Observable<GameState> {
    return this.http.post<GameState>(
      `${this.apiUrl}/games/${gameId}/actions`,
      { action: 'build_city', node_id: nodeId }
    ).pipe(
      tap(gameState => {
        this.dispatch({
          type: GameAction.SET_GAME_STATE,
          payload: gameState
        });
        this.dispatch({
          type: GameAction.SET_IS_BUILDING_CITY,
          payload: false
        });
      }),
      catchError(error => {
        console.error('Error building city:', error);
        return throwError(() => new Error('Failed to build city'));
      })
    );
  }

  // Roll dice
  rollDice(gameId: string): Observable<GameState> {
    return this.http.post<GameState>(
      `${this.apiUrl}/games/${gameId}/actions`,
      { action: 'roll_dice' }
    ).pipe(
      tap(gameState => {
        this.dispatch({
          type: GameAction.SET_GAME_STATE,
          payload: gameState
        });
      }),
      catchError(error => {
        console.error('Error rolling dice:', error);
        return throwError(() => new Error('Failed to roll dice'));
      })
    );
  }

  // End turn
  endTurn(gameId: string): Observable<GameState> {
    return this.http.post<GameState>(
      `${this.apiUrl}/games/${gameId}/actions`,
      { action: 'end_turn' }
    ).pipe(
      tap(gameState => {
        this.dispatch({
          type: GameAction.SET_GAME_STATE,
          payload: gameState
        });
      }),
      catchError(error => {
        console.error('Error ending turn:', error);
        return throwError(() => new Error('Failed to end turn'));
      })
    );
  }

  // Move the robber
  moveRobber(gameId: string, coordinate: Coordinate): Observable<GameState> {
    return this.http.post<GameState>(
      `${this.apiUrl}/games/${gameId}/actions`,
      { action: 'move_robber', coordinate }
    ).pipe(
      tap(gameState => {
        this.dispatch({
          type: GameAction.SET_GAME_STATE,
          payload: gameState
        });
        this.dispatch({
          type: GameAction.SET_IS_MOVING_ROBBER,
          payload: false
        });
      }),
      catchError(error => {
        console.error('Error moving robber:', error);
        return throwError(() => new Error('Failed to move robber'));
      })
    );
  }

  // Play road building development card
  playRoadBuilding(gameId: string): Observable<GameState> {
    return this.http.post<GameState>(
      `${this.apiUrl}/games/${gameId}/actions`,
      { action: 'play_road_building' }
    ).pipe(
      tap(gameState => {
        this.dispatch({
          type: GameAction.SET_GAME_STATE,
          payload: gameState
        });
      }),
      catchError(error => {
        console.error('Error playing road building card:', error);
        return throwError(() => new Error('Failed to play road building card'));
      })
    );
  }

  // Play knight development card
  playKnightCard(gameId: string): Observable<GameState> {
    return this.http.post<GameState>(
      `${this.apiUrl}/games/${gameId}/actions`,
      { action: 'play_knight_card' }
    ).pipe(
      tap(gameState => {
        this.dispatch({
          type: GameAction.SET_GAME_STATE,
          payload: gameState
        });
      }),
      catchError(error => {
        console.error('Error playing knight card:', error);
        return throwError(() => new Error('Failed to play knight card'));
      })
    );
  }

  // Buy development card
  buyDevelopmentCard(gameId: string): Observable<GameState> {
    return this.http.post<GameState>(
      `${this.apiUrl}/games/${gameId}/actions`,
      { action: 'buy_development_card' }
    ).pipe(
      tap(gameState => {
        this.dispatch({
          type: GameAction.SET_GAME_STATE,
          payload: gameState
        });
      }),
      catchError(error => {
        console.error('Error buying development card:', error);
        return throwError(() => new Error('Failed to buy development card'));
      })
    );
  }

  // Execute a trade
  executeTrade(gameId: string, trade: any): Observable<GameState> {
    return this.http.post<GameState>(
      `${this.apiUrl}/games/${gameId}/actions`,
      { action: 'maritime_trade', ...trade }
    ).pipe(
      tap(gameState => {
        this.dispatch({
          type: GameAction.SET_GAME_STATE,
          payload: gameState
        });
      }),
      catchError(error => {
        console.error('Error executing trade:', error);
        return throwError(() => new Error('Failed to execute trade'));
      })
    );
  }

  // Core action method to match React UI's postAction function
  // This sends actions via WebSocket instead of HTTP
  postAction(gameId: string, action?: any): Observable<GameState> {
    const subject = new Subject<GameState>();
    
    if (!action) {
      // This is a bot action in the React implementation
      this.websocketService.sendMessage({
        type: 'bot_action',
        game_id: gameId
      });
    } else {
      // Regular player action
      this.websocketService.sendMessage({
        type: 'player_action',
        game_id: gameId,
        action: action
      });
    }
    
    // Set up one-time listener for the response
    const subscription = this.websocketService.messages$.subscribe((message: WsMessage) => {
      if (message.type === 'game_state') {
        // Update internal state
        this.dispatch({
          type: GameAction.SET_GAME_STATE,
          payload: message.data
        });
        
        // Emit the response
        subject.next(message.data);
        subject.complete();
        
        // Clean up subscription
        subscription.unsubscribe();
      } else if (message.type === 'error') {
        subject.error(new Error(message.data.message || 'Action failed'));
        subscription.unsubscribe();
      }
    });
    
    return subject.asObservable();
  }
  
  // Method to directly update game state (useful for watch mode)
  updateGameState(gameState: GameState): void {
    this.dispatch({
      type: GameAction.SET_GAME_STATE,
      payload: gameState
    });
  }
  
  // Simplified helper methods that match the React implementation pattern
  
  // Build a road using the generic postAction format
  buildRoadAction(gameId: string, edgeId: string): Observable<GameState> {
    return this.postAction(gameId, ['BUILD_ROAD', edgeId]);
  }
  
  // Build a settlement using the generic postAction format
  buildSettlementAction(gameId: string, nodeId: string): Observable<GameState> {
    return this.postAction(gameId, ['BUILD_SETTLEMENT', nodeId]);
  }
  
  // Build a city using the generic postAction format
  buildCityAction(gameId: string, nodeId: string): Observable<GameState> {
    return this.postAction(gameId, ['BUILD_CITY', nodeId]);
  }
  
  // Roll dice using the generic postAction format
  rollDiceAction(gameId: string): Observable<GameState> {
    return this.postAction(gameId, ['ROLL']);
  }
  
  // End turn using the generic postAction format
  endTurnAction(gameId: string): Observable<GameState> {
    return this.postAction(gameId, ['END_TURN']);
  }
  
  // Move robber using the generic postAction format
  moveRobberAction(gameId: string, coordinate: Coordinate, targetColor?: string): Observable<GameState> {
    if (targetColor) {
      return this.postAction(gameId, ['MOVE_ROBBER', coordinate, targetColor]);
    } else {
      return this.postAction(gameId, ['MOVE_ROBBER', coordinate]);
    }
  }
  
  // Play development cards using the generic postAction format
  playMonopolyAction(gameId: string, resource: string): Observable<GameState> {
    return this.postAction(gameId, ['PLAY_MONOPOLY', resource]);
  }
  
  playYearOfPlentyAction(gameId: string, resources: string[]): Observable<GameState> {
    return this.postAction(gameId, ['PLAY_YEAR_OF_PLENTY', ...resources]);
  }
  
  playRoadBuildingAction(gameId: string): Observable<GameState> {
    return this.postAction(gameId, ['PLAY_ROAD_BUILDING']);
  }
  
  playKnightAction(gameId: string): Observable<GameState> {
    return this.postAction(gameId, ['PLAY_KNIGHT']);
  }
  
  // Buying development card
  buyDevelopmentCardAction(gameId: string): Observable<GameState> {
    return this.postAction(gameId, ['BUY_DEVELOPMENT_CARD']);
  }
  
  // Trading
  tradeWithBankAction(gameId: string, give: string, receive: string): Observable<GameState> {
    return this.postAction(gameId, ['TRADE_WITH_BANK', give, receive]);
  }
} 
import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { BehaviorSubject, Observable, throwError } from 'rxjs';
import { map, tap, catchError } from 'rxjs/operators';
import { environment } from '../../environments/environment';
import { WebsocketService } from './websocket.service';

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
  ports: {
    coordinate: Coordinate;
    port: { resource: string | null; ratio: number; direction: string };
  }[];
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

// Backend sends Rust PlayerAction enum - can be strings for unit variants or objects for data variants
export type PlayableAction = 
  // Unit variants become strings
  | 'Roll'
  | 'EndTurn' 
  | 'BuyDevelopmentCard'
  | 'PlayKnight'
  | 'PlayRoadBuilding'
  // Variants with data become objects
  | { BuildRoad: { edge_id: [number, number] } }
  | { BuildSettlement: { node_id: number } }
  | { BuildCity: { node_id: number } }
  | { PlayYearOfPlenty: { resources: [string, string | null] } }
  | { PlayMonopoly: { resource: string } }
  | { MaritimeTrade: { give: string; take: string; ratio: number } }
  | { OfferTrade: { give: string[]; take: string[] } }
  | { AcceptTrade: { trade_id: string } }
  | { RejectTrade: { trade_id: string } }
  | { MoveRobber: { coordinate: [number, number, number]; victim?: string } }
  | { Discard: { resources: string[] } };

export interface Game {
  id: string;
  players: Player[];
  game_state: string;
  board: GameBoard;
  current_player_index: number;
  dice_rolled: boolean;
  turns: number;
  current_dice_roll?: [number, number];
  actions: any[]; // Game log actions
  // Array of Rust enum objects as sent by backend
  current_playable_actions: PlayableAction[];
  is_initial_build_phase: boolean;
  current_color?: string;
  current_prompt?: string;
  bot_colors: string[];
}

export interface GameState {
  id: string;
  status: 'waiting' | 'in_progress' | 'finished';
  game: Game;
  // Array of Rust enum objects as sent by backend
  current_playable_actions: PlayableAction[];
  current_color?: string;
  current_prompt?: string;
  bot_colors: string[];
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
  SET_IS_MOVING_ROBBER = 'SET_IS_MOVING_ROBBER',
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
  providedIn: 'root',
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
    isMovingRobber: false,
  });

  // Expose as observable
  gameUIState$ = this.gameUIState.asObservable();

  constructor(
    private http: HttpClient,
    private websocketService: WebsocketService
  ) {
    // Listen for WebSocket messages to update game state
    this.websocketService.messages$.subscribe((message: any) => {
      // Changed WsMessage to any as WsMessage is removed
      console.debug('🎮 GameService processing WebSocket message:', message.type);

      if (message.type === 'game_state' || message.type === 'game_updated') {
        // WebSocket sends {type: 'game_state', game: Game}, so message.game contains the Game object
        const game = message.game;
        console.log('🎲 Extracting game from message:', game);

        if (game) {
          const gameState: GameState = {
            id: game.id,
            status: 'in_progress',
            game: game,
            current_playable_actions: game.current_playable_actions || [],
            current_color: game.current_color,
            current_prompt: game.current_prompt,
            bot_colors: game.bot_colors || [],
          };

          console.log('🔄 Dispatching SET_GAME_STATE with:', gameState);
          this.dispatch({
            type: GameAction.SET_GAME_STATE,
            payload: gameState,
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
  dispatch(action: { type: GameAction; payload?: any }) {
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
    console.log('🌐 GameService: Creating game with config:', config);
    return this.http.post<Game>(`${this.apiUrl}/games`, config).pipe(
      tap(game => {
        console.log('🌐 GameService: Game created successfully:', game);
        console.log(
          '🌐 GameService: Game has current_playable_actions:',
          game.current_playable_actions?.length || 0,
          'actions'
        );
        console.log('🌐 GameService: Game bot_colors:', game.bot_colors);
        console.log('🌐 GameService: Game current_color:', game.current_color);
        console.log('🌐 GameService: Game is_initial_build_phase:', game.is_initial_build_phase);

        // HTTP API returns Game object directly, wrap it as GameState
        const gameState: GameState = {
          id: game.id,
          status: 'in_progress',
          game: game,
          current_playable_actions: game.current_playable_actions || [],
          current_color: game.current_color,
          current_prompt: game.current_prompt,
          bot_colors: game.bot_colors || [],
        };
        console.log('🌐 GameService: Dispatching SET_GAME_STATE with:', gameState);
        this.dispatch({
          type: GameAction.SET_GAME_STATE,
          payload: gameState,
        });
      }),
      map(game => ({
        id: game.id,
        status: 'in_progress' as const,
        game: game,
        current_playable_actions: game.current_playable_actions,
        current_color: game.current_color,
        current_prompt: game.current_prompt,
        bot_colors: game.bot_colors || [],
      })),
      catchError(error => {
        console.error('❌ GameService: Error creating game:', error);
        return throwError(() => new Error('Failed to create game'));
      })
    );
  }

  // ✅ REMOVED: getGameState() HTTP method
  // Game state is now fetched via WebSocket using websocketService.requestGameState()

  // Build a road at an edge
  // ✅ REMOVED: Legacy HTTP methods - buildRoad, buildSettlement, buildCity, rollDice, endTurn
  // All actions now use WebSocket via postAction() and the *Action() helper methods below

  // ✅ REMOVED: Legacy HTTP methods - moveRobber, playRoadBuilding, playKnightCard, buyDevelopmentCard, executeTrade
  // All actions now use WebSocket via postAction() and the *Action() helper methods below

  // Core action method - sends actions via WebSocket using enum format
  postAction(gameId: string, action?: any): Observable<GameState> {
    return new Observable(observer => {
      console.debug('🎮 GameService.postAction called with:', {
        gameId,
        action_type: action ? Object.keys(action)[0] : 'BOT_ACTION',
      });

      if (!action) {
        // ✅ REMOVED: Bot action requests - bots should act automatically
        observer.error(new Error('Manual bot actions not supported - bots act automatically'));
        return;
      } else {
        // Regular player action in enum format
        console.debug('👤 Sending player action:', Object.keys(action)[0]);
        this.websocketService.sendPlayerAction(gameId, action);
      }

      // Set up one-time listener for the response
      const subscription = this.websocketService.messages$.subscribe((message: any) => {
        console.debug('📨 GameService received WebSocket message:', message.type);

        if (message.type === 'game_state' || message.type === 'game_updated') {
          // Extract game from message
          const game = message.game;
          if (game) {
            console.debug('🎲 Converting game to GameState');

            const gameState: GameState = {
              id: game.id,
              status: 'in_progress',
              game: game,
              current_playable_actions: game.current_playable_actions || [],
              current_color: game.current_color,
              current_prompt: game.current_prompt,
              bot_colors: game.bot_colors || [],
            };

            // Update internal state
            this.dispatch({
              type: GameAction.SET_GAME_STATE,
              payload: gameState,
            });

            // Emit the response
            observer.next(gameState);
            observer.complete();

            // Clean up subscription
            subscription.unsubscribe();
          }
        } else if (message.type === 'action_result') {
          // Action processed successfully, wait for game state update
          console.debug('✅ Action result received');
        } else if (message.type === 'error') {
          console.error('❌ Error from WebSocket:', message.message);
          observer.error(new Error(message.message || 'Action failed'));
          subscription.unsubscribe();
        }
      });

      // Set a timeout to avoid hanging forever
      setTimeout(() => {
        subscription.unsubscribe();
        observer.error(new Error('WebSocket response timeout'));
      }, 10000); // 10 second timeout
    });
  }

  // Method to directly update game state (useful for watch mode)
  updateGameState(gameState: GameState): void {
    this.dispatch({
      type: GameAction.SET_GAME_STATE,
      payload: gameState,
    });
  }

  // Simplified helper methods using enum format

  // Build a road using enum format
  buildRoadAction(gameId: string, edgeId: string): Observable<GameState> {
    return this.postAction(gameId, { BuildRoad: { edge_id: edgeId } });
  }

  // Build a settlement using enum format
  buildSettlementAction(gameId: string, nodeId: string): Observable<GameState> {
    return this.postAction(gameId, { BuildSettlement: { node_id: nodeId } });
  }

  // Build a city using enum format
  buildCityAction(gameId: string, nodeId: string): Observable<GameState> {
    return this.postAction(gameId, { BuildCity: { node_id: nodeId } });
  }

  // Roll dice using enum format
  rollDiceAction(gameId: string): Observable<GameState> {
    return this.postAction(gameId, { Roll: {} });
  }

  // End turn using enum format
  endTurnAction(gameId: string): Observable<GameState> {
    return this.postAction(gameId, { EndTurn: {} });
  }

  // Move robber using enum format
  moveRobberAction(
    gameId: string,
    coordinate: Coordinate,
    targetColor?: string
  ): Observable<GameState> {
    const coordinateArray = [coordinate.x, coordinate.y, coordinate.z];
    if (targetColor) {
      return this.postAction(gameId, {
        MoveRobber: { coordinate: coordinateArray, victim: targetColor },
      });
    } else {
      return this.postAction(gameId, { MoveRobber: { coordinate: coordinateArray, victim: null } });
    }
  }

  // Play development cards using enum format
  playMonopolyAction(gameId: string, resource: string): Observable<GameState> {
    return this.postAction(gameId, { PlayMonopoly: { resource } });
  }

  playYearOfPlentyAction(gameId: string, resources: string[]): Observable<GameState> {
    return this.postAction(gameId, { PlayYearOfPlenty: { resources } });
  }

  playRoadBuildingAction(gameId: string): Observable<GameState> {
    return this.postAction(gameId, { PlayRoadBuilding: {} });
  }

  playKnightAction(gameId: string): Observable<GameState> {
    return this.postAction(gameId, { PlayKnight: {} });
  }

  // Buying development card
  buyDevelopmentCardAction(gameId: string): Observable<GameState> {
    return this.postAction(gameId, { BuyDevelopmentCard: {} });
  }

  // Trading
  tradeWithBankAction(gameId: string, give: string, receive: string): Observable<GameState> {
    return this.postAction(gameId, { MaritimeTrade: { give, take: receive, ratio: 4 } });
  }
}

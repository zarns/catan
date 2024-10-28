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

export interface JoinGamePayload {
  player_name: string;  // Note: using snake_case to match Rust
}

export interface GameStatePayload {
  players: string[];
  // Add other game state properties from your Rust backend
}

export interface ErrorPayload {
  message: string;
}
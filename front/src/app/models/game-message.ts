export interface GameMessage {
  type: string;
  payload: any;  // We can make this more specific later
}

// You might also want to define specific message types
export enum GameMessageType {
  JOIN_GAME = 'JoinGame',
  CREATE_GAME = 'CreateGame',
  GAME_STATE = 'GameState',
  PLAYER_ACTION = 'PlayerAction',
  ERROR = 'Error'
}

// Specific payload types
export interface JoinGamePayload {
  playerName: string;
}

export interface GameStatePayload {
  players: string[];
  // Add other game state properties as needed
}
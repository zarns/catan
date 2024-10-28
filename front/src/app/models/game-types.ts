// src/app/models/game-types.ts

export interface Position {
  x: number;
  y: number;
}

export interface Hex {
  position: Position;
  terrain: TerrainType;
  token?: number;
  hasRobber: boolean;
}

export interface Harbor {
  position: Position;
  harborType: HarborType;
}

export enum TerrainType {
  Hills = 'HILLS',      // Brick
  Forest = 'FOREST',    // Lumber
  Mountains = 'MOUNTAINS', // Ore
  Fields = 'FIELDS',    // Grain
  Pasture = 'PASTURE',  // Wool
  Desert = 'DESERT'
}

export enum HarborType {
  Generic = 'GENERIC',  // 3:1 trade
  Brick = 'BRICK',      // 2:1 trade
  Lumber = 'LUMBER',
  Ore = 'ORE',
  Grain = 'GRAIN',
  Wool = 'WOOL'
}

export enum Resource {
  Brick = 'BRICK',
  Lumber = 'LUMBER',
  Ore = 'ORE',
  Grain = 'GRAIN',
  Wool = 'WOOL'
}

export enum DevelopmentCard {
  Knight = 'KNIGHT',
  VictoryPoint = 'VICTORY_POINT',
  RoadBuilding = 'ROAD_BUILDING',
  YearOfPlenty = 'YEAR_OF_PLENTY',
  Monopoly = 'MONOPOLY'
}

export enum BuildingType {
  Settlement = 'SETTLEMENT',
  City = 'CITY'
}

export enum GamePhase {
  Lobby = 'LOBBY',
  Setup = 'SETUP',
  MainGame = 'MAIN_GAME',
  Ended = 'ENDED'
}

export interface Board {
  hexes: Hex[];
  harbors: Harbor[];
  roads: Map<string, string>;        // (startPos-endPos) -> playerId
  settlements: Map<string, [string, BuildingType]>;  // pos -> [playerId, buildingType]
}

export interface GameState {
  players: Map<string, Player>;
  board: Board;
  currentTurn: string;
  diceValue: [number, number] | null;
  phase: GamePhase;
  robberPosition: Position;
}

export interface Player {
  id: string;
  name: string;
  resources: Map<Resource, number>;
  developmentCards: DevelopmentCard[];
  roads: Set<string>;
  settlements: Set<string>;
  cities: Set<string>;
  knightsPlayed: number;
}
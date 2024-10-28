export interface Position {
  x: number;
  y: number;
}

export interface Player {
  id: string;
  name: string;
  resources: Record<Resource, number>;
  developmentCards: DevelopmentCard[];
  roads: Set<[Position, Position]>;
  settlements: Set<Position>;
  cities: Set<Position>;
  knightsPlayed: number;
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

export interface Harbor {
  position: Position;
  harborType: HarborType;
}

export enum HarborType {
  Generic = 'GENERIC',
  Brick = 'BRICK',
  Lumber = 'LUMBER',
  Ore = 'ORE',
  Grain = 'GRAIN',
  Wool = 'WOOL'
}

export interface Hex {
  position: Position;
  terrain: TerrainType;
  token?: number | null;
  hasRobber: boolean;
}

export enum TerrainType {
  Hills = 'HILLS',
  Forest = 'FOREST',
  Mountains = 'MOUNTAINS',
  Fields = 'FIELDS',
  Pasture = 'PASTURE',
  Desert = 'DESERT'
}

export interface Board {
  hexes: Hex[];
  harbors: Harbor[];
  roads: Map<[Position, Position], string>;  // Map of road positions to player ID
  settlements: Map<Position, [string, BuildingType]>;  // Map of position to [playerID, buildingType]
}

export enum SetupPhase {
  PlacingFirstSettlement = 'PLACING_FIRST_SETTLEMENT',
  PlacingFirstRoad = 'PLACING_FIRST_ROAD',
  PlacingSecondSettlement = 'PLACING_SECOND_SETTLEMENT',
  PlacingSecondRoad = 'PLACING_SECOND_ROAD'
}

export enum MainGamePhase {
  Rolling = 'ROLLING',
  Building = 'BUILDING',
  Trading = 'TRADING',
  MovingRobber = 'MOVING_ROBBER',
  Discarding = 'DISCARDING'
}

export enum GamePhase {
  Lobby = 'LOBBY',
  Setup = 'SETUP',
  MainGame = 'MAIN_GAME',
  Ended = 'ENDED'
}

export interface GameState {
  players: Map<string, Player>;
  board: Board;
  currentTurn: string;
  diceValue: [number, number] | null;
  phase: GamePhase;
  setupPhase?: SetupPhase;
  mainGamePhase?: MainGamePhase;
  robberPosition: Position;
}
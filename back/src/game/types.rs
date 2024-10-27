// src/game/types.rs
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Resource {
    Brick,
    Lumber,
    Ore,
    Grain,
    Wool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DevelopmentCard {
    Knight,
    VictoryPoint,
    RoadBuilding,
    YearOfPlenty,
    Monopoly,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BuildingType {
    Settlement,
    City,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub resources: HashMap<Resource, u32>,
    pub development_cards: Vec<DevelopmentCard>,
    pub roads: HashSet<(Position, Position)>,
    pub settlements: HashSet<Position>,
    pub cities: HashSet<Position>,
    pub knights_played: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Harbor {
    pub position: Position,
    pub harbor_type: HarborType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HarborType {
    Generic,  // 3:1 trade
    Brick,    // 2:1 trade
    Lumber,
    Ore,
    Grain,
    Wool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hex {
    pub position: Position,
    pub terrain: TerrainType,
    pub token: Option<u8>,
    pub has_robber: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TerrainType {
    Hills,    // Brick
    Forest,   // Lumber
    Mountains, // Ore
    Fields,   // Grain
    Pasture,  // Wool
    Desert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub hexes: Vec<Hex>,
    pub harbors: Vec<Harbor>,
    pub roads: HashMap<(Position, Position), String>,  // String is player_id
    pub settlements: HashMap<Position, (String, BuildingType)>,  // (player_id, building_type)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SetupPhase {
    PlacingFirstSettlement,
    PlacingFirstRoad,
    PlacingSecondSettlement,
    PlacingSecondRoad,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MainGamePhase {
    Rolling,
    Building,
    Trading,
    MovingRobber,
    Discarding,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GamePhase {
    Lobby,
    Setup(SetupPhase),
    MainGame(MainGamePhase),
    Ended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub players: HashMap<String, Player>,
    pub board: Board,
    pub current_turn: String,
    pub dice_value: Option<(u8, u8)>,
    pub phase: GamePhase,
    pub robber_position: Position,
}

// Update your GameCommand enum to include EndTurn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameCommand {
    JoinGame {
        player_name: String,
    },
    RollDice,
    BuildRoad {
        start: Position,
        end: Position,
    },
    BuildSettlement {
        position: Position,
    },
    BuildCity {
        position: Position,
    },
    PlayDevelopmentCard {
        card: DevelopmentCard,
    },
    TradeProposal {
        offer: HashMap<Resource, u32>,
        request: HashMap<Resource, u32>,
    },
    AcceptTrade {
        player_id: String,
    },
    EndTurn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameResponse {
    GameJoined {
        player_id: String,
        game_state: GameState,
    },
    GameStateUpdate {
        game_state: GameState,
    },
    DiceRolled {
        player_id: String,
        values: (u8, u8),
    },
    Error {
        message: String,
    },
}
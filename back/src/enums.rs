use crate::{
    map_instance::{EdgeId, NodeId},
    map_template::Coordinate,
};
use serde::{Deserialize, Serialize};

// Define FreqDeck type directly in this module
pub type FreqDeck = [u8; 5]; // 5 resources

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Color {
    Red = 0,
    Blue = 1,
    Orange = 2,
    White = 3,
}

pub const COLORS: [Color; 4] = [Color::Red, Color::Blue, Color::Orange, Color::White];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Resource {
    Wood,
    Brick,
    Sheep,
    Wheat,
    Ore,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DevCard {
    Knight,
    YearOfPlenty,
    Monopoly,
    RoadBuilding,
    VictoryPoint,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BuildingType {
    Settlement,
    City,
    Road,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NodeRef {
    North,
    Northeast,
    Southeast,
    South,
    Southwest,
    Northwest,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EdgeRef {
    East,
    Southeast,
    Southwest,
    West,
    Northwest,
    Northeast,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionPrompt {
    BuildInitialSettlement,
    BuildInitialRoad,
    PlayTurn,
    Discard,
    MoveRobber,
    DecideTrade,
    DecideAcceptees,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Roll {
        color: u8,
        dice_opt: Option<(u8, u8)>,
    },
    MoveRobber {
        color: u8,
        coordinate: Coordinate,
        victim_opt: Option<u8>,
    },
    Discard {
        color: u8,
    },
    BuildRoad {
        color: u8,
        edge_id: EdgeId,
    },
    BuildSettlement {
        color: u8,
        node_id: NodeId,
    },
    BuildCity {
        color: u8,
        node_id: NodeId,
    },
    BuyDevelopmentCard {
        color: u8,
    },
    PlayKnight {
        color: u8,
    },
    PlayYearOfPlenty {
        color: u8,
        resources: (u8, Option<u8>),
    },
    PlayMonopoly {
        color: u8,
        resource: u8,
    },
    PlayRoadBuilding {
        color: u8,
    },
    MaritimeTrade {
        color: u8,
        give: u8,
        take: u8,
        ratio: u8,
    },
    OfferTrade {
        color: u8,
        trade: (FreqDeck, FreqDeck),
    },
    AcceptTrade {
        color: u8,
        trade: (FreqDeck, FreqDeck),
    },
    RejectTrade {
        color: u8,
    },
    ConfirmTrade {
        color: u8,
        trade: (FreqDeck, FreqDeck, u8),
    },
    CancelTrade {
        color: u8,
    },
    EndTurn {
        color: u8,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MapType {
    Mini,
    Base,
    Tournament,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfiguration {
    pub discard_limit: u8,
    pub vps_to_win: u8,
    pub map_type: MapType,
    pub num_players: u8,
    pub max_ticks: u32,
}

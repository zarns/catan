use serde::{Deserialize, Serialize};
use crate::enums::{Resource, DevCard};
use crate::map_instance::{EdgeId, NodeId};
use crate::map_template::Coordinate;

/// Unique identifier for players
pub type PlayerId = String;

/// Unique identifier for games  
pub type GameId = String;

/// Core player actions that can be taken in the game
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlayerAction {
    // Basic actions
    Roll,
    EndTurn,
    
    // Building actions
    BuildRoad { edge_id: EdgeId },
    BuildSettlement { node_id: NodeId },
    BuildCity { node_id: NodeId },
    
    // Development cards
    BuyDevelopmentCard,
    PlayKnight,
    PlayYearOfPlenty { resources: (Resource, Option<Resource>) },
    PlayMonopoly { resource: Resource },
    PlayRoadBuilding,
    
    // Trading
    MaritimeTrade { give: Resource, take: Resource, ratio: u8 },
    OfferTrade { give: Vec<Resource>, take: Vec<Resource> },
    AcceptTrade { trade_id: String },
    RejectTrade { trade_id: String },
    
    // Special actions
    MoveRobber { coordinate: Coordinate, victim: Option<PlayerId> },
    Discard { resources: Vec<Resource> },
}

/// High-level commands that can be sent to the game system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameCommand {
    /// Player wants to perform an action
    PlayerAction { 
        player_id: PlayerId, 
        action: PlayerAction 
    },
    
    /// System-initiated actions
    StartGame { game_id: GameId },
    AddPlayer { game_id: GameId, player_id: PlayerId, name: String },
    RemovePlayer { game_id: GameId, player_id: PlayerId },
    
    /// Bot actions
    RequestBotAction { game_id: GameId, player_id: PlayerId },
}

/// Events that occur as a result of commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    /// Game lifecycle events
    GameCreated { game_id: GameId },
    GameStarted { game_id: GameId },
    GameEnded { game_id: GameId, winner: Option<PlayerId> },
    
    /// Player events
    PlayerJoined { game_id: GameId, player_id: PlayerId, name: String },
    PlayerLeft { game_id: GameId, player_id: PlayerId },
    
    /// Action events
    ActionExecuted { 
        game_id: GameId, 
        player_id: PlayerId, 
        action: PlayerAction,
        success: bool,
        message: String,
    },
    
    /// State changes
    GameStateChanged { 
        game_id: GameId, 
        new_state: crate::game::GameState 
    },
    
    /// Turn management
    TurnChanged { 
        game_id: GameId, 
        current_player: PlayerId 
    },
    
    /// Dice events
    DiceRolled { 
        game_id: GameId, 
        player_id: PlayerId, 
        dice: [u8; 2] 
    },
    
    /// Error events
    Error { 
        game_id: GameId, 
        player_id: Option<PlayerId>, 
        error: String 
    },
}

/// Result of executing an action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    pub message: String,
    pub events: Vec<GameEvent>,
}

impl ActionResult {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            events: Vec::new(),
        }
    }
    
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            events: Vec::new(),
        }
    }
    
    pub fn with_events(mut self, events: Vec<GameEvent>) -> Self {
        self.events = events;
        self
    }
}

/// Convert from the internal Action enum to our PlayerAction
impl From<crate::enums::Action> for PlayerAction {
    fn from(action: crate::enums::Action) -> Self {
        use crate::enums::Action as EnumAction;
        
        match action {
            EnumAction::Roll { .. } => PlayerAction::Roll,
            EnumAction::BuildRoad { edge_id, .. } => PlayerAction::BuildRoad { edge_id },
            EnumAction::BuildSettlement { node_id, .. } => PlayerAction::BuildSettlement { node_id },
            EnumAction::BuildCity { node_id, .. } => PlayerAction::BuildCity { node_id },
            EnumAction::BuyDevelopmentCard { .. } => PlayerAction::BuyDevelopmentCard,
            EnumAction::PlayKnight { .. } => PlayerAction::PlayKnight,
            EnumAction::EndTurn { .. } => PlayerAction::EndTurn,
            EnumAction::MoveRobber { coordinate, victim_opt, .. } => {
                PlayerAction::MoveRobber { 
                    coordinate, 
                    victim: victim_opt.map(|c| format!("player_{}", c))
                }
            },
            _ => PlayerAction::EndTurn, // Default for unhandled actions
        }
    }
}

/// Convert from PlayerAction to internal Action enum
impl From<PlayerAction> for crate::enums::Action {
    fn from(action: PlayerAction) -> Self {
        use crate::enums::Action as EnumAction;
        
        match action {
            PlayerAction::Roll => EnumAction::Roll { color: 0, dice_opt: None },
            PlayerAction::BuildRoad { edge_id } => EnumAction::BuildRoad { color: 0, edge_id },
            PlayerAction::BuildSettlement { node_id } => EnumAction::BuildSettlement { color: 0, node_id },
            PlayerAction::BuildCity { node_id } => EnumAction::BuildCity { color: 0, node_id },
            PlayerAction::BuyDevelopmentCard => EnumAction::BuyDevelopmentCard { color: 0 },
            PlayerAction::PlayKnight => EnumAction::PlayKnight { color: 0 },
            PlayerAction::EndTurn => EnumAction::EndTurn { color: 0 },
            PlayerAction::MoveRobber { coordinate, .. } => {
                EnumAction::MoveRobber { 
                    color: 0, 
                    coordinate, 
                    victim_opt: None 
                }
            },
            _ => EnumAction::EndTurn { color: 0 }, // Default for unhandled actions
        }
    }
} 
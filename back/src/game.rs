use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Basic game types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Resource {
    Brick,
    Lumber,
    Wool,
    Grain,
    Ore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DevelopmentCard {
    Knight,
    VictoryPoint,
    RoadBuilding,
    YearOfPlenty,
    Monopoly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub color: String,
    pub resources: HashMap<Resource, u32>,
    pub dev_cards: Vec<DevelopmentCard>,
    pub knights_played: u32,
    pub victory_points: u32,
    pub longest_road: bool,
    pub largest_army: bool,
}

impl Player {
    pub fn new(id: String, name: String, color: String) -> Self {
        Player {
            id,
            name,
            color,
            resources: HashMap::new(),
            dev_cards: Vec::new(),
            knights_played: 0,
            victory_points: 0,
            longest_road: false,
            largest_army: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameBoard {
    // Board details will be implemented later
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameAction {
    RollDice,
    BuildRoad,
    BuildSettlement,
    BuildCity,
    BuyDevelopmentCard,
    PlayDevelopmentCard,
    TradeWithBank,
    TradeWithPlayer,
    EndTurn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub players: Vec<Player>,
    pub board: GameBoard,
    pub current_player_index: usize,
    pub dice_rolled: bool,
    pub winner: Option<String>,
    pub turns: u32,
}

impl Game {
    pub fn new(id: String, player_names: Vec<String>) -> Self {
        let colors = vec!["red", "blue", "white", "orange"];
        
        let players = player_names
            .into_iter()
            .enumerate()
            .map(|(i, name)| {
                let player_id = format!("player_{}", i);
                let color = colors[i % colors.len()].to_string();
                Player::new(player_id, name, color)
            })
            .collect();

        Game {
            id,
            players,
            board: GameBoard {},
            current_player_index: 0,
            dice_rolled: false,
            winner: None,
            turns: 0,
        }
    }

    pub fn process_action(&mut self, player_id: &str, action: GameAction) -> Result<(), String> {
        // Basic validation
        if self.winner.is_some() {
            return Err("Game already has a winner".into());
        }

        // Game action handling will be implemented later
        Ok(())
    }
}

// Game simulation for bot play
pub fn simulate_bot_game(num_players: u8) -> Game {
    let player_names = (0..num_players)
        .map(|i| format!("Bot {}", i + 1))
        .collect();
    
    let game_id = format!("sim_{}", uuid::Uuid::new_v4());
    Game::new(game_id, player_names)
}

// Initial setup for a game against Catanatron
pub fn start_human_vs_catanatron(human_name: String, num_bots: u8) -> Game {
    let mut player_names = vec![human_name];
    
    for i in 0..num_bots {
        player_names.push(format!("Catanatron {}", i + 1));
    }
    
    let game_id = format!("hvs_{}", uuid::Uuid::new_v4());
    Game::new(game_id, player_names)
} 
use catan::enums::{Action, GameConfiguration, MapType};
use catan::global_state::GlobalState;
use catan::map_instance::MapInstance;
use catan::players::{Player, RandomPlayer};
use catan::state::State;
use clap::{Parser, ValueEnum};
use std::sync::Arc;
use std::time::Instant;

// CLI Arguments
#[derive(Parser)]
#[command(
    name = "Catan Simulator",
    about = "Simulates games between different player types",
    version = "0.1.0"
)]
struct Args {
    /// Player types configuration (e.g., RR for Random vs Random)
    #[arg(short, long, default_value = "RR")]
    players: String,

    /// Number of games to simulate
    #[arg(short, long, default_value_t = 100)]
    num_games: usize,

    /// Whether to print detailed logs of each game
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

// Player type enum
#[derive(Debug, Clone, Copy, ValueEnum)]
enum PlayerType {
    Random,
    MCTS,
}

fn create_player(player_type: PlayerType) -> Box<dyn Player> {
    match player_type {
        PlayerType::Random => Box::new(RandomPlayer {}),
        PlayerType::MCTS => {
            // When MCTS is implemented, uncomment this line
            // Box::new(MctsPlayer::new())
            // For now, default to random
            Box::new(RandomPlayer {})
        }
    }
}

fn parse_player_config(config: &str) -> Vec<PlayerType> {
    config
        .chars()
        .map(|c| match c {
            'R' => PlayerType::Random,
            'M' => PlayerType::MCTS,
            _ => PlayerType::Random, // Default to random for unknown types
        })
        .collect()
}

// Runs a single game between the specified player types
fn run_game(player_types: &[PlayerType], verbose: bool) -> (Option<usize>, usize) {
    // Create global state which has map templates
    let global_state = GlobalState::new();
    
    // Create game configuration
    let config = Arc::new(GameConfiguration {
        map_type: MapType::Base,
        num_players: player_types.len() as u8,
        discard_limit: 7,
        vps_to_win: 10,
        max_ticks: 1000, // Prevent infinite games
    });
    
    // Create map instance using global state's templates
    let map_instance = Arc::new(MapInstance::new(
        &global_state.base_map_template,
        &global_state.dice_probas,
        rand::random::<u64>(), // Use random seed
    ));
    
    // Create state
    let mut state = State::new(config, map_instance);

    // Create players
    let players: Vec<Box<dyn Player>> = player_types
        .iter()
        .map(|&pt| create_player(pt))
        .collect();

    // Play the game
    let mut actions = 0;
    let start_time = Instant::now();

    while actions < 10000 { // Safety limit to prevent infinite loops
        // Get the available actions
        let playable_actions = state.generate_playable_actions();
        
        if playable_actions.is_empty() {
            if verbose {
                println!("No playable actions available. Game ended.");
            }
            break;
        }

        // Get the current player index
        let current_color = state.get_current_color() as usize;
        
        // Let the player decide which action to take
        let chosen_action = players[current_color].decide(&state, &playable_actions);
        
        if verbose {
            println!("Player {} took action: {:?}", current_color, chosen_action);
        }

        // Apply the action
        state.apply_action(chosen_action);
        actions += 1;

        // Check if the game is over
        if let Some(winner) = state.winner() {
            if verbose {
                println!("Player {} has won!", winner);
            }
            return (Some(winner as usize), actions);
        }
    }

    let duration = start_time.elapsed();
    
    if verbose {
        println!(
            "Game completed in {:.2?} with {} actions. No winner determined (hit action limit).",
            duration, actions
        );
    }

    (None, actions)
}

fn main() {
    // Initialize the logger
    env_logger::init();

    // Parse command line arguments
    let args = Args::parse();
    
    // Parse player configuration
    let player_types = parse_player_config(&args.players);
    
    if player_types.len() < 2 || player_types.len() > 4 {
        eprintln!("Error: Player configuration must specify 2-4 players");
        std::process::exit(1);
    }
    
    println!("Simulating {} games with configuration: {}", args.num_games, args.players);
    println!("Player types: {:?}", player_types);
    
    // Initialize statistics
    let mut wins = vec![0; player_types.len()];
    let mut total_actions = 0;
    let total_start_time = Instant::now();
    
    // Run the simulations
    for i in 0..args.num_games {
        if args.verbose {
            println!("\nStarting game {}/{}", i + 1, args.num_games);
        }
        
        let (winner, actions) = run_game(&player_types, args.verbose);
        
        if let Some(winner_idx) = winner {
            wins[winner_idx] += 1;
        }
        total_actions += actions;
    }
    
    let total_duration = total_start_time.elapsed();
    
    // Print statistics
    println!("\n=== Simulation Results ===");
    println!("Total time: {:.2?}", total_duration);
    println!("Average actions per game: {:.2}", total_actions as f64 / args.num_games as f64);
    println!("Win distribution:");
    
    for (i, &win_count) in wins.iter().enumerate() {
        let win_rate = win_count as f64 / args.num_games as f64 * 100.0;
        println!("  Player {} ({:?}): {}/{} games ({:.2}%)", 
                 i, player_types[i], win_count, args.num_games, win_rate);
    }
} 
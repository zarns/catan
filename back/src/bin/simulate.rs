use catan::enums::Action;
use catan::game::*;
use std::env;

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let mut num_games = 1;
    let mut verbose = false;
    let mut players_config = "RRRR".to_string(); // Default: 4 random players

    // Parse command line arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-n" | "--num_games" => {
                if i + 1 < args.len() {
                    num_games = args[i + 1].parse().unwrap_or(1);
                    i += 1;
                }
            }
            "-p" | "--players" => {
                if i + 1 < args.len() {
                    players_config = args[i + 1].clone();
                    i += 1;
                }
            }
            "-v" | "--verbose" => {
                verbose = true;
            }
            _ => {}
        }
        i += 1;
    }

    log::info!("🎮 Catan Game Simulation");
    log::info!("=======================");
    log::info!("Configuration:");
    log::info!("  - Players: {} ({})", players_config, players_config.len());
    log::info!("  - Number of games: {num_games}");
    log::info!("  - Verbose: {verbose}");

    let mut wins = vec![0; players_config.len()];
    let mut total_turns = 0;
    let mut completed_games = 0;

    for game_num in 0..num_games {
        if num_games > 1 {
            log::info!("\n🎯 Game {} of {}", game_num + 1, num_games);
        }

        let result = simulate_single_game(players_config.len() as u8, verbose);
        match result {
            Some((winner, turns)) => {
                wins[winner as usize] += 1;
                total_turns += turns;
                completed_games += 1;
                if num_games > 1 {
                    log::info!("  Winner: Player {winner} in {turns} turns");
                }
            }
            None => {
                if num_games > 1 {
                    log::warn!("  Game did not complete within turn limit");
                }
            }
        }
    }

    if num_games > 1 {
        log::info!("\n📊 Tournament Results:");
        log::info!("====================");
        for (i, &win_count) in wins.iter().enumerate() {
            let win_rate = if completed_games > 0 {
                (win_count as f64 / completed_games as f64) * 100.0
            } else {
                0.0
            };
            log::info!("Player {i}: {win_count} wins ({win_rate:.1}%)");
        }
        log::info!("Completed games: {completed_games}/{num_games}");
        if completed_games > 0 {
            log::info!(
                "Average turns per game: {:.1}",
                total_turns as f64 / completed_games as f64
            );
        }
    }
}

fn simulate_single_game(num_players: u8, verbose: bool) -> Option<(u8, u32)> {
    // Create a real game with actual game logic
    let mut game = simulate_bot_game(num_players);

    if verbose {
        log::debug!(
            "✅ Created game with real State and {} players",
            game.players.len()
        );
    }

    // Print initial state
    if let Some(ref state) = game.state {
        if verbose {
            log::debug!("🏁 Initial state:");
            log::debug!(
                "   - Is initial build phase: {}",
                state.is_initial_build_phase()
            );
            log::debug!("   - Current player: {}", state.get_current_color());
            log::debug!(
                "   - Players: {:?}",
                game.players.iter().map(|p| &p.name).collect::<Vec<_>>()
            );
            log::debug!("📊 Initial Victory Points:");
            // Show initial victory points breakdown
            for color in 0..state.get_num_players() {
                let vp = state.get_actual_victory_points(color);
                let settlements = state.get_settlements(color).len();
                let cities = state.get_cities(color).len();
                let roads = state.get_roads_by_color()[color as usize];
                log::debug!(
                    "   🏆 Player {color}: {vp} VP (settlements: {settlements}, cities: {cities}, roads: {roads})"
                );
            }
        }
    }

    // Simulate turns with real actions
    let mut turn_count = 0;
    const MAX_TURNS: u32 = 5000; // Higher limit for real games - increased for thorough testing
    let mut last_vp_log = 0;

    if verbose {
        log::info!("🎯 Starting simulation with MAX_TURNS = {MAX_TURNS}");
    }

    while turn_count < MAX_TURNS {
        // Check for winner
        if let Some(ref state) = game.state {
            if let Some(winner) = state.winner() {
                if verbose {
                    log::info!("🎉 GAME WON! Player {winner} is the winner!");
                    log::info!("📊 Final Victory Points:");
                    // Show final victory points breakdown
                    for color in 0..state.get_num_players() {
                        let vp = state.get_actual_victory_points(color);
                        let settlements = state.get_settlements(color).len();
                        let cities = state.get_cities(color).len();
                        let roads = state.get_roads_by_color()[color as usize];
                        log::info!(
                            "   🏆 Player {color}: {vp} VP (settlements: {settlements}, cities: {cities}, roads: {roads})"
                        );
                    }
                }
                log::info!("✅ Game completed in {turn_count} turns");
                return Some((winner, turn_count));
            }
        }

        // Get available actions and current player info
        let (available_actions, current_player) = if let Some(ref state) = game.state {
            let actions = state.generate_playable_actions();
            let player = state.get_current_color();
            (actions, player)
        } else {
            if verbose {
                log::error!("❌ No game state available!");
            }
            break;
        };

        // Log turn info every 10 turns or for debugging
        if verbose && (turn_count % 10 == 0 || turn_count < 5) {
            log::debug!(
                "\n🎯 Turn {}: Player {} has {} actions",
                turn_count + 1,
                current_player,
                available_actions.len()
            );
        }

        if available_actions.is_empty() {
            if verbose {
                log::error!("❌ No actions available! This is a bug.");
                if let Some(ref state) = game.state {
                    log::debug!("🔍 Debug info:");
                    log::debug!("   - Phase: {:?}", state.get_action_prompt());
                    log::debug!("   - Is initial: {}", state.is_initial_build_phase());
                    log::debug!("   - Is discarding: {}", state.is_discarding());
                    log::debug!("   - Is moving robber: {}", state.is_moving_robber());
                    state.log_victory_points();
                }
            }
            break;
        }

        // Show first few available actions for debugging (only early turns)
        if verbose && turn_count < 5 {
            if available_actions.len() <= 5 {
                log::debug!("   Available actions: {available_actions:?}");
            } else {
                log::debug!("   First 3 actions: {:?}", &available_actions[..3]);
                log::debug!("   ... and {} more", available_actions.len() - 3);
            }
        }

        // Show current player's detailed status for building/VP analysis
        if verbose && (turn_count % 10 == 0 || turn_count < 5) {
            if let Some(ref state) = game.state {
                let vp = state.get_actual_victory_points(current_player);
                let settlements = state.get_settlements(current_player).len();
                let cities = state.get_cities(current_player).len();
                let roads = state.get_roads_by_color()[current_player as usize];

                // Log player's resources
                let hand = state.get_player_hand(current_player);
                log::debug!(
                    "   📊 Player {current_player} status: {vp} VP (settlements: {settlements}, cities: {cities}, roads: {roads})"
                );
                log::debug!(
                    "   💰 Resources: Wood={}, Brick={}, Sheep={}, Wheat={}, Ore={}",
                    hand[0],
                    hand[1],
                    hand[2],
                    hand[3],
                    hand[4]
                );
            }
        }

        // Choose action: prioritize building actions over EndTurn for more interesting gameplay
        let action = choose_best_action(&available_actions);

        if verbose && (turn_count % 10 == 0 || turn_count < 5) {
            log::debug!("🤖 Player {current_player} action: {action:?}");
        }

        // Apply the action using real game logic
        if let Some(ref mut state) = game.state {
            state.apply_action(action);
        }

        // Log victory points every 20 turns or when something interesting happens
        let should_log_vp = verbose
            && (turn_count % 20 == 0
                || matches!(
                    action,
                    Action::BuildSettlement { .. } | Action::BuildCity { .. }
                ));

        if should_log_vp && turn_count != last_vp_log {
            log::info!("📊 Victory Points Status (Turn {}):", turn_count + 1);
            if let Some(ref state) = game.state {
                // Custom victory points logging that actually shows the values
                for color in 0..state.get_num_players() {
                    let vp = state.get_actual_victory_points(color);
                    let settlements = state.get_settlements(color).len();
                    let cities = state.get_cities(color).len();
                    let roads = state.get_roads_by_color()[color as usize];
                    log::info!(
                        "   🏆 Player {color}: {vp} VP (settlements: {settlements}, cities: {cities}, roads: {roads})"
                    );
                }
            }
            last_vp_log = turn_count;
        }

        turn_count += 1;

        // Small delay for readability (only for early turns and verbose mode)
        if verbose && turn_count < 20 {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    if turn_count >= MAX_TURNS && verbose {
        log::info!("⏰ Simulation ended after {MAX_TURNS} turns (max reached)");
        if let Some(ref state) = game.state {
            log::info!("📊 Final Victory Points:");
            // Show final victory points breakdown
            for color in 0..state.get_num_players() {
                let vp = state.get_actual_victory_points(color);
                let settlements = state.get_settlements(color).len();
                let cities = state.get_cities(color).len();
                let roads = state.get_roads_by_color()[color as usize];
                log::info!(
                        "   🏆 Player {color}: {vp} VP (settlements: {settlements}, cities: {cities}, roads: {roads})"
                    );
            }
        }
    }

    None
}

/// Choose the most interesting action from available options
/// Prioritizes building actions over basic actions like EndTurn
fn choose_best_action(actions: &[Action]) -> Action {
    // Priority order: Settlement > City > Road > Development > Trade > Robber > Other > EndTurn

    // Check for building actions first (most important)
    if let Some(action) = actions
        .iter()
        .find(|a| matches!(a, Action::BuildSettlement { .. }))
    {
        return *action;
    }

    if let Some(action) = actions
        .iter()
        .find(|a| matches!(a, Action::BuildCity { .. }))
    {
        return *action;
    }

    if let Some(action) = actions
        .iter()
        .find(|a| matches!(a, Action::BuildRoad { .. }))
    {
        return *action;
    }

    // Check for development card actions
    if let Some(action) = actions
        .iter()
        .find(|a| matches!(a, Action::BuyDevelopmentCard { .. }))
    {
        return *action;
    }

    if let Some(action) = actions.iter().find(|a| {
        matches!(
            a,
            Action::PlayKnight { .. }
                | Action::PlayYearOfPlenty { .. }
                | Action::PlayMonopoly { .. }
                | Action::PlayRoadBuilding { .. }
        )
    }) {
        return *action;
    }

    // Check for trade actions
    if let Some(action) = actions
        .iter()
        .find(|a| matches!(a, Action::MaritimeTrade { .. }))
    {
        return *action;
    }

    // Check for robber actions
    if let Some(action) = actions
        .iter()
        .find(|a| matches!(a, Action::MoveRobber { .. }))
    {
        return *action;
    }

    // Check for discard actions
    if let Some(action) = actions.iter().find(|a| matches!(a, Action::Discard { .. })) {
        return *action;
    }

    // Check for roll action
    if let Some(action) = actions.iter().find(|a| matches!(a, Action::Roll { .. })) {
        return *action;
    }

    // Fall back to first action (likely EndTurn)
    actions[0]
}

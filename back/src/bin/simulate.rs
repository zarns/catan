use catan::enums::Action;
use catan::game::*;
use catan::players::{
    AlphaBetaPlayer, BotPlayer, GreedyPlayer, RandomPlayer, WeightedRandomPlayer,
};
use std::collections::HashMap;
use std::env;

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let mut num_games = 1;
    let mut verbose = false;
    let mut players_config = "RRRR".to_string(); // Default: 4 random players
    let mut dump_timeout = false;

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
            "-t" | "--dump-timeout" => {
                dump_timeout = true;
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

    let num_players = players_config.len();
    let mut wins = vec![0u32; num_players];
    let mut total_turns: u64 = 0;
    let mut completed_games: u32 = 0;
    // Per-player VP aggregates for completed games
    let mut vp_sum = vec![0u64; num_players];
    let mut vp_sum_sq = vec![0u128; num_players];
    // Termination reasons
    let mut timeout_games: u32 = 0;
    let mut no_actions_games: u32 = 0;
    let mut no_state_games: u32 = 0;
    let mut no_actions_by_prompt: HashMap<String, u32> = HashMap::new();
    let mut timeout_turns: u64 = 0;
    let mut timeout_bank_zero_sum: u64 = 0;
    let mut timeout_actions_sum: u64 = 0;
    let mut no_actions_turns: u64 = 0;
    let mut timeout_vp_sum: u64 = 0; // sum of total VP across players at timeout
    let mut no_actions_vp_sum: u64 = 0; // sum of total VP across players at no-actions

    // Build bot lineup from players_config (R,G,W,A)
    let (bots, bot_labels) = build_bots_from_config(&players_config);

    for game_num in 0..num_games {
        if num_games > 1 {
            log::info!("\n🎯 Game {} of {}", game_num + 1, num_games);
        }

        let result = simulate_single_game(&bots, verbose, dump_timeout);
        match result {
            SimOutcome::Completed { winner, turns, vps } => {
                wins[winner as usize] += 1;
                total_turns += turns as u64;
                completed_games += 1;
                for (i, &vp) in vps.iter().enumerate() {
                    vp_sum[i] += vp as u64;
                    vp_sum_sq[i] += (vp as u128) * (vp as u128);
                }
                if num_games > 1 {
                    let label = bot_labels[winner as usize].as_str();
                    log::info!("  Winner: Player {winner} ({label}) in {turns} turns");
                }
            }
            SimOutcome::Timeout {
                turns,
                vps,
                bank_zeroes,
                actions_len,
            } => {
                timeout_games += 1;
                timeout_turns += turns as u64;
                timeout_vp_sum += vps.iter().map(|&v| v as u64).sum::<u64>();
                timeout_bank_zero_sum += bank_zeroes as u64;
                timeout_actions_sum += actions_len as u64;
            }
            SimOutcome::NoActions { turns, prompt, vps } => {
                no_actions_games += 1;
                no_actions_turns += turns as u64;
                no_actions_vp_sum += vps.iter().map(|&v| v as u64).sum::<u64>();
                *no_actions_by_prompt.entry(prompt).or_insert(0) += 1;
            }
            SimOutcome::NoState => no_state_games += 1,
        }
    }

    // Always print a summary to stdout so it's visible without RUST_LOG
    if num_games > 1 {
        println!("\n📊 Tournament Results:\n====================");
    } else {
        println!("\n📊 Game Result:\n=============");
    }
    for (i, &win_count) in wins.iter().enumerate() {
        let win_rate = if completed_games > 0 {
            (win_count as f64 / completed_games as f64) * 100.0
        } else {
            0.0
        };
        let (mean_vp, std_vp) = if completed_games > 0 {
            let n = completed_games as f64;
            let mean = vp_sum[i] as f64 / n;
            let mean_sq = vp_sum_sq[i] as f64 / n;
            let var = (mean_sq - mean * mean).max(0.0);
            (mean, var.sqrt())
        } else {
            (0.0, 0.0)
        };
        let label = &bot_labels[i];
        println!(
            "Player {i} ({label}): {win_count} wins ({win_rate:.1}%), mean VP: {mean_vp:.2} ± {std_vp:.2}"
        );
    }
    println!("Completed games: {completed_games}/{num_games}");
    let incomplete = num_games as u32 - completed_games;
    if incomplete > 0 {
        println!(
            "Incomplete: {} (timeouts: {}, no_actions: {}, no_state: {})",
            incomplete, timeout_games, no_actions_games, no_state_games
        );
        if timeout_games > 0 {
            let avg = timeout_turns as f64 / timeout_games as f64;
            let mean_vp_sum = timeout_vp_sum as f64 / timeout_games as f64;
            let mean_bank_zeroes = timeout_bank_zero_sum as f64 / timeout_games as f64;
            let mean_actions = timeout_actions_sum as f64 / timeout_games as f64;
            println!(
                "  - Timeouts: avg turns {:.1}, mean total VP at timeout {:.2}, mean bank zero-res types {:.2}, mean legal actions {:.2}",
                avg, mean_vp_sum, mean_bank_zeroes, mean_actions
            );
        }
        if no_actions_games > 0 {
            let avg = no_actions_turns as f64 / no_actions_games as f64;
            let mean_vp_sum = no_actions_vp_sum as f64 / no_actions_games as f64;
            println!(
                "  - NoActions: avg turns {:.1}, mean total VP {:.2}",
                avg, mean_vp_sum
            );
            // Print top 3 prompts causing no_actions
            let mut items: Vec<(String, u32)> = no_actions_by_prompt.into_iter().collect();
            items.sort_by(|a, b| b.1.cmp(&a.1));
            for (i, (prompt, count)) in items.into_iter().take(3).enumerate() {
                println!("    {}. {}: {}", i + 1, prompt, count);
            }
        }
    }
    if completed_games > 0 {
        println!(
            "Average turns per game: {:.1}",
            total_turns as f64 / completed_games as f64
        );
    }
}

enum SimOutcome {
    Completed {
        winner: u8,
        turns: u32,
        vps: Vec<u8>,
    },
    Timeout {
        turns: u32,
        vps: Vec<u8>,
        bank_zeroes: u8,
        actions_len: usize,
    },
    NoActions {
        turns: u32,
        prompt: String,
        vps: Vec<u8>,
    },
    NoState,
}

fn simulate_single_game(
    bots: &[Box<dyn BotPlayer>],
    verbose: bool,
    dump_timeout: bool,
) -> SimOutcome {
    // Create a real game with actual game logic
    let mut game = simulate_bot_game(bots.len() as u8);

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
    const MAX_TURNS: u32 = 10000; // Higher limit for real games - increased for thorough testing
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
                let final_vps = collect_final_vps(state);
                log::info!("✅ Game completed in {turn_count} turns");
                return SimOutcome::Completed {
                    winner,
                    turns: turn_count,
                    vps: final_vps,
                };
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
            return SimOutcome::NoState;
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
                    log::debug!("   - Rolled this turn: {}", state.current_player_rolled());
                    let color = state.get_current_color();
                    let hand = state.get_player_hand(color);
                    log::debug!(
                        "   - Current player {} hand: Wd={} Br={} Sh={} Wh={} Or={}",
                        color,
                        hand[0],
                        hand[1],
                        hand[2],
                        hand[3],
                        hand[4]
                    );
                    let dev = state.get_player_devhand(color);
                    log::debug!(
                        "   - Dev hand: K={} YOP={} Mono={} RB={} VP={}",
                        dev[0],
                        dev[1],
                        dev[2],
                        dev[3],
                        dev[4]
                    );
                    let bank = state.get_bank_resources();
                    log::debug!(
                        "   - Bank: Wd={} Br={} Sh={} Wh={} Or={}",
                        bank[0],
                        bank[1],
                        bank[2],
                        bank[3],
                        bank[4]
                    );
                    // Also dump candidate possibilities by category for PlayTurn
                    if matches!(
                        state.get_action_prompt(),
                        catan::enums::ActionPrompt::PlayTurn
                    ) {
                        let color = state.get_current_color();
                        let mut cats = Vec::new();
                        cats.push((
                            "settlement",
                            state.settlement_possibilities(color, false).len(),
                        ));
                        cats.push(("road", state.road_possibilities(color, false).len()));
                        cats.push(("city", state.city_possibilities(color).len()));
                        cats.push((
                            "buy_dev",
                            state.buy_development_card_possibilities(color).len(),
                        ));
                        cats.push(("maritime", state.maritime_trade_possibilities(color).len()));
                        log::debug!("   - Category sizes: {:?}", cats);
                    }
                    state.log_victory_points();
                }
            }
            if let Some(ref state) = game.state {
                let prompt = format!("{:?}", state.get_action_prompt());
                let vps = collect_final_vps(state);
                return SimOutcome::NoActions {
                    turns: turn_count,
                    prompt,
                    vps,
                };
            } else {
                return SimOutcome::NoState;
            }
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

        // Choose action via configured bot for the current player
        let bot_idx = current_player as usize;
        let action = if bot_idx < bots.len() {
            bots[bot_idx].decide(game.state.as_ref().unwrap(), &available_actions)
        } else {
            // Fallback: first action
            available_actions[0]
        };

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

    // If we reach here, no winner within MAX_TURNS
    if let Some(ref state) = game.state {
        let vps = collect_final_vps(state);
        // Quick diagnostics at timeout
        let bank = state.get_bank_resources();
        let bank_zeroes = bank.iter().filter(|&&c| c == 0).count() as u8;
        let actions_len = state.generate_playable_actions().len();
        if dump_timeout {
            println!("\n⏰ Timeout dump:");
            println!("  - Turn: {}", turn_count);
            println!("  - Current player: {}", state.get_current_color());
            println!("  - Action prompt: {:?}", state.get_action_prompt());
            println!("  - Rolled this turn: {}", state.current_player_rolled());
            println!(
                "  - Bank: wood={} brick={} sheep={} wheat={} ore={} (zero-types={})",
                bank[0], bank[1], bank[2], bank[3], bank[4], bank_zeroes
            );
            for color in 0..state.get_num_players() {
                let hand = state.get_player_hand(color);
                let dev = state.get_player_devhand(color);
                println!(
                    "  - P{}: VP={} | hand [w={},b={},s={},w={},o={}] | dev [K={},YOP={},M={},RB={},VP={}]",
                    color,
                    state.get_actual_victory_points(color),
                    hand[0], hand[1], hand[2], hand[3], hand[4],
                    dev[0], dev[1], dev[2], dev[3], dev[4]
                );
            }
            let color = state.get_current_color();
            println!(
                "  - Legal actions now: {} (settle={}, road={}, city={}, buy_dev={}, maritime={})",
                actions_len,
                state.settlement_possibilities(color, false).len(),
                state.road_possibilities(color, false).len(),
                state.city_possibilities(color).len(),
                state.buy_development_card_possibilities(color).len(),
                state.maritime_trade_possibilities(color).len()
            );
            let acts = state.generate_playable_actions();
            let preview = acts.iter().take(5).collect::<Vec<_>>();
            println!("  - Actions (up to 5): {:?}", preview);
        }
        return SimOutcome::Timeout {
            turns: turn_count,
            vps,
            bank_zeroes,
            actions_len,
        };
    }
    SimOutcome::NoState
}

/// Choose the most interesting action from available options
/// Prioritizes building actions over basic actions like EndTurn
// removed unused choose_best_action helper; decisions now come from configured bots

fn build_bots_from_config(config: &str) -> (Vec<Box<dyn BotPlayer>>, Vec<String>) {
    let colors = ["red", "blue", "white", "orange"]; // cosmetic only
    let mut bots: Vec<Box<dyn BotPlayer>> = Vec::new();
    let mut labels: Vec<String> = Vec::new();

    for (i, c) in config.chars().enumerate() {
        match c {
            'G' | 'g' => {
                bots.push(Box::new(GreedyPlayer::new(
                    format!("player_{i}"),
                    format!("Greedy {i}"),
                    colors[i % colors.len()].to_string(),
                )));
                labels.push("Greedy".to_string());
            }
            'W' | 'w' => {
                bots.push(Box::new(WeightedRandomPlayer::new(
                    format!("player_{i}"),
                    format!("Weighted {i}"),
                    colors[i % colors.len()].to_string(),
                )));
                labels.push("WeightedRandom".to_string());
            }
            'A' | 'a' => {
                bots.push(Box::new(AlphaBetaPlayer::new(
                    format!("player_{i}"),
                    format!("AlphaBeta {i}"),
                    colors[i % colors.len()].to_string(),
                )));
                labels.push("AlphaBeta".to_string());
            }
            _ => {
                bots.push(Box::new(RandomPlayer::new(
                    format!("player_{i}"),
                    format!("Random {i}"),
                    colors[i % colors.len()].to_string(),
                )));
                labels.push("Random".to_string());
            }
        }
    }

    (bots, labels)
}

fn collect_final_vps(state: &catan::state::State) -> Vec<u8> {
    let num_players = state.get_num_players();
    (0..num_players)
        .map(|c| state.get_actual_victory_points(c))
        .collect()
}

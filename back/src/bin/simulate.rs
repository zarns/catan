use catan::enums::Action;
use catan::game::*;

fn main() {
    env_logger::init();

    println!("🎮 Catan Game Simulation");
    println!("=======================");

    // Create a real game with actual game logic
    let mut game = simulate_bot_game(4); // 4 players for full Catan experience
    println!(
        "✅ Created game with real State and {} players",
        game.players.len()
    );

    // Print initial state
    if let Some(ref state) = game.state {
        println!("🏁 Initial state:");
        println!(
            "   - Is initial build phase: {}",
            state.is_initial_build_phase()
        );
        println!("   - Current player: {}", state.get_current_color());
        println!(
            "   - Players: {:?}",
            game.players.iter().map(|p| &p.name).collect::<Vec<_>>()
        );
        state.log_victory_points();
    }

    // Simulate turns with real actions
    let mut turn_count = 0;
    const MAX_TURNS: u32 = 500; // Higher limit for real games - increased for thorough testing
    let mut last_vp_log = 0;

    while turn_count < MAX_TURNS {
        // Check for winner
        if let Some(ref state) = game.state {
            if let Some(winner) = state.winner() {
                println!("🎉 GAME WON! Player {} is the winner!", winner);
                println!("📊 Final Victory Points:");
                state.log_victory_points();
                break;
            }
        }

        // Get available actions and current player info
        let (available_actions, current_player) = if let Some(ref state) = game.state {
            let actions = state.generate_playable_actions();
            let player = state.get_current_color();
            (actions, player)
        } else {
            println!("❌ No game state available!");
            break;
        };

        println!(
            "\n🎯 Turn {}: Player {} has {} actions",
            turn_count + 1,
            current_player,
            available_actions.len()
        );

        if available_actions.is_empty() {
            println!("❌ No actions available! This is a bug.");
            if let Some(ref state) = game.state {
                println!("🔍 Debug info:");
                println!("   - Phase: {:?}", state.get_action_prompt());
                println!("   - Is initial: {}", state.is_initial_build_phase());
                println!("   - Is discarding: {}", state.is_discarding());
                println!("   - Is moving robber: {}", state.is_moving_robber());
                state.log_victory_points();
            }
            break;
        }

        // Show first few available actions for debugging
        if available_actions.len() <= 5 {
            println!("   Available actions: {:?}", available_actions);
        } else {
            println!("   First 3 actions: {:?}", &available_actions[..3]);
            println!("   ... and {} more", available_actions.len() - 3);
        }

        // Show current player's detailed status for building/VP analysis
        if let Some(ref state) = game.state {
            let vp = state.get_actual_victory_points(current_player);
            let settlements = state.get_settlements(current_player).len();
            let cities = state.get_cities(current_player).len();
            let roads = state.get_roads_by_color()[current_player as usize];
            println!(
                "   📊 Player {} status: {} VP (settlements: {}, cities: {}, roads: {})",
                current_player, vp, settlements, cities, roads
            );
        }

        // Choose action: prioritize building actions over EndTurn for more interesting gameplay
        let action = choose_best_action(&available_actions);
        println!("🤖 Player {} action: {:?}", current_player, action);

        // Apply the action using real game logic
        if let Some(ref mut state) = game.state {
            state.apply_action(action);
        }

        // Log victory points every 10 turns or when something interesting happens
        let should_log_vp = turn_count % 10 == 0
            || matches!(
                action,
                Action::BuildSettlement { .. } | Action::BuildCity { .. }
            );

        if should_log_vp && turn_count != last_vp_log {
            println!("📊 Victory Points Status:");
            if let Some(ref state) = game.state {
                state.log_victory_points();
            }
            last_vp_log = turn_count;
        }

        turn_count += 1;

        // Small delay for readability
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    if turn_count >= MAX_TURNS {
        println!(
            "⏰ Simulation ended after {} turns (max reached)",
            MAX_TURNS
        );
        if let Some(ref state) = game.state {
            println!("📊 Final Victory Points:");
            state.log_victory_points();
        }
    } else {
        println!("✅ Simulation completed in {} turns", turn_count);
    }
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
        return action.clone();
    }

    if let Some(action) = actions
        .iter()
        .find(|a| matches!(a, Action::BuildCity { .. }))
    {
        return action.clone();
    }

    if let Some(action) = actions
        .iter()
        .find(|a| matches!(a, Action::BuildRoad { .. }))
    {
        return action.clone();
    }

    // Check for development card actions
    if let Some(action) = actions
        .iter()
        .find(|a| matches!(a, Action::BuyDevelopmentCard { .. }))
    {
        return action.clone();
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
        return action.clone();
    }

    // Check for trade actions
    if let Some(action) = actions
        .iter()
        .find(|a| matches!(a, Action::MaritimeTrade { .. }))
    {
        return action.clone();
    }

    // Check for robber actions
    if let Some(action) = actions
        .iter()
        .find(|a| matches!(a, Action::MoveRobber { .. }))
    {
        return action.clone();
    }

    // Check for discard actions
    if let Some(action) = actions.iter().find(|a| matches!(a, Action::Discard { .. })) {
        return action.clone();
    }

    // Check for roll action
    if let Some(action) = actions.iter().find(|a| matches!(a, Action::Roll { .. })) {
        return action.clone();
    }

    // Fall back to first action (likely EndTurn)
    actions[0].clone()
}

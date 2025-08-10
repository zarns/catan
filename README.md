# Catan

## front

- run `ng serve`
- to deploy:
  - from root dir
  - `ng build --configuration production` (builds with production settings)
  - `firebase login`
  - `firebase deploy`

## back

- from back dir
- run `shuttle run`
- deploy `shuttle deploy`

## simulation

The project includes a CLI tool for simulating Catan games between AI players:

- Build: `cargo build --bin simulate`
- Run: `cargo run --bin simulate -- [OPTIONS]`
- Run (optimized): `cargo run --release --bin simulate -- [OPTIONS]`

### Options

- `-p, --players <CONFIG>`: Player types (e.g., "MR" for MCTS vs Random)
  - `R`: Random player
  - `M`: MCTS player
- `-n, --num_games <N>`: Number of games to simulate (default: 1)
- `-v, --verbose`: Show detailed game logs

### Examples

- MCTS vs Random (single game): `cargo run --bin simulate -- -p MR`
- Random vs MCTS (10 games): `cargo run --bin simulate -- -p RM -n 10`
- Random vs Random with logs: `cargo run --bin simulate -- -p RR -v`

## Attribution

Inspired by [bcollazo's Catanatron](https://github.com/bcollazo/catanatron). Licensed under GPL-3.0.


# TODO
- Add play against catanatron buttons
- Cleanup
  - Evaluate frontend conversion serialized enums and remove unnecessary conversions — PARTIAL (typed action helpers)

- Build the greatest catan bot player of all time
- Implement DB functionality to track user count and game history
  - MCP integration
- Robber getting moved after first turn to wood 10 tile every time?
- stop highlighting clickable nodes for bot players on bot turns
- Resource distribution mismatch between sheep/wood??
- Can build settlements during other player's turn

## Cleanup Progress (tracking)

- Backend logging
  - Switched noisy `println!` to `log::debug!` in `back/src/state_vector.rs` (deterministic seating) — DONE
  - Converted test `println!` in `back/src/state/move_application.rs` to `log::debug!` — DONE

- Longest road logic
  - Fixed enemy-node handling; count terminal edge but do not traverse through — DONE
  - Removed unused `connected_set` parameter from DFS and simplified logic — DONE

- Determinism and component split
  - Settlement bisection deterministic and rules-correct — DONE

- Frontend typing and actions
  - `buildRoadAction` now takes tuple `[number, number]`; `buildSettlementAction`/`buildCityAction` use `number` — DONE
  - `WebsocketService.sendPlayerAction` parameter `gameId` marked unused (not sent in payload) — DONE


- Future safe-guards
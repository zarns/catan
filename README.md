# Catan

## front

- run `ng serve`
- to deploy:
  - `ng build --configuration production` (builds with production settings)
  - `firebase login`
  - `firebase deploy`

## back

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
- hide mobile buttons after clicking
- finish leftdrawer
  - ✅ Add logging component below player-state-box
  - clean up logging statements "You" "unknown"
- implement rightdrawer
  - backend mcts endpoint
- Actions toolbar
  - center actions buttons
  - remove card numbers
- GameBoard
  - implement player interactivity so user can play against bots
  - Hide actionsToolbar buttons when bots are playing
- Fix cargo test - failed tests
- Tokio Dependabot vuln
- Add play against catanatron buttons
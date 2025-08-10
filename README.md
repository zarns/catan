# Catan

## front

- run `ng serve`
- to deploy:
  - from front dir
  - `ng build --configuration production` (builds with production settings)
  - from root dir
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
- Build the greatest catan bot player of all time
- Implement DB functionality to track user count and game history
  - MCP integration
- Robber getting moved after first turn to wood 10 tile every time?

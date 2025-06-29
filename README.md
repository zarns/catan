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
- ✅ hide mobile buttons after clicking
- ✅ implement interactive node/edge clicking system
  - ✅ Parse current_playable_actions from backend
  - ✅ Replace offset node indicators with centered pulse animations
  - ✅ Enable proper click handling for settlements, cities, and roads
  - ✅ Connect frontend actions to backend via WebSocket
  - ✅ Dynamic actions toolbar based on available actions
- ✅ **COMPLETE: Interactive Human vs Bot Gameplay**
  - ✅ Backend exposes current_playable_actions in legacy [player_color, action_type, action_data] format
  - ✅ Frontend parses actions and enables proper node/edge clicking
  - ✅ Full action flow: Frontend click → Backend action processing → State update → WebSocket response
  - ✅ Bot player identification via bot_colors array
  - ✅ Initial build phase support for settlement/road placement
  - ✅ Actions toolbar dynamically enables/disables based on available actions
  - ✅ Proper human player turn detection and UI state management
  - ✅ implement player interactivity so user can play against bots
- ✅ **COMPLETE: ActionsToolbar React-Style Implementation**
  - ✅ Dynamic button filtering (only show enabled actions)
  - ✅ Player-specific roll detection (ROLL vs END button logic)
  - ✅ Fixed button layout (no horizontal shifting when buttons hide)
  - ✅ Proper material icons (dice for ROLL, skip for END)
  - ✅ Visibility-based hiding instead of conditional rendering
- implement rightdrawer
  - backend mcts endpoint
- GameBoard
  - Hide actionsToolbar buttons when bots are playing
- Fix cargo test - failed tests
- Add play against catanatron buttons
- Cleanup
  - remove unnecessary logging from frontend & backend
  - convert println statements to debug statements (or remove)
  - Evaluate frontend conversion serialized enums and remove unnecessary conversions
  - Attempt to remove node_coordinates.rs. Frontend should be responsible for this logic.
- Build the greatest catan bot player of all time
- Implement DB functionality to track user count and game history
  - MCP integration
- Robber getting moved after first turn to wood 10 tile every time?
- stop highlighting clickable nodes for bot players on bot turns
- Resource distribution mismatch between sheep/wood??
- Can build settlements during other player's turn
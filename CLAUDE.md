# Catan Game Project - AI Assistant Guide

## Project Overview

This is a full-stack implementation of the Settlers of Catan board game with the goal of building "the greatest Catan bot player of all time." The project features:

- **Real-time multiplayer gameplay** between humans and AI bots
- **Advanced AI simulation** using MCTS (Monte Carlo Tree Search) algorithms
- **Web-based interface** for interactive gameplay
- **CLI simulation tools** for AI vs AI testing and evaluation

## Architecture

### Tech Stack
- **Frontend**: Angular 19 with TypeScript, WebSocket communication
- **Backend**: Rust with Axum web framework, WebSocket server
- **AI Engine**: MCTS implementation with simulation framework
- **Deployment**: Firebase (frontend) + Shuttle (backend)

### Key Components
```
/front/          - Angular web application
/back/           - Rust backend server
/simulation/     - CLI tools for AI testing
README.md        - Setup and deployment instructions
HumanVBot.md     - Current implementation status and known issues
```

## Current Status (December 2024)

### ✅ Completed
- Core WebSocket architecture for real-time communication
- Basic Human vs Bot gameplay functionality
- Interactive board with click-to-build mechanics
- AI simulation CLI tools (MCTS vs Random players)
- Initial build phase (settlement/road placement)

### 🚨 Critical Issues (Priority: High)
1. **Node-tile coordinate mismatch**: Frontend shows node adjacent to sheep-12 tile, backend calculates node adjacent to sheep-10 tile
2. **UI/UX issues**: Button highlighting, robber movement timing, building flows

### 📋 Current Focus
Fixing the critical gameplay bugs to achieve fully functional Human vs Bot gameplay.

## Development Workflow

### Setup
```bash
# Backend (from /back directory)
shuttle run

# Frontend (from /front directory)  
ng serve

# AI Simulation (from root)
cargo run --bin simulate -- -p MR -n 10  # MCTS vs Random, 10 games
```

### Testing
```bash
# Backend tests
cargo test

# Frontend tests
ng test

# Linting
npm run format:fix
```

### Deployment
```bash
# Frontend
ng build --configuration production
firebase deploy

# Backend  
shuttle deploy
```

## AI Assistant Context

### Coding Conventions
- **Rust**: Standard rustfmt formatting, comprehensive error handling
- **TypeScript**: Angular style guide, strict type checking
- **WebSocket Protocol**: JSON messages with Rust enum format: `{Roll: {}}`, `{BuildSettlement: {node_id: 7}}`

### Key Files to Understand
- `/back/src/websocket.rs` - WebSocket message handling
- `/back/src/state/move_application.rs` - Game state management  
- `/front/src/app/services/websocket.service.ts` - Frontend WebSocket client
- `/front/src/app/components/game/game.component.ts` - Main game UI

### Current Bug Locations
1. **Multiple dice rolls**: `/back/src/state/move_application.rs` - `roll_dice()` method
2. **Bot turn interference**: `/front/src/app/components/game/game.component.ts` - action visibility logic
3. **UI issues**: `/front/src/app/components/actions-toolbar/` - button highlighting

### Development Priorities
1. **Fix game-breaking bugs first** (multiple rolls, bot interference)
2. **Maintain WebSocket architecture** (it's working well)
3. **Follow existing patterns** for consistency
4. **Test thoroughly** - gameplay bugs are subtle

### Testing Notes
- Use `cargo run --bin simulate` for AI testing
- Test Human vs Bot gameplay end-to-end
- Debug mode available (press 'D' key in browser)
- Backend logs show detailed game state transitions

## Project Goals

### Short-term (1-2 weeks)
- Fix critical gameplay bugs
- Achieve stable Human vs Bot gameplay
- Clean up UI/UX issues

### Long-term (Ongoing)
- Implement advanced AI strategies
- Add complete Catan rule set (development cards, maritime trading)
- Performance optimization and scaling
- Tournament/ranking system for AI evaluation

## Notes for AI Assistants

- **Don't break the WebSocket architecture** - it's the foundation that works
- **Focus on game logic bugs** rather than infrastructure changes
- **Test changes thoroughly** - gameplay bugs are hard to catch
- **Follow the existing patterns** in both Rust and TypeScript code
- **Check HumanVBot.md** for detailed analysis of current issues
- **Use the simulation CLI** to test AI improvements

## Attribution

Inspired by [bcollazo's Catanatron](https://github.com/bcollazo/catanatron). Licensed under GPL-3.0.
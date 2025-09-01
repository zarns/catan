# AlphaCatan AI – Development Roadmap v2.0

## Overview
Build an AlphaZero-style AI for simplified Catan (open hands, no trading) using Rust and Candle, deployable as 'Z' player in simulate.rs.

**Key Simplifications Working in Your Favor:**
- Open hands eliminate imperfect information challenges
- No trading reduces action space complexity by ~70%
- Fixed board topology suits CNN architectures well
- Deterministic state transitions (except dice) simplify MCTS

## Phase 0 – Infrastructure & Benchmarking ✓
- [x] Simulate matches between bots
- [x] Basic win-rate tracking
- [ ] **Enhanced Metrics Collection**
  - Track mean VP, variance, game length distributions
  - Per-phase timing analysis (setup vs main game); branching factor per phase
  - Action entropy and visit entropy per move (measure exploration vs exploitation)
  - Elo rating and SPRT gating between checkpoints
  - Deterministic seeding: record RNG seeds for dice, MCTS, and self-play per game
  - Persist metrics to CSV/Parquet with a versioned schema for longitudinal analysis
  - **Validation**: RandomPlayer baseline established (1000+ games)

### Current Progress (Week 1)
- Implemented a minimal single-threaded pure MCTS `AlphaZeroPlayer` ('Z')
- Integrated into `simulate.rs` and verified end-to-end play
- Initial result vs ValueFunction (VZ, n=1): AlphaZero loses; avg 185 turns; ~3.06s/game
- Added time-bounded search and short playout caps for responsiveness
- Added smart rollout policy (60/30/10) and deterministic seeding per move
- Introduced PUCT selection with heuristic priors (net-ready seam), and a tiny TT accumulator
- Increased default search budget: `ALPHAZERO_DEFAULT_SIMULATIONS = 400`, `MCTS_DECIDE_TIME_BUDGET_MS = 500`
- Added NN scaffolding under `back/src/players/nn/`:
  - `types.rs` (trait + structs), `noop_impl.rs`, `candle_impl.rs` (device auto-select),
  - `encoder.rs` with `encode_state_tensor()` stub returning [23,7,7] zeros (CHW) and legal action indexer
- Wired NN priors path in `zero.rs` to call `PolicyValueNet` (falls back to heuristic if empty)
 - Added training binary stub `back/src/bin/train.rs` and registered in Cargo; roadmap step for fleshing self-play & training loop

### Next Immediate Targets
- Reuse search tree between moves (re-rooting); bootstrap children with TT stats
- Add basic determinism tests and per-move time budget enforcement tests
- Implement real encoder features (player-relative channels, dice/robber/ports, roads/settlements/cities)
- Implement minimal ResNet in Candle with GroupNorm and dual heads
- Masked softmax for policy in fp32; map logits to legal actions via indexer
- Add loader for `back/models/latest.safetensors`; replace uniform priors with net outputs
 - Flesh out `train.rs`: self-play collection, replay buffer, losses (CE+0.5*MSE+L2), AdamW, checkpointing
  
## Phase 1 – MCTS Foundation (No NN)
**Goal:** Beat RandomPlayer >80%, approach GreedyPlayer performance

### Core Implementation in `alphaZero.rs`
- **PUCT-based MCTS** with configurable exploration constant (start c_puct ≈ 1.5–2.0)
- **Progressive Simulation** (MiniZero approach):
  - Start with 100 simulations/move
  - Scale to 800 by endgame
  - Formula: `sims = min(100 + turn * 20, 800)`
- **Tree Reuse**: Maintain subtree between moves
- **Virtual Loss**: -0.3 to -0.5 during parallel expansions

### Chance Handling (Dice & Robber)
- Prefer explicit chance nodes for dice rolls (expectimax-style with 11 outcomes weighted by probability) for stable backups
- Alternative: stochastic sampling per simulation (lower branching, higher variance). Choose one and keep consistent with TT hashing
- Ensure robber-related randomness is represented deterministically in the node/state (e.g., via explicit actions for robber movement and victim selection)

### Critical Optimizations
- **Transposition Table** using Zobrist hashing:
  - Hash: settlements, cities, roads, robber position, bank stocks, largest army/longest road ownership, dev deck composition/remaining, per-player piece stocks, and any flags needed for setup phase
  - Cache size: 100K positions initially
- **Progressive Widening** for build phase:
  - Tie widening to visitation and empirical branching factor rather than a fixed formula
  - Seed early children using domain priors (production value, connectivity, port leverage)
- **Smart Rollout Policy** (not random):
  - 60% exploit: choose highest production value
  - 30% explore: uniform random
  - 10% strategic: longest road/largest army progress
  - Remove rollouts entirely once a value head is integrated
- **Concurrency & Node Storage**:
  - Use an arena allocator with stable node indices; hot fields updated via atomics
  - Keep per-node hot data in a structure-of-arrays layout for cache locality
  - Shard the TT or use a lock-free map to reduce contention

### Leaf Evaluation (No NN Yet)
```
value = w1 * vp_ratio + 
        w2 * production_score +
        w3 * resource_diversity +
        w4 * port_access +
        w5 * longest_road_potential +
        w6 * dev_card_value
```
Initial weights: [0.4, 0.2, 0.15, 0.1, 0.1, 0.05]

**Validation Milestone**: 
- Win rate vs RandomPlayer: >80% (1000 games)
- Win rate vs GreedyPlayer: >40% (500 games)
- Average game tree size: 5000-10000 nodes

### Status
- Minimal MCTS online (single-thread); no TT/tree reuse yet
- Using stochastic playouts with caps; selection via average value + exploration

## Phase 2 – State Encoding & Action Masking

### State Tensor Design (7×7×C or Graph)
**Recommended: Start with CNN-friendly 7×7 grid** (switch to GNN later if needed)

#### Channel Layout (23 channels total):
```rust
// Spatial channels (per hex): 0-6
0: Desert/Water mask
1-6: Resource types (one-hot)

// Dice channels: 7-8  
7: Dice number (normalized /12)
8: Production probability

// Player pieces (4 players × 3): 9-20
9-12: Player settlements
13-16: Player cities  
17-20: Player roads (edge encoding)

// Game state: 21-22
21: Robber position
22: Harbor types
```

#### Player-relative encoding & symmetries
- Encode the current player's pieces/features in fixed channels; rotate other players by seat order
- Precompute rotational/reflectional symmetry maps for hexes, nodes, and edges; use symmetry augmentation during training

#### Additional per-state features (non-spatial)
- Bank stocks (roads/settlements/cities remaining per player; resource bank optional if used)
- Dev deck composition remaining; flags for largest army/longest road and thresholds
- Turn number and phase indicators (setup vs main game)

### Action Space Design
**Hierarchical Action Decomposition** (reduces 1000+ → ~200 actions):
```rust
enum ActionType {
    BuildSettlement(NodeId),  // ~54 nodes
    BuildCity(NodeId),        // ~54 nodes  
    BuildRoad(EdgeId),        // ~72 edges
    PlayDevCard(DevType),     // ~5 types
    MoveRobber(HexId),       // 19 hexes
    BuyDevelopmentCard,       // purchase action (consumes resources)
    ChooseRobberVictim(PlayerId), // when applicable
    DiscardOnSeven { counts: [u8; 5] }, // if using discard rules
    SetupPlacement { settlement: NodeId, road: EdgeId }, // initial phase
    EndTurn,                  // 1 action
}
```

### Action Masking Implementation
- Generate legal move mask alongside state encoding
- **Off-Policy Invalid Action Masking (Off-PIAM)** approach
- Mask invalid actions with a numerically stable masked softmax (add large negative to invalid logits)
- Cache mask computation (changes infrequently)

**Validation**: 
- State encoding/decoding round-trip test
- Legal action generation matches game engine 100%
- Encoding performance: <1ms per state

## Phase 3 – Neural Network Architecture

### Candle-based ResNet Implementation
```rust
// In alphaZero.rs
struct AlphaZeroNet {
    // Initial convolution
    conv_block: ConvBlock,     // 3×3, 64 filters
    
    // Residual tower (start small)
    res_blocks: Vec<ResBlock>, // 6 blocks, 64 filters
    
    // Dual heads
    policy_head: PolicyHead,   // Conv → Dense(256) → Dense(actions)
    value_head: ValueHead,     // Conv → Dense(256) → Dense(1) → tanh
}
```

### Key Candle Considerations
- Use `candle_nn` for layer abstractions
- Implement custom ResBlock with skip connections
- Prefer GroupNorm or LayerNorm for stability with small/self-play batches
- **Critical**: Ensure deterministic forward pass for debugging

### Training Configuration
- Optimizer: AdamW (lr=0.001, weight_decay=0.0001)
- Batch size: 32 (memory permitting)
- Loss: `0.5 * MSE(value) + CrossEntropy(policy) + 0.0001 * L2`
- **Mixed precision**: Use f16 for forward pass, f32 for gradients
- Gradient clipping (global norm 1.0–2.0)
- Cosine learning-rate decay with warmup
- EMA of parameters for evaluation checkpoints
- Compute masking/log-softmax for policies in fp32 for numerical safety

**Validation**:
- Network processes batch of 32 states in <100ms
- Policy head outputs sum to 1.0 (after softmax)
- Value head outputs in [-1, 1] range

## Phase 4 – Self-Play Training Loop

### AlphaZero Training Pipeline
```rust
// Main training loop in alphaZero.rs
loop {
    // 1. Self-play generation (parallel)
    let games = parallel_self_play(
        network: &current_net,
        num_games: 100,
        mcts_sims: 400,
        temperature: 1.0 → 0.1 (after move 30)
    );
    
    // 2. Add to replay buffer (circular, 100K positions)
    replay_buffer.extend(games);
    
    // 3. Sample and train
    for _ in 0..1000 {
        let batch = replay_buffer.sample(32);
        train_step(&mut network, batch);
    }
    
    // 4. Evaluate and checkpoint
    if iteration % 10 == 0 {
        evaluate_and_save(&network);
    }
}
```

### Critical Implementation Details
- **Batched inference**: inference worker batches leaf evaluations across threads (max batch size, short flush timeout)
- **Dirichlet noise** at root: scale α with |legal_actions| (e.g., α ≈ 10 / |A|, clipped to [0.03, 0.5]); ε=0.25
- **Temperature schedule**: 1.0 for exploration → 0.1 for exploitation
- **Value target**: Use game outcome (not Q-values initially)
- **Progressive simulation increase**: Start 100 → 800 over 50 iterations
- **Replay buffer**: shard to disk, maintain an in-memory recency window; sample with recency bias (e.g., 0.7 recent / 0.3 uniform)
- **Evaluation & gating**: fixed-seed boards, opponent pool (Random, Greedy, baseline MCTS, last N checkpoints); Elo + SPRT acceptance

### Gumbel Improvements (Phase 4.5)
- Replace standard PUCT with Gumbel AlphaZero
- Sample actions without replacement using Gumbel-Max
- **Proven to improve with limited simulations** (your use case)
- Apply mask to logits before Gumbel sampling; consider enabling once vanilla MCTS is stable

**Validation Milestones**:
- Iteration 10: Beats RandomPlayer >95%
- Iteration 25: Beats GreedyPlayer >60%
- Iteration 50: Beats pure MCTS >70%
- Network learns opening preferences (high-production spots)

## Phase 5 – Optimization & Polish

### Performance Optimizations
- **Batched inference**: Queue leaf evaluations, process together
- **State caching**: LRU cache for repeated positions
- **Candle-specific**:
  - Pre-allocate tensors where possible
  - Use `no_grad()` context for inference
  - Profile with `cargo flamegraph`
- **Node layout & concurrency**:
  - Structure-of-arrays for hot fields (visits, value_sum, prior)
  - Contiguous child ranges per node for cache-friendly traversal
  - Thread pool with work-stealing; sharded locks or lock-free structures to avoid global contention
- **Device management**:
  - Automatic device selection; adapt batch size under memory pressure; CPU fallback path

### Integration Requirements
- Implement `Player` trait for AlphaZeroPlayer
- Add 'Z' designation in simulate.rs
- Configuration via TOML/JSON:
  ```toml
  [alphazero]
  model_path = "models/latest.safetensors"
  mcts_simulations = 800
  temperature = 0.1
  use_gpu = true
  ```
  - Place implementations under `back/src/players/alphazero_{mcts,net}.rs` and `alphazero.rs`
  - Centralize hyperparameters and expose via CLI and config; keep defaults minimal and overridable

### Testing & Validation Suite
```rust
#[cfg(test)]
mod tests {
    // Unit tests for MCTS operations
    test_mcts_expansion()
    test_ucb_calculation()
    test_backup_propagation()
    
    // Integration tests
    test_full_game_play()
    test_deterministic_play()
    test_time_constraints()
}
```
Additional testing:
- Property tests for state encoding/decoding and legality parity (e.g., with `proptest`)
- Symmetry tests across rotations/reflections
- Determinism tests under fixed seeds (forward pass and MCTS stats)
- Time-boxed search stability under wall-clock deadlines

## Expected Pain Points & Solutions

### 1. **Candle Learning Curve**
- **Issue**: Limited RL examples in Candle ecosystem
- **Solution**: Reference Candle's MNIST/ResNet examples, implement layers incrementally
- **Fallback**: Port minimal PyTorch model if needed

### 2. **MCTS Performance**
- **Issue**: Rust MCTS might be slower than expected initially
- **Solution**: Profile aggressively; batch inference and use a work-stealing thread pool
- **Key**: Avoid excessive cloning; use an arena + atomics for thread-safe nodes and sharded/lock-free TT

### 3. **GPU Memory Management**
- **Issue**: Candle's GPU memory handling differs from PyTorch
- **Solution**: Explicit tensor cleanup, batch size tuning
- **Monitor**: GPU memory usage during self-play

### 4. **Training Convergence**
- **Issue**: Network may overfit to self-play
- **Solution**: Maintain opponent pool, add noise, use dropout (0.3)
- **Track**: Policy entropy over time (shouldn't collapse)

### 5. **State Representation Bugs**
- **Issue**: Mismatch between game state and tensor encoding
- **Solution**: Extensive unit tests, visual debugging tools
- **Critical**: Canonical ordering for roads/edges

## Success Metrics

### Week 1-2: MCTS Baseline
- [ ] Pure MCTS beats RandomPlayer >80%
- [ ] Tree search handles 10K+ nodes efficiently
- [ ] Rollout policy shows strategic improvement

### Week 3-4: Neural Network Integration  
- [ ] Network trains without NaN/Inf issues
- [ ] Self-play generates diverse games
- [ ] GPU utilization >60% during training

### Week 5-6: Convergence & Optimization
- [ ] AlphaZero beats all scripted players
- [ ] Inference time <100ms per move
- [ ] Model size <10MB (quantized)

### Final Validation
- [ ] 100-game match vs best scripted player: >70% win rate
- [ ] Human playtesting: "feels intelligent"
- [ ] Deployment: Works seamlessly with 'Z' flag

## Next Steps After MVP

1. **Expand to full Catan rules** (trading, hidden hands)
2. **Implement GNN architecture** for board flexibility
3. **Multi-agent training** with different play styles
4. **Transfer learning** from 4-player to 3/5/6 players
5. **Explainability**: Attention visualization, move explanations

## Key Research References
- MiniZero progressive simulation strategies
- Gumbel AlphaZero for limited budget MCTS
- Off-PIAM for action masking
- Candle ResNet examples for architecture patterns

---
*Remember: Start simple, validate often, profile everything. The simplified rules work in your favor - embrace them for faster iteration!*
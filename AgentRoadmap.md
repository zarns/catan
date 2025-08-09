# Superhuman Catan AI – Development Roadmap

## Phase 0 – Infrastructure
- [x] Simulate matches between bots
- [ ] Ensure `simulate.rs` supports large batch runs
- [ ] Add win-rate, mean VP, variance tracking

## Phase 1 – Search-Based Baseline
**Goal:** Beat `RandomPlayer` without ML  
- Implement Alpha-Beta or MCTS (no NN)
- Use static eval function:
  - Victory Points (VP)
  - Resource count/variety
  - Production probability
- Add action masking to reduce branching factor

## Phase 2 – State Encoding
**Approach:** 7×7×C CNN-friendly tensor (Graph-Tensor Encoding)  
- Axial hex → 7×7 grid mapping
- Channels (example):
  1. Tile type
  2. Dice value (normalized)
  3. Robber position
  4–11. Settlements/cities per player
  12–15. Roads per player
  16. Harbors
  17. Dev cards (open hand)
  18. Trade ratios / markers
- Batch encoding for simulation speed

## Phase 3 – CNN Policy-Value Network
**Architecture:** ResNet-6, 64 channels, dual heads  
- **Policy head:** 289 actions (hierarchical or flat)
- **Value head:** Win probability
- Integrate with bot for decision-making

## Phase 4 – Self-Play + Training
**AlphaZero-style loop:**
1. MCTS guided by NN policy/value
2. Store `(state, policy, value)` triples
3. Train NN with:
   - Policy loss (cross-entropy)
   - Value loss (MSE)
   - L2 regularization
4. Iterate → stronger agent

## Phase 5 – Optimization & Extensions
- GPU acceleration with Candle
- Memory-efficient batching (contiguous, in-place ops)
- Parallel self-play workers
- Add full rules: hidden hands, discard choices
- Research GNN variant for board-size flexibility

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

## AlphaCatan Roadmap (Detailed)

This section refines the plan with concrete milestones and engineering details tailored for a performant Rust backend, efficient NN integration via Candle, and robust benchmarking.

### Phase 0 — Infra and Bench (strength prerequisites)
- Determinism: fixed seeds, canonical edge IDs (done), reproducible builds.
- Simulator UX: batch mode, CSV/JSON metrics (win-rate, mean VP, stdev, turns).
- Eval harness: head-to-head matrix (Random, Greedy, AlphaBeta, MCTS), ELO, significance tests.
- CI gates: cargo fmt/clippy/test; nightly “league” run with trend charts.

### Phase 1 — MCTS Baseline (drop Alpha-Beta)
- Core: PUCT MCTS with tree reuse across moves; limit sims/time.
- Search quality:
  - Progressive widening for large branching (build phases).
  - Transposition table (Zobrist hash over roads/buildings/robber/dev state).
  - Heuristic rollout policy (not random): prioritize high-prod nodes, road connectivity, port usage.
  - Leaf eval (no NN): weighted features − effective production, resource diversity, distance-to-next-build, longest-road/army potential, 7-risk, dev leverage.
  - Optional RAVE/AMAF and virtual loss for parallel sims.
- Hidden info: determinization of unseen hands/dev deck; average over K samples.
- Action masking: legal-only, and simple trade abstraction before full trading.
- Measurable goal: >80% vs Random over 1k games; parity or better vs Greedy.

### Phase 2 — State Encoding (keep flexible)
- Prefer graph-native encoding (roads/nodes/tiles as typed graph) for future GNN.
- If keeping 7×7 grid, add channels for: Tile type, dice number, robber, ports, per-player settlements/cities/roads, bank rates/hand counts (public), phase flags.
- Batched encoders; masks for legal actions generated alongside tensors.

### Phase 3 — Policy-Value Network (right-size and outputs)
- Start minimal (ResNet-6 or Tiny GNN); upgrade later.
- Outputs:
  - Policy: hierarchical, masked heads — Node actions (settlement/city), Edge actions (road), Dev plays, Trade templates.
  - Value: win prob from current player perspective.
- Losses: CE(policy) + MSE(value) + L2; temperature/Dirichlet for exploration.
- Calibrate value head (ECE/Platt).

### Phase 4 — AlphaZero-style Loop
- Self-play with NN-guided MCTS; store (state, policy, value).
- Replay buffer (e.g., 1–5M samples), prioritized by novelty/outcome.
- Train (AdamW, cosine LR, mixed precision), periodic evaluations with gating.
- Model registry and rollback.

### Phase 5 — Performance & Completeness
- Parallel self-play workers; shared inference service (batching).
- Quantization for inference; memory pools for state clones.
- Expand rules: discard decisions, domestic trade negotiation (start with templates).
- Opponent modeling (later).

### Phase 6 — Productization
- Integrate difficulty knobs (sims/time, heuristics weight).
- Telemetry: per-decision stats (depth, visits, best-k).
- Safety: timeouts, graceful degradation to heuristic when budget exceeded.

### Immediate next steps (low-risk, high-impact)
- Wire MctsPlayer into: Bot decision path; Analyze endpoint (replace rollouts).
- Add transposition table + progressive widening.
- Implement leaf evaluation + simple rollout policy.
- Enhance simulator: batch runs + metrics export.
- Add request_id already in place to evaluate analyze at scale.

### Milestones
- M1: MCTS baseline beats Random >80%.
- M2: With heuristics/TT/widening, beats Greedy by >60%.
- M3: NN policy/value integrated; strong vs scripted baselines; human evals.
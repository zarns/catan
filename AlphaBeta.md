## AlphaBeta (Rust) Improvement Roadmap

This document summarizes concrete changes to evolve `back/src/players/minimax.rs` toward the capabilities of the Python Catanatron implementation in `catanatron/players/minimax.py` and its helpers.

### 1) Correct turn semantics (Max/Min instead of unconditional negamax)
- Problem: Rust negates at every non-roll action. In Catan, many consecutive actions belong to the same player within a single turn (build, buy, play dev, trade, etc.). Negating there undervalues multi-action turns.
- Plan:
  - Determine `is_maximizing = state.get_current_color() == my_color` at each node.
  - Use explicit max/min branches; drop unconditional sign flip. Only the player switch should cause max↔min role change.
  - Optionally add a `SameTurnAlphaBeta` mode that searches only while `current_color == my_color` (mirrors Python’s `SameTurnAlphaBetaPlayer`).

### 2) Generalize chance nodes (expectiminimax "spectrum" per action)
- Python expands outcomes for any stochastic action via `expand_spectrum`:
  - Deterministic: END_TURN, BUILD_{SETTLEMENT/ROAD/CITY}, PLAY_{KNIGHT/YOP/ROAD_BUILDING/MONOPOLY}, MARITIME_TRADE, DISCARD
  - BUY_DEVELOPMENT_CARD: branch over possible card identities weighted by inferred deck composition
  - ROLL: branch over 2–12 using `number_probability`
  - MOVE_ROBBER: if a victim exists, branch over stolen resource types uniformly (or by victim hand composition)
- Plan (Rust):
  - Introduce `expand_outcomes(state: &State, action: Action) -> Vec<(State, f64)>`.
  - Compute action value as `Σ p_i * minimax(next_i)`.
  - Start by wiring existing dice logic; add dev-card purchases and robber steals next.

### 3) Domain-aware action pruning before ordering
- Python’s `list_prunned_actions` removes low-impact or dominated actions prior to search:
  - Initial placements: prune 1-tile settlement spots
  - Maritime trade: when a 3:1 port is available, prune 4:1 trades of the same resource
  - Robber: keep only the most impactful tile placement (max enemy production impact)
- Plan (Rust):
  - Add `prune_actions(state, actions) -> Vec<Action>` implementing the above. Keep it conservative to preserve correctness.
  - Apply pruning, then ordering, then beam.

### 4) Move ordering and beam usage
- Keep the beam, but order after pruning. Improve ordering with domain signals:
  - Prioritize actions that increase visible VPs, city/settlement builds, strong robber moves, dev plays that unlock build sequences, etc.
  - Use recent-best/root-best (killer move) hints when available.
- Maintain a configurable `MAX_ORDERED_ACTIONS` and allow disabling the beam when the branching factor is already small.

### 5) Time management and iterative deepening
- Python enforces a deadline to keep move latency predictable.
- Plan (Rust):
  - Add a `deadline: Instant` to search; stop when reached.
  - Implement iterative deepening (1..D) and return best-so-far when time expires.
  - Optionally use aspiration windows around the previous iteration’s root value for faster convergence.

### 6) Evaluation: port the richer Catanatron value function
- Python’s `value.py` provides a sophisticated, pluggable evaluator with two presets:
  - `base_fn(DEFAULT_WEIGHTS)` and `contender_fn(CONTENDER_WEIGHTS)`
  - Features include:
    - public_vps (huge), production and enemy_production (considering robber), num_tiles covered by owned nodes, reachable_production at 0 and 1 road distance, buildable_nodes, longest_road, hand_synergy (distance to city/settlement), hand_resources count, discard_penalty (>7), hand_devs, army_size (played knights)
- Plan (Rust):
  - Extract an `Evaluator` trait with a default implementation mirroring `base_fn`.
  - Make weights configurable and allow a “contender” preset.
  - Remove hardcoded `num_players = 4`; derive from state.
  - Add the necessary feature helpers on `State` to compute production, reachability (0/1 roads initially), bank pressure (discard risk), dev/army metrics, and tile coverage.

### 7) Transposition table (TT)
- Add Zobrist hashing for `State`.
- Store entries keyed by `(hash, depth)` with value type {Exact, Alpha, Beta} and score.
- Use TT for cutoff ordering and to avoid re-searching identical substates.
- Integrate tightly with iterative deepening.

### 8) Root policy: tie-breaking and exploration
- Break ties randomly among equal-best root actions.
- Optional epsilon-greedy exploration (for self-play/tuning only; off in production).

### 9) Testing and benchmarking
- Add deterministic scenarios and regression tests for:
  - Same-turn sequences (no sign flip)
  - Spectrum expansion correctness (dice, dev buys, robber steals)
  - Pruning soundness (no loss of obviously better actions)
  - Evaluator parity tests against Python on mirrored positions (within tolerance)
- Benchmark decision time vs. depth/beam and Elo vs. current bot.

### 10) Optional search refinements
- Quiescence search: extend at “tactical volatility” nodes (e.g., immediate builds/plays) to reduce horizon effects.
- History heuristic / killer moves to improve ordering beyond static heuristics.
- Null-move pruning: likely risky with chance nodes; consider only after extensive testing.

---

## Notes from catanatron Python code

- File: `catanatron/players/minimax.py`
  - Uses explicit max/min based on `game.state.current_color() == self.color`.
  - Expectiminimax via `expand_spectrum(game, actions)` → action → [(next_game, p)] and computes expected values.
  - Supports pruning through `get_actions()` → `list_prunned_actions(game)`.
  - Uses deadline (`MAX_SEARCH_TIME_SECS`) and returns best-so-far.
  - Variants: `SameTurnAlphaBetaPlayer` restricts search to same-player turn.

- File: `catanatron/players/tree_search_utils.py`
  - `expand_spectrum` covers deterministic actions, `BUY_DEVELOPMENT_CARD`, `ROLL` (2–12 with `number_probability`), and `MOVE_ROBBER` (steal distribution).
  - `list_prunned_actions` prunes: initial 1-tile settlements, suboptimal maritime trades, and compresses robber moves to the most impactful tile using production features.

- File: `catanatron/players/value.py`
  - `base_fn` and `contender_fn` with comprehensive features and tuned weights (`DEFAULT_WEIGHTS`, `CONTENDER_WEIGHTS`).
  - Utility `value_production` mixes production sum and variety bonus.
  - `get_value_fn` is pluggable; also supports injecting a custom function.

- File: `catanatron/features.py`
  - Feature extractors for production (with/without robber), reachability (0/1 road distance), hand state, graph/tiles/ports, longest road, expansion, port distances, and game/bank features.

---

## Implementation order (recommended)
1. Turn semantics: replace negamax with explicit max/min; add `SameTurn` option.
2. Time budget + iterative deepening; keep current evaluator.
3. Outcome spectrum: extend dice to dev buys and robber steals.
4. Pruning pass before ordering; then beam-ordering.
5. Pluggable evaluator and weight sets (port subset of features first: production, enemy production, hand resources, hand synergy, reachable_production_0/1, longest road, army).
6. Transposition table + improved ordering (killer/history).
7. Broaden evaluator features toward parity with Python.

---

## Progress Log

- [x] Switched `minimax.rs` from unconditional negamax to explicit max/min based on `state.get_current_color() == my_color`. This prevents sign flips during same-turn action chains and aligns with Catan’s turn structure.
- [x] Introduced `prune_actions` hook (currently passthrough) and applied it at root and internal nodes before ordering/beam. This is the insertion point for domain-aware pruning from Python (`list_prunned_actions`).
- [x] Added `evaluate_action_with_chance` helper and routed dice rolls through the existing expectation logic; non-chance actions recurse without sign flip. This prepares for a general spectrum expansion per action.
- [x] Added `players/value.rs` with `ValueFunctionPlayer` and tunable `ValueWeights`, plus epsilon-greedy. The evaluator mirrors Python’s base_fn partially using available `State` accessors (public VPs, effective production, basic hand features, buildable nodes, tile coverage, army size). Exported via `players/mod.rs`.
- [x] Integrated `ValueFunctionPlayer` into `simulate.rs` as 'V'/'v'. Verified dominance vs Random in user run.
- [x] AlphaBeta: added iterative deepening with a short time budget (`max_time_ms`, default 50ms) and wired a `deadline` through recursion, including `roll_expectation`. Added an initial conservative `prune_actions` rule (drop 1-tile initial settlements).
- [x] Strengthened AlphaBeta evaluator by reusing the ValueFunction-style features (effective production, enemy production, hand synergy, hand size/penalty, devs, army size, buildable nodes, tile coverage). This should close part of the gap to the Python agent pending chance-node spectrum and richer pruning.
- [x] Added domain-aware pruning: when a 3:1 port is owned, 4:1 maritime trades are pruned. Retained initial-settlement 1-tile pruning.
- [x] Implemented probabilistic expectation for robber steals: when moving the robber with a victim, branch over stolen resource based on victim hand composition.
- [x] Improved move ordering with a shallow (0-ply) evaluator mixed with static action scores to enhance pruning efficiency.
- [x] Added aspiration windows around the best-so-far at root during iterative deepening to accelerate cutoffs; falls back to full window on fail-high/low.
- [x] Introduced a simple transposition table keyed by `(hash64, depth)` with alpha/beta/exact flags using a stable FNV-1a hash of the state vector. Integrated into search for early cutoffs and reuse.
- [x] Added expected-value handling for `BuyDevelopmentCard` using remaining bank composition to weight outcomes (approximate; uses state bank counts).
- [x] Root policy improvements: optional epsilon-greedy exploration and random tie-breaking among near-equal best root actions.
- [x] Generalize chance expansion beyond dice (dev buys, robber steals).
- [x] Add time budget + iterative deepening with best-so-far tracking.
- [x] Implement domain-aware pruning rules (initial placement, maritime trade, robber compression).
- [x] Make evaluator pluggable with weight presets and additional features.
- [x] Add transposition table and improved move ordering heuristics.



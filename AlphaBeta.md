Next Steps: Strengthen AlphaBetaPlayer (Rust)

Objective
Make `AlphaBetaPlayer` stronger and more stable by implementing targeted, engine-grade improvements without changing overall architecture.

Scope (Only real issues/improvements)
- Transposition Table collisions: add verification to avoid poisoned cutoffs.
- Futility pruning symmetry: apply conservative futility at minimizing nodes.
- Late Move Reductions (LMR): replace fixed thresholds with depth/index–aware scaling and re-search when needed.
- Depth-1 move cap: always keep all tactical moves; cap only quiets.
- History/killer hygiene: decay history less frequently; reset killers at root.
- Optional polish: small readability guard for PVS window.

Implementation Tasks (in order)

1) Transposition Table collision safety
- Change TT key/entry types:
  - `type TtKey = (u64, u64);`
  - `type TtEntry = (f64, i8, Option<Action>, i32);`
  - `type TtMap = HashMap<TtKey, TtEntry>;`
- Compute two independent hashes:
  - Keep `hash_a = state.compute_hash64()`.
  - Add a second hash `hash_b` via a different seed/table (e.g., `compute_hash64_alt()` or construct a second Zobrist set seeded once at init).
- Use `(hash_a, hash_b)` as the TT key for insert/lookup.
- If adding a second hash is not feasible immediately, store a lightweight verifier alongside the entry (e.g., `(side_to_move, robber_tile_id, remaining_dev_counts_checksum)`) and compare on probe; treat mismatches as TT misses.

2) Symmetric futility pruning
- Remove redundant `is_maximizing &&` in the maximizing futility check.
- Add a mirrored guard in the minimizing branch for quiet moves at shallow depths:
  - Compute `eval = evaluate_relative(state, my_color)`.
  - If `depth == 1 && is_quiet && eval - 100.0 > beta` then `continue;`.
  - If `depth == 2 && is_quiet && eval - 200.0 > beta` then `continue;`.
- Keep futility off for tactical moves.

3) Depth/index–aware LMR
- Replace `get_lmr_reduction` with a smooth, conservative formula that excludes PV and tactical moves and never reduces the first few moves:
```rust
fn get_lmr_reduction(&self, depth: i32, move_index: usize, is_quiet: bool) -> i32 {
  if !is_quiet || depth < 3 || move_index < 3 { return 0; }
  let d = depth as f64;
  let m = ((move_index + 1).max(1) as f64).ln();
  let r = (0.8 * d.ln() * m).floor() as i32;
  r.clamp(0, 2).min(depth - 1)
}
```
- In both max/min loops: apply reduced depth for null-window probes; if the value is inside the window, re-search at full depth/window.

4) Depth-1 move-count pruning refinement
- At `depth == 1`:
  - Partition ordered moves into `tactical` and `quiet` using `is_tactical_move`.
  - Keep all `tactical`.
  - Fill up to the cap (e.g., 10 total) with `quiet` in order.
  - This guarantees no tactical move is lost at the frontier.

5) History and killer maintenance
- History decay: apply decay every N root decisions (e.g., every 10 calls to `decide`) instead of after each decision. Use `/= 2` when decaying.
- Killer moves: clear the `killer_moves` map at the start of each `decide` call.

6) PVS window readability (optional)
- Keep the existing guard. Optionally add `if beta - alpha <= PVS_EPS { use_pvs = false; }` for clarity.

Testing & Validation
- Unit tests:
  - LMR: verify no reductions for early or tactical/PV moves; verify reductions increase (0→1→2) with depth/move index.
  - Futility: confirm quiet moves can be skipped at depth ≤ 2 and not at deeper levels; ensure tactical moves are never skipped.
  - TT verification: craft synthetic collision test (mock verifier) to ensure mismatches are treated as misses.
- Integration checks:
  - Run short tournaments vs `ValueFunctionPlayer` and prior AlphaBeta to confirm strength and stability.
  - Track average search depth and node count; expect deeper effective depth at same time budget.

Future Strength Areas (post above)
- Evaluation upgrades: improve production/reachability terms; add dynamic scarcity and longest-road reach.
- Hybrid search: use AlphaBeta tactically and MCTS for strategic planning phases.
- Learning: train a value network from self-play to replace/augment `evaluate_state`.



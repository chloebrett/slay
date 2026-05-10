# Plan: Multi-column Map Topology (Subtask 4)

**Status**: Active

## Goal

Replace the single-column map skeleton with the real Slay the Spire path-tracing algorithm:
7 columns × 15 traversal floors, 6 paths, no crossing edges, weighted movement.

## Algorithm (from reverse-engineering post)

1. Trace 6 paths through a 7×15 grid, floor 0 → floor 14.
   Each step picks from {col−1, col, col+1} ∩ [0,6] (weighted straight:side = 2:1 by default).
   Crossed edges are forbidden; first two paths must start from distinct columns.
2. Trim edges between floor 0 and floor 1 that converge on the same floor-1 node
   (keeps only the one from the smallest floor-0 column; the rest disappear from floor 0).
3. Build MapGraph: occupied columns per floor (sorted), edges derived from paths.
4. Assign node types with the existing bucket system, scaled to actual node count,
   with adjacency constraints checking all parents + same-floor siblings.

## Configurable constants (`MapConfig`)

```
num_paths:      6   paths traced
num_cols:       7   columns in the grid
num_floors:     15  traversal floors (boss is floor 15, appended separately)
straight_weight:2   relative weight for staying in the same column
side_weight:    1   relative weight for moving left or right
```

---

## Steps

### Step 1: `MapConfig` struct + `crosses_edge` + signature migration

**Acceptance criteria**:
- `MapConfig` is a public struct with the five fields above and a `Default` impl.
- `crosses_edge(a, b, c, d) -> bool` is a private pure function.
- `generate_map` takes `(config: &MapConfig, rng)` instead of just `(rng)`.
- All existing callers compile and all existing tests pass (map behaviour unchanged).

**RED**: Tests for `crosses_edge` + compile error from updated `generate_map` signature.  
**GREEN**: Add `MapConfig`, add `crosses_edge`, update signature + callers.  
**MUTATE / KILL / REFACTOR**: Standard cycle.  
**Done when**: All tests green, no clippy warnings.

---

### Step 2: `choose_next_col` + `generate_raw_paths`

**Acceptance criteria**:
- `choose_next_col(col, floor_edges, config, rng) -> usize` returns a column in [0, num_cols−1]
  that does not create a crossing edge with any existing edge in `floor_edges`.
  Candidates are weighted: straight appears `straight_weight` times, each side appears `side_weight` times.
  Falls back to straight if all candidates cross.
- `generate_raw_paths(config, rng) -> Vec<Vec<usize>>` returns exactly `num_paths` paths,
  each with `num_floors` entries, all in [0, num_cols−1], first two with distinct starting columns,
  and no crossing edges anywhere.

**RED**: Unit tests for `choose_next_col`; property tests for `generate_raw_paths` across 20 seeds.  
**GREEN**: Implement both functions.  
**MUTATE / KILL / REFACTOR**: Standard cycle.  
**Done when**: All tests green.

---

### Step 3: `trim_floor0_merges` + multi-column `generate_map`

**Acceptance criteria**:
- `trim_floor0_merges(paths) -> Vec<(usize, usize)>` returns the floor-0→floor-1 edge set
  with at most one floor-0 source per floor-1 destination (smallest source wins).
- `generate_map` produces a 16-floor graph (15 traversal + boss) where:
  - floor 0 has ≥ 1 node (usually 2–6) and all are easy Combat
  - floor 8 has all-Treasure nodes
  - floor 14 has all-RestSite nodes
  - floor 15 has exactly one Boss node
  - every edge references a valid column index in the target floor
  - no Elite before floor index 5, no Rest before floor index 5
  - no Rest at floor index 13
  - no two nodes connected by an edge have the same special type (Rest/Shop/Elite)
  - no two siblings (same parent) have the same special type
- `full_run_reaches_victory` integration test passes with NoOpRng.

**RED**: Tests for `trim_floor0_merges` + graph property tests across 20 seeds.  
**GREEN**: Implement `trim_floor0_merges`, rewrite `generate_map` with multi-column logic,
  scale bucket to actual node count, update `bucket_kind_valid` for multi-parent check.  
**MUTATE / KILL / REFACTOR**: Standard cycle.  
**Done when**: All 1400+ tests green, integration test passes.

---

## Callsite inventory (things to update when signature changes)

- `crates/slay-core/src/run.rs` line ~208: `new_run` → `generate_map(rng)` → `generate_map(&MapConfig::default(), rng)`
- `crates/slay-core/src/run.rs` `test_graph()` helper
- `crates/slay-core/src/run.rs` constraint property tests (pass `&MapConfig::default()`)
- `crates/slay-core/src/lib.rs`: add `MapConfig` to `pub use run::{...}`
- `crates/slay-tui/tests/integration.rs`: if `generate_map` used directly (check)

---
*Delete this file when the plan is complete.*

# Plan: Branching Graph Map

**Branch**: main
**Status**: Active

## Goal

Replace the static `MAP_NODES` slice with a pre-generated DAG where the player navigates a 10-floor map with branching paths and must stay on their chosen path.

## Acceptance Criteria

- [ ] Map has 10 floors with 3 combat segments separated by convergence nodes (Merchant at 3, Rest at 6, Boss at 9)
- [ ] Each combat segment has 2–3 columns; choosing a column at floor N constrains available columns at floor N+1
- [ ] `MapState` carries the full graph and `available_cols` — player cannot choose an unreachable node
- [ ] Plain-text map display shows all floors as a grid, marks visited nodes (✓), marks the current floor (▶), and lists available choices
- [ ] `ChooseNode(col)` replaces the bare Enter-to-advance; player picks column among `available_cols`
- [ ] Combat and Boss nodes carry their enemy lists (no more `enemies_for_floor`)
- [ ] All existing tests pass with new structure; new tests cover DAG generation and path constraints

## Data Structures

```rust
// MapNode carries its own enemies now
pub enum MapNode {
    Combat(Vec<EnemyKind>),
    RestSite,
    Boss(Vec<EnemyKind>),
    Merchant,
}

// edges[row][col] = list of col indices in row+1 that this node connects to
pub struct MapGraph {
    pub rows: Vec<Vec<MapNode>>,
    pub edges: Vec<Vec<Vec<usize>>>,
}

pub struct MapState {
    pub player: Player,
    pub floor: usize,
    pub graph: MapGraph,
    pub available_cols: Vec<usize>,  // cols reachable from where we are
    pub scenario: Scenario,
}

// GameState::Combat gains is_boss
GameState::Combat { state: CombatState, floor: usize, is_boss: bool, scenario: Scenario }
```

## 10-Floor Layout

```
Floor  Type          Cols  Notes
─────────────────────────────────────
  9    Boss           1    convergence — single boss node
  8    Combat         2    segment 3
  7    Combat         2    segment 3
  6    Rest           1    convergence
  5    Combat         2    segment 2
  4    Combat         2    segment 2
  3    Merchant       1    convergence
  2    Combat         2    segment 1
  1    Combat         2    segment 1
  0    Combat         2    segment 1 (start, both cols available)
```

## Edge Rules (per 3-floor segment)

Each segment has 2 columns (L=0, R=1):

```
F[n][L] → { F[n+1][L], F[n+1][R] }   (left can go to either)
F[n][R] → { F[n+1][L], F[n+1][R] }   (right can go to either)
```

But `available_cols` is computed from the player's _actual chosen column_, so:

- If player chose col 0 at floor N, `available_cols` for floor N+1 = edges[N][0]
- If player chose col 1 at floor N, `available_cols` for floor N+1 = edges[N][1]

For segment start (floor 0, 3→4, 6→7): both cols are available.
Convergence nodes (floor 3, 6, 9): single col, `available_cols = [0]`.

## Steps

Every step follows RED-GREEN-MUTATE-KILL MUTANTS-REFACTOR.

### Step 1: Graph types and generation (`slay-core`)

**Acceptance criteria**: `MapGraph` and updated `MapNode` exist; `generate_map(rng) -> MapGraph` produces a valid 10-floor graph with correct node types, correct column counts per floor, and edges that respect the DAG rules. `new_run` and `new_simple_run` use the new `MapState` (with `graph` + `available_cols`). `MAP_NODES` and `enemies_for_floor` are removed.

**RED**: Tests for `generate_map`:

- `map_has_ten_floors`
- `convergence_floors_have_one_column` (floors 3, 6, 9)
- `combat_floors_have_two_columns` (all others)
- `boss_floor_node_is_boss_variant`
- `merchant_floor_node_is_merchant_variant`
- `rest_floor_node_is_restsite_variant`
- `edges_from_combat_floor_reach_next_floor_columns`
- `available_cols_starts_as_both_columns`

**GREEN**: Add `MapGraph`, update `MapNode`, write `generate_map`, update `MapState`, update `new_run`/`new_simple_run`. Remove `MAP_NODES` and `enemies_for_floor`. Update any callers in `run.rs` that used them (the `ChooseNode` handler will use `graph` instead).

**MUTATE**: Run `cargo mutants -p slay-core` — produce report.

**KILL MUTANTS**: Address survivors.

**REFACTOR**: Assess.

**Done when**: All above tests pass, no regressions, mutation report reviewed, human approves commit.

---

### Step 2: State machine — `ChooseNode(col)`, path tracking, `is_boss` (`slay-core`)

**Acceptance criteria**: `ChooseNode(col)` succeeds only when `col` is in `available_cols`; after choosing, `available_cols` for the next floor is derived from `edges[floor][col]`. Entering a `Boss` node sets `is_boss: true` on `GameState::Combat`. `ChooseNode` with invalid col returns `CommandError::InvalidPhase`. The scenario simple run still works (no graph needed — it can use a minimal 1-col graph or keep `Scenario::Simple` special-cased).

**RED**: Tests:

- `choose_node_advances_to_combat_with_correct_enemies`
- `choose_node_rejects_unavailable_col`
- `choose_node_updates_available_cols_for_next_floor`
- `choose_node_at_convergence_advances_without_choice`
- `boss_node_sets_is_boss_true`
- `non_boss_node_sets_is_boss_false`
- `full_run_with_graph_reaches_victory` (integration — update existing)

**GREEN**: Update `apply_command` `GameState::Map` arm to use `graph` + `available_cols`. Add `is_boss` to `GameState::Combat`. Update post-combat transition to carry forward `available_cols` edges. Update `Scenario::Simple` path as needed.

**MUTATE**: Run `cargo mutants -p slay-core` — produce report.

**KILL MUTANTS**: Address survivors.

**REFACTOR**: Assess.

**Done when**: All above tests pass, no regressions, mutation report reviewed, human approves commit.

---

### Step 3: Renderers — plain text + TUI (`slay-tui`)

**Acceptance criteria**: Plain-text map shows a grid with all 10 floors, marks visited (✓), marks current floor (▶), marks available choices, and prints `[1] Name  [2] Name` for the current floor's choices. TUI map view shows the same structure. `ChooseNode` replaces the bare Enter command in both `command.rs` and `tui.rs`.

**RED**: Update/add snapshot script that exercises choosing a path. Add TUI test that asserts map grid text appears in buffer.

**GREEN**: Update `render_map` in `game.rs` to iterate `graph.rows`, display grid, show `available_cols` choices. Update `command.rs` `parse_map` to emit `ChooseNode(n-1)` instead of `ChooseNode(0)`. Update `tui.rs` map rendering and key handler.

**MUTATE**: Run `cargo mutants -p slay-tui` — produce report.

**KILL MUTANTS**: Address survivors.

**REFACTOR**: Assess.

**Done when**: Snapshot tests pass (accept new snapshots), TUI test passes, mutation report reviewed, human approves commit.

## Pre-PR Quality Gate

Before PR:

1. Mutation testing — run `mutation-testing` skill
2. Refactoring assessment — run `refactoring` skill
3. `cargo clippy --all -- -D warnings` passes
4. `cargo test --all` passes

---

_Delete this file when the plan is complete. If `plans/` is empty, delete the directory._

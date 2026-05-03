# Plan: Scenario System + Snapshot Testing

**Status**: Active

## Goal

Add a `Simple` scenario mode with deterministic RNG and explicit enemy spawning, then use it to snapshot-test the full TUI output from `.slay` script files.

## Acceptance Criteria

- [ ] `EnemyKind` has string IDs (`id()` / `from_id()`) like `Card` and `Relic`
- [ ] `AnyRng` enum wraps `ThreadRng` | `NoOpRng` and implements `Rng` via enum dispatch
- [ ] `Command::Spawn(Vec<EnemyKind>)` sets the next combat's enemies; `ChooseNode(0)` consumes them
- [ ] `Scenario::Simple` starts with an empty deck, uses `NoOpRng`, skips card rewards (returns straight to map after combat)
- [ ] `run_game(reader, writer, rng, scenario, debug)` is a public function in `slay_tui`; `main()` delegates to it
- [ ] `slay-tui/tests/scripts.rs` discovers `scripts/simple/*.slay` files and snapshot-tests each one with `insta`
- [ ] 4–5 example `scripts/simple/*.slay` files covering key mechanics

## Steps

### Step 1: Enemy IDs

**What**: Add `EnemyKind::id(&self) -> &'static str` and `EnemyKind::from_id(s: &str) -> Option<EnemyKind>` in `slay-core/src/enemies/mod.rs`, mirroring the pattern on `Card` and `Relic`.

IDs: `"louse"`, `"fungibeast"`, `"cultist"`, `"jaw-worm"`, `"small-spike-slime"`, `"red-louse"`.

**RED**: Tests in `enemies/mod.rs` — `louse_id_round_trips`, `unknown_id_returns_none`, one test per variant.
**GREEN**: Add `id()` match arm and `from_id()` match arm.
**MUTATE**: `cargo mutants -p slay-core` scoped to new functions.
**REFACTOR**: Check if any existing code can use `id()` instead of inline strings.
**Done when**: All ID round-trip tests pass, no surviving mutants on these functions.

---

### Step 2: `AnyRng`

**What**: Add `pub enum AnyRng { Thread(ThreadRng), NoOp(NoOpRng) }` to `slay-core/src/rng.rs`, implementing `Rng` via enum dispatch.

**RED**: Test in `rng.rs` — `any_rng_noop_does_not_shuffle` (wrap a known-order slice, shuffle with `AnyRng::NoOp`, assert order unchanged), `any_rng_thread_compiles` (just construct and call shuffle — can't assert randomness, but proves it compiles and runs).
**GREEN**: Add the enum and `impl Rng for AnyRng`.
**MUTATE**: Scoped to `AnyRng` impl — surviving mutants on a no-op shuffle are expected (nothing to kill there).
**REFACTOR**: None expected.
**Done when**: Tests pass, `AnyRng` is exported from `slay_core`.

---

### Step 3: Spawn command

**What**: Add `Command::Spawn(Vec<EnemyKind>)` to `slay-core`. Add `next_enemies: Option<Vec<EnemyKind>>` to `MapState`. When `ChooseNode(0)` is processed and `next_enemies` is `Some`, use those enemies for the combat (and clear the field); otherwise fall back to `enemies_for_floor`.

**RED**: Tests in `run.rs` — `spawn_command_sets_next_enemies`, `choose_node_after_spawn_uses_spawned_enemies` (verify the combat starts with the spawned enemy kind, not the floor default).
**GREEN**: Add `next_enemies` to `MapState`, handle `Command::Spawn` in `apply_command`, update `ChooseNode` handler to check `next_enemies` first.
**MUTATE**: Scoped to the new handler branches.
**REFACTOR**: None expected.
**Done when**: Tests pass; `Command::Spawn` is exported from `slay_core`.

---

### Step 4: Parse `spawn` in TUI command parser

**What**: Add `spawn <id> [<id> ...]` parsing to `parse_map` in `slay-tui/src/command.rs`. Always available (not debug-only). Returns `Command::Spawn(enemies)` — silently ignores unknown IDs (skips them).

**RED**: Unit tests in `command.rs` — `spawn_single_enemy`, `spawn_multiple_enemies`, `spawn_unknown_id_ignored`, `spawn_empty_after_all_unknown`.
**GREEN**: Parse the `spawn` prefix, split remaining tokens, map each via `EnemyKind::from_id`, collect the `Some` values.
**MUTATE**: Scoped to parse function.
**REFACTOR**: None expected.
**Done when**: Parser tests pass.

---

### Step 5: `Scenario` type + Simple run

**What**: Add `pub enum Scenario { Main, Simple }` to `slay-core`. Add `pub fn new_simple_run() -> GameState` that creates a player with an empty deck and a `MapState` with floor 0. In `apply_command`, when combat ends in `Simple` scenario (enemy dies), return directly to `MapState` instead of `CardRewardState` — skip the reward entirely.

The scenario needs to be tracked somewhere reachable by `apply_command`. The simplest approach: thread it through `run.rs` by storing `scenario: Scenario` on `MapState` and `CombatState` (behind a simple field, not a big refactor).

**RED**: Tests in `run.rs` — `simple_run_starts_with_empty_deck`, `simple_run_combat_win_returns_to_map_not_reward`.
**GREEN**: Add `Scenario` enum, `new_simple_run`, `scenario` field on `MapState`/`CombatState`, conditional in combat-win branch.
**MUTATE**: Scoped to new branches.
**REFACTOR**: None expected.
**Done when**: Tests pass; `Scenario` and `new_simple_run` exported from `slay_core`.

---

### Step 6: Game loop refactor — extract `run_game`

**What**: Extract the game loop body from `main()` into `pub fn run_game(reader: impl BufRead, writer: &mut impl Write, rng: &mut AnyRng, scenario: Scenario, debug: bool)` in `slay-tui/src/lib.rs`. Replace all `println!` / `print!` in the game loop and render functions with `writeln!(writer)` / `write!(writer)`. `main()` becomes a thin wrapper that constructs reader/writer/rng and calls `run_game`.

This is purely mechanical — no behaviour changes.

**RED**: A smoke test in `slay-tui/tests/scripts.rs` (or inline) — call `run_game` with a script reader containing just `win\nskip\n`, a `Vec<u8>` writer, `AnyRng::NoOp(NoOpRng)`, `Scenario::Simple`, and verify the output is non-empty and contains "Enemy slain" or similar. (This test will fail to compile until `run_game` exists.)
**GREEN**: The mechanical refactor — move all render/print logic to use the writer.
**MUTATE**: Skip mutation testing for this step — it's a pure refactor with no new logic.
**REFACTOR**: Clean up any awkward `write!` formatting that arose.
**Done when**: Existing tests still pass, TUI binary still works interactively, smoke test passes.

---

### Step 7: Snapshot test harness

**What**: Add `insta` to `slay-tui` dev-dependencies. In `slay-tui/tests/scripts.rs`, discover all `*.slay` files under `scripts/simple/`, run each through `run_game` with `NoOpRng` + `Scenario::Simple`, and assert the captured output with `insta::assert_snapshot!`. Script files may have a `scenario=simple` header line (a comment directive) — the harness checks for it; files without it are skipped (reserved for future `scenario=main` tests).

**RED**: Add `insta` dep, write the harness with one placeholder script `scripts/simple/00-smoke.slay` containing `spawn louse\nenter\nwin\n`. The snapshot won't exist yet — `cargo test` will fail (insta pending review).
**GREEN**: Run `cargo insta review` to accept the initial snapshot. All further test runs pass until the script or game output changes.
**REFACTOR**: Ensure the test name derives cleanly from the filename (e.g. `00-smoke` → snapshot name `scripts__00-smoke`).
**Done when**: `cargo test` passes, snapshot file committed to `scripts/simple/snapshots/`.

---

### Step 8: Example simple scripts

**What**: Write 4–5 `.slay` files in `scripts/simple/` covering:

- `01-basic-combat.slay` — spawn louse, add strike×3 + defend×2, fight to win
- `02-poison.slay` — spawn louse, add deadly-poison + defend, watch poison tick
- `03-cultist-ritual.slay` — spawn cultist, end turns to watch Strength escalate
- `04-flame-tackle-dazed.slay` — spawn small-spike-slime, end turn, inspect discard for Dazed
- `05-vulnerable.slay` — spawn louse, add bash + strike, Bash applies Vulnerable, Strike hits harder

Run `cargo insta review` to accept snapshots for all new scripts.

**Done when**: All 5 scripts have accepted snapshots and `cargo test` is green.

---

## Pre-merge Quality Gate

- `cargo test` passes
- `cargo clippy` passes with zero warnings
- All insta snapshots committed
- `plans/scenario-snapshot-testing.md` deleted

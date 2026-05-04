---
name: Known equivalent/unreachable mutants in run.rs
description: Surviving cargo-mutants mutants in slay-core/src/run.rs that are confirmed equivalent or unreachable — do not re-investigate
type: project
---

After mutation testing on `crates/slay-core/src/run.rs`, these 4 survivors are confirmed non-issues:

**Lines 429, 435 — `&& → ||` in `apply_command` (equivalent mutants)**

```rust
// Line 429
if events.contains(&Event::TurnEnded) && matches!(new_combat.phase, CombatPhase::EnemyTurn)
// Line 435
if events.iter().any(|e| matches!(e, Event::TurnStarted { .. })) && matches!(new_combat.phase, CombatPhase::PlayerTurn)
```

`TurnEnded` is only ever emitted by `EndTurn`, which also transitions phase to `EnemyTurn`. The two conditions always coincide, so `&&` and `||` produce identical observable behavior. Genuinely equivalent mutants — no test can distinguish them.

**Line 331 — `floor + 1` → `floor - 1 / floor * 1` in the ChooseNode instant-win path (unreachable in tests)**

```rust
// This branch fires only if enemies already have ≤0 HP on node entry
if combat_state.enemies.iter().all(|e| e.hp <= Hp(0)) { ... floor + 1 ... }
```

In every test, enemies have positive HP when `ChooseNode` is called, so this branch is never taken. The mutation survives because the path is dead in the test suite. Not worth testing — this is a defensive guard for edge cases (e.g. Spawn + modified HP), not a primary flow.

**Why:** Investigated during Step 2 of the Branching Graph Map plan. Killed 16 other mutants before reaching these 4.

**How to apply:** When running `cargo mutants -p slay-core --file crates/slay-core/src/run.rs`, expect 4 survivors + 15 unviable. These are the known 4; any NEW survivors beyond these warrant investigation.

---

## slay-tui — pre-existing survivors (~77 after Step 3 cleanup)

After Step 3 of the Branching Graph Map plan, `command.rs` was driven to 0 survivors. The remaining ~77 survivors across `slay-tui` are pre-existing and fall into categories that are genuinely hard or not worth testing:

**`main.rs` (~7 survivors)** — CLI argument parsing (`--plain`, `--script`, `--debug` flag checks). Not unit-testable without running the binary.

**`engine.rs` (~3 survivors)** — `describe_event` edge-case path, `pile_names` return value. Low value; event descriptions are snapshot-tested end-to-end.

**`game.rs` (~15 survivors)** — Rendering functions (`render_rest`, `render_card_reward`, `render_shop`, `render_pile`), pile view shortcut arms (`z`, `x`, `c`), game-over transition arms. These are all visual/IO output; snapshot tests cover the end-to-end render but don't distinguish all internal arithmetic mutations.

**`tui.rs` (~52 survivors)** — Layout arithmetic in `render_pile_overlay` (centered popup width/height/offset math), `hp_bar` bar-length calculation, `render_hand` card count guard, `render_rest`/`render_game_over` heal arithmetic (duplicate of game.rs logic), `run_tui` event-loop control flow and key handler arms. Layout math is visual; the control flow is integration-tested via `handle_enter` but not all branches.

**How to apply:** When running `cargo mutants -p slay-tui`, expect ~77 survivors. Focus attention on any NEW survivors in `command.rs`, `engine.rs`, or `game.rs` rendering logic that aren't in this list.

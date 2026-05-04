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

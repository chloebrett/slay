# Plan: Lagavulin Elite

**Branch**: main
**Status**: In progress — steps 1–3 and the map node portion of step 6 are done; steps 4, 5, and initial-statuses (step 6) remain.

## Acceptance Criteria

- [x] Lagavulin has 109 HP and appears as an Elite node on the map
- [ ] Lagavulin starts with Metallicize 8 and Sleep 3 when spawned into combat (**gap**: `initial_statuses()` not implemented)
- [ ] Lagavulin sleeps for up to 3 turns; if the player deals no HP damage in that time, it wakes without stunning (**gap**: auto-wake-without-stun not implemented)
- [ ] If the player deals HP damage while Lagavulin is sleeping, it wakes immediately and is Stunned for 1 turn (**gap**: `on_player_attack_damage` not implemented for Lagavulin)
- [ ] On waking, Metallicize is removed (**gap**: tied to the two items above)
- [x] Awake attack cycle: Attack 18 → Attack 18 → SiphonSoul → repeat
- [x] SiphonSoul applies -1 Strength and -1 Dexterity to the player
- [ ] When spawned awake (Dead Adventurer event path), cycle begins with SiphonSoul

## Implementation Note: Sleep vs SleepCounter

The plan specifies `StatusEffect::SleepCounter`, but the implementation uses `StatusEffect::Sleep` (a plain decrementing debuff). This is functionally equivalent for the counter/decrement behaviour. The remaining steps below use `Sleep` to match what's in the code.

## Architecture Notes

- **Metallicize** is a new `StatusEffect` variant — at end of the enemy's turn (in `EndEnemyTurn` processing), an enemy with Metallicize N gains N block.
- **Sleep counter** can be tracked via a dedicated `StatusEffect::SleepCounter` (counts down from 3). Present while sleeping; removed when Lagavulin wakes.
- **Stunned** is a new `StatusEffect::Stunned` — an enemy with Stunned skips its move and the stacks tick down at end of turn (like other debuffs).
- **Wake-on-damage** lives in `on_player_attack_damage` for `EnemyKind::Lagavulin` — if SleepCounter is present and HP damage was dealt, return a reaction that removes SleepCounter+Metallicize and sets Stunned 1 (and forces move to `LagavulinStunned`).
- **SiphonSoul** applies `ApplyStatus(Strength, -1)` and `ApplyStatus(Dexterity, -1)` to the player.
- **Elite map node**: add `MapNode::Elite(Vec<EnemyKind>)`. Place one Elite node in the map (around row 5–6 in Act 1). Enemies in Elite nodes get displayed/fought the same as Combat nodes.

## Steps

Every step follows RED-GREEN-MUTATE-KILL MUTANTS-REFACTOR. No production code without a failing test.

---

### ~~Step 1: Add Metallicize status effect~~ ✅ DONE

**Acceptance criteria**: An enemy with `StatusEffect::Metallicize(N)` gains N block at the end of its turn (after its move resolves), visible via the block value on the enemy state.

**RED**: Test in `combat.rs` tests — enemy with Metallicize 8, `EndEnemyTurn`, assert enemy block == 8.

**GREEN**: Add `Metallicize` to `StatusEffect` enum. In `EndEnemyTurn` processing loop (after tick_statuses), check `get_stacks(&state.enemies[i].statuses, StatusEffect::Metallicize)`; if > 0, add that many to `state.enemies[i].block`, push `Event::EnemyDefended`.

**MUTATE**: Run `cargo mutants -f src/combat.rs`.

**KILL MUTANTS**: Add boundary test (Metallicize 0 → no block gain).

**REFACTOR**: Assess only.

**Done when**: Metallicize adds block after enemy turn; tests pass; mutation report reviewed.

---

### ~~Step 2: Add Stunned status effect~~ ✅ DONE

**Acceptance criteria**: An enemy with `StatusEffect::Stunned` (≥1 stack) skips its move on that turn; Stunned decrements at end of turn (same as other debuff ticks).

**RED**: Test — enemy with Stunned 1 and a damage move; `EndEnemyTurn`; assert player HP unchanged and stunned gone afterward.

**GREEN**: In `EndEnemyTurn` loop, before executing `current_move.def().effects`, check if `get_stacks(&state.enemies[i].statuses, StatusEffect::Stunned) > 0` — if so, skip the effects block entirely. `tick_statuses` already handles decrement (since Stunned is a debuff-like counter).

**MUTATE**: Run `cargo mutants -f src/combat.rs`.

**KILL MUTANTS**: Add test that Stunned 2 still blocks the move on the first turn but not the second.

**REFACTOR**: Assess only.

**Done when**: Stunned prevents move execution for exactly N turns; tests pass; mutation report reviewed.

---

### ~~Step 3: Implement Lagavulin enemy module~~ ✅ DONE

**Acceptance criteria**: `EnemyKind::Lagavulin` exists with HP 109, named "Lagavulin". Sleep state is encoded via `SleepCounter` status (starts at 3). Moves: `LagavulinSleep` (no effects, placeholder), `LagavulinStunned` (no effects), `LagavulinAttack` (18 damage), `LagavulinSiphonSoul` (-1 Str, -1 Dex to player). Move sequence when awake: Attack → Attack → SiphonSoul → repeat.

**RED**: Tests for HP, name, first move (LagavulinSleep), awake cycle (Attack → Attack → SiphonSoul → Attack), and move intents.

**GREEN**:
- Add `crates/slay-core/src/enemies/lagavulin.rs` with `DEF`, `next_move`.
- Add `LagavulinSleep`, `LagavulinStunned`, `LagavulinAttack`, `LagavulinSiphonSoul` to `Move` enum.
- Wire `LagavulinSiphonSoul` in `Move::def()` with `ApplyStatus(Strength, -1)` + `ApplyStatus(Dexterity, -1)`.
- Add `EnemyKind::Lagavulin` to all match arms in `mod.rs`.
- `next_move` for Lagavulin: if last is None → `LagavulinSleep`. If last is Sleep/Stunned → `LagavulinAttack`. Awake cycle: Attack→Attack→Siphon→Attack→... (track via last move).

**MUTATE**: Run `cargo mutants -f src/enemies/lagavulin.rs`.

**KILL MUTANTS**: Ensure cycle ordering tested precisely.

**REFACTOR**: Assess only.

**Done when**: Lagavulin moves, HP, name all correct; tests pass; mutation report reviewed.

---

### Step 4: Wake-on-damage reaction

**Acceptance criteria**: When the player deals HP damage to a sleeping Lagavulin (SleepCounter present), Lagavulin immediately wakes — SleepCounter removed, Metallicize removed, Stunned 1 applied, next move forced to `LagavulinStunned`. When sleeping with no HP damage, nothing changes.

**RED**: Test in `combat.rs` — Lagavulin sleeping (SleepCounter 3, Metallicize 8), player plays an attack that deals damage; assert SleepCounter gone, Metallicize gone, Stunned 1 present.

**GREEN**: In `on_player_attack_damage` for `EnemyKind::Lagavulin`, if `statuses.contains_key(&StatusEffect::SleepCounter)` and `hp_lost > 0`, return `EnemyDamageReaction` with `silent_sets` removing SleepCounter + Metallicize, status_events for Stunned 1, `force_move: Some(Move::LagavulinStunned)`.

**MUTATE**: Run `cargo mutants -f src/enemies/lagavulin.rs`.

**KILL MUTANTS**: Test that 0 HP lost (all blocked) does NOT wake Lagavulin.

**REFACTOR**: Assess only.

**Done when**: Wake-on-damage reaction correct; tests pass; mutation report reviewed.

---

### Step 5: Sleep counter auto-wake

**Acceptance criteria**: Lagavulin's SleepCounter decrements at end of each enemy turn. When it reaches 0, Lagavulin wakes (SleepCounter + Metallicize removed) without being stunned — next move becomes `LagavulinAttack`.

**RED**: Test — Lagavulin with SleepCounter 1; `EndEnemyTurn`; assert SleepCounter gone, Metallicize gone, next move is Attack.

**GREEN**: In `EndEnemyTurn` loop, after processing moves and ticking statuses: if enemy is Lagavulin and SleepCounter just hit 0 (was decremented to 0 by tick), remove Metallicize too and set next move to `LagavulinAttack`.

**MUTATE**: Run `cargo mutants -f src/combat.rs`.

**KILL MUTANTS**: Add boundary tests for SleepCounter 2→1 (still sleeping) vs 1→0 (wake).

**REFACTOR**: Assess only.

**Done when**: 3-turn auto-wake works correctly; tests pass; mutation report reviewed.

---

### Step 6: Add Elite map node and place Lagavulin — ⚠️ PARTIALLY DONE

**Acceptance criteria**: `MapNode::Elite(Vec<EnemyKind>)` exists ✅. `generate_map` places one Elite node (containing Lagavulin) at a fixed row ✅. The run correctly processes Elite nodes as combat encounters ✅. Lagavulin starts with Sleep 3 and Metallicize 8 when spawned into combat ❌ (`initial_statuses()` not yet implemented).

**RED**: Tests — generated map contains exactly one Elite node; Elite node round-trips through the run command flow; Lagavulin combat starts with correct initial statuses.

**GREEN**:
- Add `MapNode::Elite(Vec<EnemyKind>)` to `MapNode` enum.
- In `generate_map`, add `MapNode::Elite(vec![EnemyKind::Lagavulin])` at row 5 (after the merchant row).
- In `apply_command` for `ChooseNode`, handle `MapNode::Elite` the same as `MapNode::Combat` (spawn enemies, start combat).
- In `spawn_enemy` (or wherever enemies are initialized), if kind is `EnemyKind::Lagavulin`, set initial statuses: SleepCounter 3, Metallicize 8.

**MUTATE**: Run `cargo mutants -f src/run.rs`.

**KILL MUTANTS**: Add test verifying Lagavulin's initial statuses in a freshly spawned combat.

**REFACTOR**: Assess only.

**Done when**: Map contains Elite node with Lagavulin; Lagavulin starts sleeping; tests pass; mutation report reviewed.

---

## Pre-PR Quality Gate

Before PR:
1. Mutation testing — run `mutation-testing` skill across all changed files
2. Refactoring assessment — run `refactoring` skill
3. `cargo clippy` and `cargo test` pass
4. Snapshot tests pass (`cargo test -p slay-tui --test scripts`)

---

*Delete this file when the plan is complete. If `plans/` is empty, delete the directory.*

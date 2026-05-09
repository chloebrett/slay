# Large Slimes — Implementation Plan

## Source
https://slay-the-spire.fandom.com/wiki/Spike_Slime_(L)
https://slay-the-spire.fandom.com/wiki/Acid_Slime_(L)
https://slay-the-spire.fandom.com/wiki/Spike_Slime_(M)
https://slay-the-spire.fandom.com/wiki/Acid_Slime_(M)

---

## Enemy Stats

| Enemy            | HP (mid) | Splits into        |
|------------------|----------|--------------------|
| Spike Slime (L)  | 67       | 2× Spike Slime (M) |
| Acid Slime (L)   | 67       | 2× Acid Slime (M)  |
| Spike Slime (M)  | 30       | —                  |
| Acid Slime (M)   | 30       | —                  |

Small slimes already implemented as `SmallSpikeSlime` / `SmallAcidSlime`.
Medium slimes do not split further (only Large slimes have the Split mechanic).

---

## Moves

### Spike Slime (L)
- **Flame Tackle:** 16 dmg + 2 Slimed shuffled into discard. *(not Dazed — Slimed is the slime-specific status card)*
- **Lick:** Apply 2 Frail to player.
- **Move pattern:** Flame Tackle 30% / Lick 70%; cannot use the same move 3 turns in a row.

### Spike Slime (M)
- **Flame Tackle:** 8 dmg + 1 Slimed shuffled into discard.
- **Lick:** Apply 1 Frail to player.
- **Move pattern:** Same probability / no-repeat rule as Large.

### Acid Slime (L)
- **Corrosive Spit:** 11 dmg + 2 Slimed shuffled into discard.
- **Lick:** Apply 2 Weak to player.
- **Tackle:** 16 dmg (pure damage, no effect).
- **Move pattern:** Probabilistic (exact weights TBD from further research); cannot use same move 3× in a row.

### Acid Slime (M)
- **Corrosive Spit:** 7 dmg + 1 Slimed shuffled into discard.
- **Lick:** Apply 1 Weak to player.
- **Tackle:** 10 dmg.
- **Move pattern:** Same as Large.

---

## Split Mechanic — Corrected Model

### When
When a Large slime's HP drops to **≤ 50% of its max HP** while still alive (hp > 0), its **next intent is replaced with Split**. The split executes on the enemy's own turn.

The split does NOT fire immediately — the slime plays out its current turn first (e.g. attacks), then on the next enemy turn the Split executes.

### If killed before splitting
If the slime's HP reaches 0 before its Split turn arrives, **no smaller slimes appear**. The enemy just dies normally.

### Split execution
When the Split move executes on the enemy's turn:
1. The splitting slime is removed from `state.enemies`.
2. Two new smaller slimes are inserted at that position.
3. **Each new slime's HP = the splitting slime's current HP at the moment of split** (i.e. whatever HP it had when the Split fires, which may be lower than when the threshold was crossed).
4. Each new slime starts with its first-turn move (`next_move(kind, None, rng)`).
5. `Event::EnemySplit` fires instead of `Event::EnemyDied`.

---

## Implementation Steps

### Step 1 — `Card::Slimed`

Spike and Acid slimes both shuffle **Slimed** into the discard pile, not Dazed. Slimed is a Status card: unplayable, 0 cost, no effect, exhausts when played (like Dazed).

**File:** `crates/slay-core/src/cards/` — add `Slimed` variant, add to `CardDef`, export from `lib.rs`.

**Tests:** name, type (Status), cost (0), exhausts.

---

### Step 2 — New `Move` variants

**File:** `crates/slay-core/src/enemies/mod.rs`

```
// Spike Slime (L)
LargeSpikeFlameTackle,   // 16 dmg + 2× Slimed to discard
LargeSpikeLick,          // 2 Frail
LargeSpikeSplit,         // no effects — handled by EndEnemyTurn split logic

// Spike Slime (M)
MediumSpikeFlameTackle,  // 8 dmg + 1× Slimed to discard
MediumSpikeLick,         // 1 Frail

// Acid Slime (L)
LargeAcidCorrosiveSpit,  // 11 dmg + 2× Slimed to discard
LargeAcidLick,           // 2 Weak
LargeAcidTackle,         // 16 dmg
LargeAcidSplit,          // no effects — handled by EndEnemyTurn split logic

// Acid Slime (M)
MediumAcidCorrosiveSpit, // 7 dmg + 1× Slimed to discard
MediumAcidLick,          // 1 Weak
MediumAcidTackle,        // 10 dmg
```

Add `def()` entries. `LargeSpikeSplit` / `LargeAcidSplit` have empty effects — their actual logic is in `EndEnemyTurn`, not in the effects loop.

`intent()` for split moves: return a new `Intent::Split` variant so the TUI can show a distinct icon.

---

### Step 3 — `Intent::Split`

**File:** `crates/slay-core/src/enemies/mod.rs`

Add `Split` to the `Intent` enum. Update `describe_intent` in `engine.rs` to display `"🔀 Split"`.

---

### Step 4 — `on_player_attack_damage` signature + slime reactions

**File:** `crates/slay-core/src/enemies/mod.rs`

The existing signature lacks `current_hp` and `max_hp`. Extend it:

```rust
pub fn on_player_attack_damage(
    kind: &EnemyKind,
    statuses: &StatusMap,
    hp_lost: i32,
    current_hp: Hp,
    max_hp: Hp,
) -> Option<EnemyDamageReaction>
```

For each Large slime, when `current_hp.0 <= max_hp.0 / 2`, return:
```rust
Some(EnemyDamageReaction { force_move: Some(Move::LargeSpikeSplit), .. })
```

Update the call site in `combat.rs` to pass `current_hp` and `max_hp`.

Also update `the_guardian::on_player_attack_damage` call to match new signature (guardian doesn't use these fields, just add `_current_hp, _max_hp`).

**Tests:**
- Large spike slime at 33 HP (max 67): returns `force_move = LargeSpikeSplit`
- Large spike slime at 34 HP (max 67, >50%): returns `None`
- Large spike slime at 0 HP: `on_player_attack_damage` is not called (guarded by `hp > 0` check in `combat.rs` — no test needed, relies on existing guard)

---

### Step 5 — Split execution in `EndEnemyTurn`

**File:** `crates/slay-core/src/combat.rs`

In the `EndEnemyTurn` handler, before executing move effects, check if the move is a Split:

```rust
if matches!(current_move, Move::LargeSpikeSplit | Move::LargeAcidSplit) {
    let current_hp = state.enemies[i].hp;
    let spawn_kinds: Vec<EnemyKind> = match state.enemies[i].kind {
        EnemyKind::LargeSpike => vec![EnemyKind::MediumSpike, EnemyKind::MediumSpike],
        EnemyKind::LargeAcid  => vec![EnemyKind::MediumAcid,  EnemyKind::MediumAcid],
        _ => unreachable!(),
    };
    state.enemies.remove(i);
    for (offset, kind) in spawn_kinds.into_iter().enumerate() {
        let first_move = next_move(&kind, None, &StatusMap::new(), rng);
        state.enemies.insert(i + offset, Enemy {
            kind, hp: current_hp, max_hp: kind.max_hp(),
            block: Block(0), move_: first_move, last_move: None,
            statuses: StatusMap::new(),
        });
    }
    events.push(Event::EnemySplit);
    // Index i now points to first spawn — continue loop from i+2
    // Adjust loop: after split, skip to the next original enemy
    continue; // or restructure the loop
}
```

**Index stability:** After removing index `i` and inserting 2, the overall `enemies` slice grows by 1. The `for i in 0..state.enemies.len()` loop uses a snapshot length so this must be handled carefully — use an index-based loop that re-derives `len()` each iteration, or collect indices first.

**No `EnemyDied` for split slime.** The split path `continue`s past the normal die/victory checks.

**New event:**
```rust
Event::EnemySplit,
```
Add to `Event` enum and `describe_event` in `engine.rs`: `"🔀 Slime splits!"`.

**Tests:**
- Slime with `LargeSpikeSplit` move executes split on `EndEnemyTurn`: enemy count goes from 1 to 2
- New slimes are `MediumSpike` kind
- New slimes have HP = split slime's current HP
- `Event::EnemySplit` is in the returned events
- `Event::EnemyDied` is NOT in the returned events
- After split, if both medium slimes die → `CombatPhase::Victory`

---

### Step 6 — Four slime files

**Files:**
- `enemies/large_spike_slime.rs` — `DEF`, `next_move` (Flame Tackle 30% / Lick 70%, no 3× repeat), `on_player_attack_damage` (split at ≤50%)
- `enemies/medium_spike_slime.rs` — `DEF`, `next_move` (same pattern, no split)
- `enemies/large_acid_slime.rs` — `DEF`, `next_move` (Corrosive Spit / Lick / Tackle probabilistic, no 3× repeat), `on_player_attack_damage` (split at ≤50%)
- `enemies/medium_acid_slime.rs` — `DEF`, `next_move` (same moves, no split)

---

### Step 7 — Wire `EnemyKind` variants

Add `LargeSpike`, `MediumSpike`, `LargeAcid`, `MediumAcid` to all dispatch tables:
- `def()`, `id()`, `from_id()`, `next_move()`, `on_player_attack_damage()`
- `enemy_icon()` in `engine.rs` (🟢 for acid slimes, 🔵 for spike slimes, or use 🫧 for both)

---

### Step 8 — Snapshot test

`tests/scripts/08-large-slime-splits.slay`:
1. Spawn large spike slime (HP 67)
2. Attack it enough to drop below 33 HP (without killing)
3. End turn → slime executes its current intent
4. End enemy turn → ... (may need multiple rounds to trigger split turn)

This is complex to script without knowing exact card damage. Consider instead a TUI unit test that directly sets up a slime with `LargeSpikeSplit` intent and calls `EndEnemyTurn`.

---

## What Changed vs. Original Plan

| Topic | Original (wrong) | Corrected |
|-------|-----------------|-----------|
| Split timing | Immediate on damage | Replaces intent; fires on enemy's turn |
| Split HP | Full HP of spawns | Current HP of splitting slime |
| Kill = split | Yes | No — if killed, no split |
| Spike Slime debuff card | Dazed | Slimed |
| Spike Slime secondary move | Lunge (Frail) | Lick (Frail) — no Lunge |
| Acid Slime move count | 2 | 3 (adds Tackle) |
| Move pattern | Strict alternating | Probabilistic, no 3× repeat |

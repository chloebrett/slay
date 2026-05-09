# Large Slimes — Implementation Plan

## Source
https://slay-the-spire.fandom.com/wiki/Spike_Slime_(L)
https://slay-the-spire.fandom.com/wiki/Acid_Slime_(L)

---

## Enemy Stats

| Enemy               | HP  | Splits into           |
|---------------------|-----|-----------------------|
| Spike Slime (L)     | 65  | 2× Spike Slime (M)    |
| Acid Slime (L)      | 68  | 2× Acid Slime (M)     |
| Spike Slime (M)     | 30  | 2× Spike Slime (S) ✓  |
| Acid Slime (M)      | 30  | 2× Acid Slime (S) ✓   |

Small slimes already implemented as `SmallSpikeSlime` / `SmallAcidSlime`.

---

## Moves

### Spike Slime (L)
- **Flame Tackle:** 16 dmg + Dazed to discard. Turn 1.
- **Lunge:** 9 dmg + 2 Frail. Alternates with Flame Tackle.

### Spike Slime (M)
- **Flame Tackle:** 8 dmg + Dazed to discard. Turn 1.
- **Lunge:** 6 dmg + 1 Frail. Alternates with Flame Tackle.

### Acid Slime (L)
- **Corrosive Spit:** 11 dmg + 2 Slimed to discard. Turn 1.
- **Lick:** 2 Weak to player. Alternates with Corrosive Spit.

### Acid Slime (M)
- **Corrosive Spit:** 7 dmg + 1 Slimed to discard. Turn 1.
- **Lick:** 1 Weak to player. Alternates with Corrosive Spit.

All four alternate strictly (no randomness): start with the damage move, alternate each turn.

---

## Split Mechanic

**Trigger:** When a slime's HP drops at or below 50% of its max HP during the player's turn, the split fires immediately after that damage resolves.

**Effect:**
1. The splitting slime is removed from `state.enemies` (it ceases to be a target).
2. Two new smaller slimes are spawned at **their own max HP** (not inherited HP).
3. The two new slimes are inserted into `state.enemies` at the position of the split slime.
4. Each new slime starts with `move_ = next_move(kind, None, &mut rng)` (i.e., their first-turn move).

**Split targets:**
- `LargeSpike` → 2× `MediumSpike`
- `LargeAcid` → 2× `MediumAcid`
- `MediumSpike` → 2× `SmallSpikeSlime`
- `MediumAcid` → 2× `SmallAcidSlime`

**Timing note:** A split slime cannot also "die" — if the attack would bring it to 0 HP, the split fires first (replace with two new enemies), and the new enemies survive at full HP. The `EnemyDied` event is NOT emitted for a split.

---

## New Infrastracture

### Step 1 — `Card::Slimed`
**File:** `crates/slay-core/src/cards/`

A status card (unplayable, costs 0, exhausts when played). Like `Card::Dazed`. Used by `Corrosive Spit` via `Effect::AddToDiscard(Card::Slimed)`.

### Step 2 — New moves
**File:** `crates/slay-core/src/enemies/mod.rs`

Add to `Move` enum:
```
LargeSpikeFlameTackle,   // 16 dmg + Dazed
LargeSpikelunge,         // 9 dmg + 2 Frail
MediumSpikeFlameTackle,  // 8 dmg + Dazed
MediumSpikeLunge,        // 6 dmg + 1 Frail
LargeCorrosiveSpit,      // 11 dmg + 2 Slimed
LargeLick,               // 2 Weak
MediumCorrosiveSpit,     // 7 dmg + 1 Slimed
MediumLick,              // 1 Weak
```

(Naming convention: prefixed by slime tier to avoid collision with Small Spike `FlameTackle` and `Lick`.)

### Step 3 — `EnemyDamageReaction::split_into`
**File:** `crates/slay-core/src/enemies/mod.rs`

Add a new field to `EnemyDamageReaction`:
```rust
pub split_into: Option<Vec<EnemyKind>>,
```

### Step 4 — `on_player_attack_damage` for slimes
**File:** `crates/slay-core/src/enemies/mod.rs`

For each large/medium slime: when `hp_lost > 0` and the enemy's HP is at or below 50% of max HP, return `split_into: Some(vec![smaller, smaller])`. The caller in `combat.rs` will handle replacement.

The check needs the enemy's current HP and max HP, so the signature of `on_player_attack_damage` needs to expand to include `current_hp: Hp, max_hp: Hp`:

```rust
pub fn on_player_attack_damage(
    kind: &EnemyKind,
    statuses: &StatusMap,
    hp_lost: i32,
    current_hp: Hp,
    max_hp: Hp,
) -> Option<EnemyDamageReaction>
```

### Step 5 — Split handling in `combat.rs`
**File:** `crates/slay-core/src/combat.rs`

After processing the reaction (around line 318), check `reaction.split_into`. If `Some(spawn_kinds)`:

```rust
if let Some(spawn_kinds) = reaction.split_into {
    // Replace the split slime with two new ones at actual_target position
    state.enemies.remove(actual_target);
    for (offset, kind) in spawn_kinds.into_iter().enumerate() {
        let hp = kind.max_hp();
        let move_ = enemies::next_move(&kind, None, &StatusMap::new(), rng);
        state.enemies.insert(actual_target + offset, Enemy {
            kind, hp, max_hp: hp, block: Block(0),
            move_, last_move: None, statuses: StatusMap::new(),
        });
    }
    events.push(Event::EnemySplit);
    // Prevent the EnemyDied/Victory check from firing for the split slime
    // (handled: the split slime is removed; new ones are at > 0 HP)
}
```

Also add `Event::EnemySplit` to the `Event` enum and `describe_event` in `engine.rs`.

Adjust the existing `EnemyDied` / `Victory` check at lines 347–353: the current check `if state.enemies[actual_target].hp <= Hp(0)` may fire stale if the split has already replaced the slime. Guard it: only check if `actual_target < state.enemies.len()` and the enemy at that index is the same one (or simpler: skip the die check if a split just happened).

### Step 6 — Four slime files
**Files:** `enemies/large_spike_slime.rs`, `enemies/medium_spike_slime.rs`, `enemies/large_acid_slime.rs`, `enemies/medium_acid_slime.rs`

Each has `DEF`, `next_move`, and `on_player_attack_damage` (returning split reaction when at or below 50% HP).

### Step 7 — Wire into `EnemyKind` dispatchers
Add `LargeSpike`, `MediumSpike`, `LargeAcid`, `MediumAcid` to:
- `def()`, `id()`, `from_id()`, `next_move()`, `on_player_attack_damage()`
- `enemy_icon()` in `engine.rs`

### Step 8 — Tests
**Core tests (enemies/mod.rs):**
- HP values
- First move for each slime type
- Alternating move pattern
- `on_player_attack_damage` returns split when HP ≤ 50%
- `on_player_attack_damage` returns None when HP > 50%

**Combat tests (combat.rs):**
- Dealing damage that pushes a large slime below 50% replaces it with 2 medium slimes
- New slimes start at full HP with their first-turn move
- `Event::EnemySplit` is emitted
- `Event::EnemyDied` is NOT emitted for the split slime
- Dealing damage to 0 HP (killing in one hit below 50%) still triggers split not death
- Killing a medium slime below 50% spawns 2 small slimes
- Victory fires only when ALL slimes (including spawned ones) are dead

### Step 9 — Snapshot test
`tests/scripts/08-large-slime-splits.slay` — spawns a large spike slime, attacks it below 50%, verify two medium slimes appear.

---

## Key Design Decisions

**Split on damage, not end of turn.** Simpler to implement, and functionally nearly equivalent. Real STS fires at end of slime turn, which is a minor timing difference.

**New slimes at full HP.** Real STS behaviour — the spawns are independent enemies, not sharing the split HP.

**Split fires even at exactly 0 HP.** If the attack would kill the slime outright, the split still fires (2 new medium slimes appear). This matches STS. The attacker doesn't get the satisfaction of a single kill — they must fight the smaller slimes too. The only exception is if the SPAWN kinds are themselves small slimes (already terminal — no split on small slimes).

**`actual_target` stability.** After a split removes the target and inserts 2, indices shift. The code after the split block must not re-index `actual_target` without checking. The safest approach: after a split, `return Ok((state, events))` immediately (the split slimes are fresh targets for future turns, not for the current card's follow-up logic).

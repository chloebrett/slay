# Gremlin Nob — Implementation Plan

## Source
https://slay-the-spire.fandom.com/wiki/Gremlin_Nob

## Enemy Stats
- **HP:** 82–86
- **Type:** Elite (Act 1)
- **Signature mechanic:** Enrage — whenever the player plays a Skill card, gains X Strength (where X = Enrage stacks)

## Moves
| Move       | Effect                                   | When used            |
|------------|------------------------------------------|----------------------|
| Bellow     | Gain 2 Enrage stacks                     | Always first turn    |
| Skull Bash | Deal 6 damage, apply 2 Vulnerable        | 33% after turn 1     |
| Bull Rush  | Deal 14 damage                           | 67% after turn 1     |

## Enrage Status
- Stacks accumulate (Bellow adds 2; can stack repeatedly if Nob somehow bellows again)
- Each time the player plays a **Skill** card during combat, every enemy with Enrage stacks gains that many Strength immediately
- This requires a new hook in `combat.rs` after the `skills_this_turn += 1` line

---

## Step-by-step Goals

### Step 1 — `StatusEffect::Enrage`
**File:** `crates/slay-core/src/status.rs`

Add `Enrage` to the `StatusEffect` enum. No decay logic needed (it's permanent for the fight).

Tests: display name, is_debuff returns false (it's a buff on the enemy).

---

### Step 2 — New moves: `NobBellow`, `SkullBash`, `BullRush`
**File:** `crates/slay-core/src/enemies/mod.rs`

Add three variants to the `Move` enum:
```
NobBellow,   // gains 2 Enrage
SkullBash,   // 6 dmg + 2 Vulnerable to player
BullRush,    // 14 dmg
```

Add `def()` entries:
- `NobBellow` → `GainStatus(Enrage, 2)` — note: this only queues the status; the actual "gain Strength on skill" behaviour lives in the combat hook, not the move itself
- `SkullBash` → `DealDamage(6)` + `ApplyStatus(Vulnerable, 2)`
- `BullRush` → `DealDamage(14)`

Update `intent()` accordingly:
- `NobBellow` → `Intent::Buff`
- `SkullBash` → `Intent::AttackDebuff` (new variant needed — see Step 2b)
- `BullRush` → `Intent::Attack(14)`

**Step 2b — `Intent::AttackDebuff`**

Skull Bash both deals damage AND applies a debuff, which the current `Intent` enum can't express. Add:
```rust
AttackDebuff(i32),   // damage + debuffs player
```
Update `intent()` calculation and `describe_intent` in `engine.rs`.

---

### Step 3 — `gremlin_nob.rs`
**File:** `crates/slay-core/src/enemies/gremlin_nob.rs`

```rust
pub const DEF: EnemyDef = EnemyDef { name: "Gremlin Nob", max_hp: Hp(84) };
// HP 84 = midpoint of 82–86 range

pub fn next_move(last: Option<Move>, rng: &mut impl Rng) -> Move {
    // First turn always Bellow
    let Some(last) = last else { return Move::NobBellow };
    // After that: 33% SkullBash, 67% BullRush — cannot repeat last
    // ...
}
```

Move pattern: after Bellow, pick from [SkullBash (×1), BullRush (×2)], excluding last move.

Tests in `enemies/mod.rs`:
- First move is `NobBellow`
- After Bellow: can be either SkullBash or BullRush
- Never repeats last move
- `NobBellow` intent is Buff
- `SkullBash` intent is `AttackDebuff(6)`
- `BullRush` intent is `Attack(14)`
- HP is 84

---

### Step 4 — Wire up `EnemyKind::GremlinNob`
**File:** `crates/slay-core/src/enemies/mod.rs`

- Add `GremlinNob` variant
- `def()`, `name()`, `max_hp()`, `id()` → `"gremlin-nob"`, `from_id()`
- `next_move()` dispatch
- `on_player_attack_damage()` — no special reaction, return `None`

---

### Step 5 — Enrage combat hook
**File:** `crates/slay-core/src/combat.rs`

After the line `state.skills_this_turn += 1` (currently ~line 325), add:

```rust
// Enrage: each enemy with Enrage stacks gains Strength
for enemy in &mut state.enemies {
    let enrage = get_stacks(&enemy.statuses, StatusEffect::Enrage);
    if enrage > 0 {
        *enemy.statuses.entry(StatusEffect::Strength).or_insert(0) += enrage;
        events.push(Event::StatusApplied { ... });
    }
}
```

Tests (in `combat.rs`):
- Playing a Skill card against a Nob with 2 Enrage stacks increases enemy Strength by 2
- Playing an Attack card does NOT trigger Enrage
- Playing a Power card does NOT trigger Enrage
- Enrage stacks: playing two Skills with 2 Enrage gives +4 total Strength (+2 per Skill)

---

### Step 6 — Status display
**File:** `crates/slay-tui/src/engine.rs`

Add `StatusEffect::Enrage` to `status_display()`:
```rust
StatusEffect::Enrage => ("⚡", "Enrage"),
```

---

### Step 7 — Map: `MapNode::Elite`
**File:** `crates/slay-core/src/run.rs`

Add a new `MapNode::Elite(Vec<EnemyKind>)` variant, parallel to `Combat` but distinct so the TUI can:
- Show a different icon (⚔️⭐ or similar elite marker)
- Grant a relic reward on victory (elites always drop a relic in real STS)

In `generate_map()`, place a Gremlin Nob elite at a fixed floor (floor 6, after the Treasure room, before Act 1's rest site pattern). In real STS the elite floors are floors 6–8 (0-indexed). For now, one guaranteed elite node at floor 6.

Handle `MapNode::Elite` in `apply` just like `MapNode::Combat` but set `is_boss = false` and add a relic reward after the fight (similar to how `MapNode::Treasure` gives a relic). 

For the relic reward: after an elite combat victory, return `GameState::TreasureRoom` with a random relic. (Real STS gives a `RelicReward` state, but reusing `TreasureRoom` is the simplest extension of the current model.)

Tests:
- `MapNode::Elite` node starts combat with `is_boss = false`
- After elite victory → transitions to TreasureRoom (relic reward)

---

### Step 8 — TUI icon for Elite
**File:** `crates/slay-tui/src/engine.rs` (in `map_node_icon` / `map_node_name`)

```rust
MapNode::Elite(_) => "💀",   // or "⚡" to distinguish from Boss
```

Use `👿` to distinguish from the Boss `💀`. Update `map_node_name` to return `"Elite"`.

---

### Step 9 — Snapshot test
**File:** `crates/slay-tui/tests/scripts/07-gremlin-nob-enrage.slay`

Script that:
1. Spawns a Gremlin Nob
2. Ends turn (Nob bellows → gains 2 Enrage)
3. Plays a Skill card (Defend)
4. Verifies Nob gained Strength

Run: `INSTA_UPDATE=new cargo test -p slay-tui --test scripts`

---

## Out of scope for this plan
- Ascension scaling (Bellow gives 3 Enrage at Ascension 18, HP 85–90 at Ascension 8)
- Multiple elite variants (Lagavulin, Sentries)
- The Colosseum event (Nob + Taskmaster)
- Relic pool for elite drops (currently uses the same random relic logic as Treasure)

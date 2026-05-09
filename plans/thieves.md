# Thieves: Looter & Mugger

## Enemies

| Enemy  | HP | Mug dmg | Lunge dmg | Smoke Block |
|--------|----|---------|-----------|-------------|
| Looter | 44 | 10      | 12        | 6           |
| Mugger | 48 | 16      | 20        | 11          |

Move pattern (fixed, 5-move sequence — never cycles, thief is gone after move 5):
- Move 1: Mug
- Move 2: Lunge
- Move 3: Mug
- Move 4: Smoke Bomb (gain block, signals imminent flee)
- Move 5: Flee (actually escapes)

---

## Architecture

### 1. `Enemy` struct — add `stolen_gold: i32`

Tracks how much this enemy has stolen. Returned to player on death; kept on flee.

```rust
pub struct Enemy {
    // ... existing fields ...
    pub stolen_gold: i32,
}
```

Default: `0`. Set to `0` in all existing `Enemy { ... }` construction sites.

---

### 2. Mug gold stealing — thief-specific, not a general `Effect`

Only thieves steal gold, so this is handled as special-case logic in `EndEnemyTurn` when the move is a Mug move, rather than a generic `Effect` variant. Keeps `Effect` enum clean.

When a Mug move is processed:
- `let stolen = amount.min(state.player.gold)` (gold can't go below zero)
- `state.player.gold -= stolen`
- `enemy.stolen_gold += stolen`
- Emit `Event::GoldStolen { amount: stolen }`

---

### 3. `Effect::EscapeCombat` — new effect variant

Only used by the Flee move (not Smoke Bomb). When processed in `EndEnemyTurn`:
- Remove enemy from `state.enemies` (keep its `stolen_gold` — it escapes with it)
- Emit `Event::EnemyFled`
- Do NOT trigger `CombatPhase::Victory` here; let the normal "all enemies dead?" check handle it

Smoke Bomb's effects: `[Effect::GainBlock(6)]` — no escape yet, just block.  
Flee's effects: `[Effect::EscapeCombat]`

---

### 4. Return stolen gold on enemy death

In the existing enemy-death handling (where `hp <= 0`), before removing the enemy:
- `state.player.gold += enemy.stolen_gold`
- If `enemy.stolen_gold > 0`, emit `Event::GoldReturned { amount: enemy.stolen_gold }`

---

### 5. `Intent::EscapeBlock(i32)` — new intent variant

Smoke Bomb communicates both block gain and imminent escape.  
`intent()` returns this for Smoke Bomb moves.  
TUI renders as e.g. `"🛡️6 Flee"`.

Alternatively reuse `Intent::Defend` if we don't need to signal the flee in the UI — but a distinct variant is more informative.

---

### 6. New moves

```
LooterMug        — effects: [DealDamage(10)]  + special mug steal logic (10 gold)
LooterLunge      — effects: [DealDamage(12)]
LooterSmokeBomb  — effects: [GainBlock(6)]
LooterFlee       — effects: [EscapeCombat]

MuggerMug        — effects: [DealDamage(16)]  + special mug steal logic (16 gold)
MuggerLunge      — effects: [DealDamage(20)]
MuggerSmokeBomb  — effects: [GainBlock(11)]
MuggerFlee       — effects: [EscapeCombat]
```

A `is_mug_move(move_)` helper (or match arm) identifies which moves trigger gold stealing.

---

### 7. New enemy modules

`crates/slay-core/src/enemies/looter.rs`  
`crates/slay-core/src/enemies/mugger.rs`

```rust
pub fn next_move(history: &[Move]) -> Move {
    match history.len() {
        0 => Move::LooterMug,
        1 => Move::LooterLunge,
        2 => Move::LooterMug,
        3 => Move::LooterSmokeBomb,
        _ => Move::LooterFlee,   // move 5 — actually escapes
    }
}
```

No modular arithmetic — this is a one-way sequence; the thief never survives past Flee.

---

### 8. `Event::GoldStolen` / `Event::GoldReturned`

```rust
Event::GoldStolen  { amount: i32 }
Event::GoldReturned { amount: i32 }
Event::EnemyFled
```

---

### 9. TUI updates (`engine.rs`)

- `enemy_icon`: Looter = `"🗡️"`, Mugger = `"🔪"` (or similar)
- `describe_intent`: `Intent::EscapeBlock(b) => format!("🛡️{b} Flee")`
- `describe_event`: `EnemyFled => "Enemy fled!"`, `GoldStolen { amount } => format!("Thief stole {amount} gold!")`, `GoldReturned { amount } => format!("Recovered {amount} gold!"`

---

## Files to create

- `crates/slay-core/src/enemies/looter.rs`
- `crates/slay-core/src/enemies/mugger.rs`

## Files to modify

- `crates/slay-core/src/combat.rs` — `Enemy` struct, effect processing, death handling, victory check
- `crates/slay-core/src/enemies/mod.rs` — new kinds, moves, effects, intent, dispatch tables, tests
- `crates/slay-tui/src/engine.rs` — icons, intent/event descriptions

---

## TDD Order

1. `Enemy.stolen_gold` field (unit test: default is 0)
2. Mug move reduces player gold by `min(amount, player.gold)`, increments `stolen_gold`, emits `GoldStolen`
3. Gold cannot go below zero (player has less gold than mug amount)
4. Stolen gold returned on enemy kill (`GoldReturned` event)
5. `Effect::EscapeCombat` removes enemy, stolen gold NOT returned, emits `EnemyFled`
6. Combat continues (no Victory) if other enemies remain after flee
7. Combat ends in Victory if fleeing enemy was the last one
8. `Intent::EscapeBlock` for Smoke Bomb; `Intent::Escape` (or similar) for Flee
9. Looter/Mugger defs, move sequences, move effects
10. TUI rendering

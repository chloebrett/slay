# Hexaghost

## Enemy stats

- HP: 250
- Boss: Act 1

## Move sequence

Fixed, deterministic:

| Step | Move |
|------|------|
| 0 | Activate (nothing) |
| 1 | Divider (HP-scaled damage) |
| 2, 9, 16… | Sear |
| 3, 10, 17… | Tackle |
| 4, 11, 18… | Sear |
| 5, 12, 19… | Inflame |
| 6, 13, 20… | Tackle |
| 7, 14, 21… | Sear |
| 8, 15, 22… | Inferno |

Formula: index 0 = Activate, index 1 = Divider, then `(index - 2) % 7` for the repeating block.

## Move effects (base, no Ascension)

| Move | Effect |
|------|--------|
| Activate | Nothing |
| Divider | `(floor(player_hp / 12) + 1) × 6` damage |
| Sear | 6 damage + add 1 Burn to discard |
| Tackle | 5 × 2 damage |
| Inflame | Gain 12 block + gain 2 Strength |
| Inferno | 2 × 6 damage + add 3 Burns to discard + upgrade all Burns |
| Sear (post-Inferno) | 6 damage + add 1 Burn+ to discard |

---

## Architecture

### 1. `Card::BurnPlus` — new card variant

Upgraded Burn. Deals 4 damage at end of turn (vs 2 for Burn).

```rust
// burn_plus.rs
EndOfTurnHook::BlockableDamage(4)
```

Add to all card dispatch tables (`def`, `id`, `from_id`, `end_of_turn_hook`, `exhausts`, `grade`, `is_playable` exclusion, `apply`).

---

### 2. `Effect::DividerDamage` — new effect variant

Damage that scales from player's current HP at time of execution. Cannot be encoded as a fixed `DealDamage(n)`.

Handled specially in `EndEnemyTurn`:
```rust
Effect::DividerDamage => {
    let n = (state.player.hp.0 / 12 + 1) * 6;
    // deal n damage (blockable, status-modified like normal attack)
}
```

`intent()` for Divider: returns `Intent::Attack(?)` — we don't know the exact value until combat. Use a new `Intent::AttackScaled` variant, or reuse `Intent::Attack(0)` as a sentinel. **Proposed: `Intent::AttackUnknown`** — TUI shows "⚔️ ?".

---

### 3. `Effect::UpgradeAllBurns` — new effect variant

When processed in `EndEnemyTurn`, iterates `hand`, `draw_pile`, and `discard_pile`, replacing every `Card::Burn` with `Card::BurnPlus`.

---

### 4. Post-Inferno Sear — two Move variants

`next_move` checks `history.contains(&Move::HexaghostInferno)` to decide which Sear to return. This avoids needing combat state access in `next_move`.

- `HexaghostSear` → `[DealDamage(6), AddToDiscard(Card::Burn)]`
- `HexaghostSearUpgraded` → `[DealDamage(6), AddToDiscard(Card::BurnPlus)]`

---

### 5. New moves

```
HexaghostActivate     — effects: []
HexaghostDivider      — effects: [DividerDamage]
HexaghostSear         — effects: [DealDamage(6), AddToDiscard(Card::Burn)]
HexaghostSearUpgraded — effects: [DealDamage(6), AddToDiscard(Card::BurnPlus)]
HexaghostTackle       — effects: [DealDamage(2), DealDamage(2), DealDamage(2), DealDamage(2), DealDamage(2)]
HexaghostInflame      — effects: [GainBlock(12), GainStatus(Strength, 2)]
HexaghostInferno      — effects: [DealDamage(6), DealDamage(6), AddToDiscard(Card::Burn), AddToDiscard(Card::Burn), AddToDiscard(Card::Burn), UpgradeAllBurns]
```

---

### 6. Intent variants

- `Intent::AttackUnknown` — for Divider (damage can't be computed without player HP)
- `HexaghostActivate` → `Intent::Buff` (does nothing, same as Lagavulin Sleep)
- `HexaghostInflame` → early return `Intent::AttackDefend` or `Intent::Buff`? It gains block + strength — `Intent::Buff` (since it doesn't attack)

Wait — Inflame gives SELF block + SELF strength. That's `Buff` territory but visually should hint at both. Use existing `Intent::Buff`.

Actually, Inflame has GainBlock for self which already falls under existing intent logic (returns `Intent::Defend(12)` because block > 0, buffs_self = true → currently maps to `Intent::Defend` since `(_, b, false, false)` isn't quite right — `buffs_self = true` here). Need to check. Add early return if needed: `if matches!(self, HexaghostInflame) { return Intent::Buff; }` — it signals preparation, not defence.

---

### 7. `next_move` implementation

```rust
pub fn next_move(history: &[Move]) -> Move {
    let already_infernoed = history.contains(&Move::HexaghostInferno);
    let sear = if already_infernoed { Move::HexaghostSearUpgraded } else { Move::HexaghostSear };
    match history.len() {
        0 => Move::HexaghostActivate,
        1 => Move::HexaghostDivider,
        n => match (n - 2) % 7 {
            0 => sear,
            1 => Move::HexaghostTackle,
            2 => sear,
            3 => Move::HexaghostInflame,
            4 => Move::HexaghostTackle,
            5 => sear,
            _ => Move::HexaghostInferno,
        }
    }
}
```

---

### 8. TUI updates (`engine.rs`)

- `enemy_icon`: `SlimeBoss => "👻"`
- `describe_intent`: `Intent::AttackUnknown => "⚔️ ?".into()`
- Divider's unknown damage shows as `"⚔️ ?"` to the player

---

## Files to create

- `crates/slay-core/src/enemies/hexaghost.rs`
- `crates/slay-core/src/cards/burn_plus.rs`

## Files to modify

- `crates/slay-core/src/enemies/mod.rs` — new kinds, moves, effects, intents, dispatch
- `crates/slay-core/src/cards/mod.rs` — `Card::BurnPlus`, all dispatch tables
- `crates/slay-core/src/combat.rs` — `Effect::DividerDamage`, `Effect::UpgradeAllBurns`, EndEnemyTurn handlers
- `crates/slay-tui/src/engine.rs` — icon, `AttackUnknown` intent, event descriptions

---

## TDD order

1. `Card::BurnPlus` — def, end-of-turn hook (4 dmg), not playable, not in starter deck
2. `Effect::UpgradeAllBurns` — replaces Burn with BurnPlus in all card zones
3. `Effect::DividerDamage` — damage = `(player_hp / 12 + 1) * 6`
4. `Intent::AttackUnknown` for Divider
5. Hexaghost defs, move sequence, move effects
6. Inflame intent early return
7. Post-Inferno Sear gives BurnPlus
8. TUI rendering

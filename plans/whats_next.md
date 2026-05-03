# What's Next

## Up Next: QoL Bug Fixes

### 1. Enemy intent shows Strength-modified damage

**Problem:** Intent display shows raw base damage, ignoring the enemy's Strength bonus. A Cultist with 3 Ritual stacks shows "Attack 6" but deals 9.

**Fix:** Add `Enemy::effective_intent(player_statuses: &StatusMap) -> Intent` in `combat.rs`. Calls `move_.intent()` then adjusts Attack/AttackDefend damage via `resolve_damage(base, &self.statuses, player_statuses)`. Update `game.rs` `render_combat` to call this instead of `enemy.move_.intent()`.

Files: `crates/slay-core/src/combat.rs`, `crates/slay-tui/src/game.rs`

### 2. Raw error enum names shown to player

`Error: InvalidPhase`, `Error: NotEnoughEnergy`, etc. are printed directly. Should be friendly messages like "Can't do that right now." / "Not enough energy."

### 3. No pile counts in combat HUD

The hand display doesn't show how many cards are in the draw or discard piles. Should show e.g. `Draw: 7  Discard: 3` in the combat header.

### 4. "Unknown command" on empty input

Pressing Enter with no input prints "Unknown command." Should silently re-prompt.

---

## Directions Beyond QoL

| Direction | Effort | Payoff |
|---|---|---|
| More cards | Medium | High (more interesting decisions) |
| More relics (Tier 4) | Medium | High (relics make runs feel different) |
| More enemies | Medium | Medium (adds variety) |
| Ratatui TUI | High | Medium (cosmetic, but much nicer) |
| Branching map | High | High (core roguelike feel) |
| Potions | Medium | Medium |
| Shop / Merchant | High | Medium |
| Events | High | Low (mostly text content) |

### More cards
`plans/ironclad_cards.json` has the full Ironclad set. Remaining cards likely include multi-hit attacks, self-mill, exhaust-matters cards, and powers.

### More relics
`plans/relic-tiers.md` maps 88 relics. Remaining tiers need:
- **Tier 4**: card-play counters (Nunchaku, Kunai, Shuriken) — need a counter on `CombatState`
- **Tier 5**: HP-reactive (Lizard Tail, Red Skull) — need an HP-change hook
- **Tier 6**: new status types (Thorns, Plating, Vigor)

### More enemies
`plans/monsters.md` has the plan. Needs new `Intent` variants, Ritual/Dexterity status types, and probabilistic intent RNG.

### Branching map
Biggest architectural change — graph replacing `MAP_NODES: &[MapNode]`. Node types: Combat, Elite, Rest, Merchant, Event, Boss. Player chooses path at each branch.

### Ratatui TUI
`run_game` already accepts `impl Write` so the seam is ready. Replace println-based rendering with persistent panels, HP bars, colour-coded hand.

### Potions
`Vec<Potion>` on Player, `Command::UsePotion(usize)`, dropped after combat.

### Merchant
`MapNode::Merchant`, `GameState::Shop` — buy/remove cards, buy relics/potions.

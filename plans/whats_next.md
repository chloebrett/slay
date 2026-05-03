# What's Next

## Recently shipped

- ✅ QoL bug fixes (enemy intent shows Strength, friendly errors, pile counts in HUD, empty input silently re-prompts)
- ✅ Ratatui interactive UI (`tui::run_tui`) with auto-fallback to plain text via `--plain` / piped stdin / `--script`

## Directions

| Direction | Effort | Payoff |
|---|---|---|
| More cards | Medium | High (more interesting decisions) |
| More relics (Tier 4) | Medium | High (relics make runs feel different) |
| More enemies | Medium | Medium (adds variety) |
| Branching map | High | High (core roguelike feel) |
| Potions | Medium | Medium |
| Shop / Merchant | High | Medium |
| Events | High | Low (mostly text content) |
| TUI polish | Low | Medium (HP bars, colour-coded statuses) |

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

### TUI polish
The ratatui UI works but has room to grow:
- Inline HP bar widget for the player (top bar) and per-enemy
- Colour-coded statuses (red Vulnerable, yellow Weak, green Poison)
- Card cost coloured by affordability (red border for unaffordable)
- Animations: brief flash on damage taken/dealt
- Mouse support: click a card to play, click an enemy to target

### Potions
`Vec<Potion>` on Player, `Command::UsePotion(usize)`, dropped after combat.

### Merchant
`MapNode::Merchant`, `GameState::Shop` — buy/remove cards, buy relics/potions.

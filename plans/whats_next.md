# What's Next

## Recently shipped

- ✅ QoL bug fixes (enemy intent shows Strength, friendly errors, pile counts in HUD, empty input silently re-prompts)
- ✅ Ratatui interactive UI (`tui::run_tui`) with auto-fallback to plain text via `--plain` / piped stdin / `--script`
- ✅ Cultist (Incantation → Ritual, Dark Strike loop)
- ✅ Jaw Worm (Chomp/Bellow/Thrash with probabilistic RNG)
- ✅ Small Spike Slime (Flame Tackle + Dazed to discard)
- ✅ Red Louse (Bite/Grow probabilistic, no-repeat)
- ✅ Green Louse, Small Acid Slime, Blue Slaver, Red Slaver (+ Entangle status)
- ✅ Potions (9 types: Fire, Explosive, Block, Strength, Swift, Fear, Weak, Blood, Energy)
- ✅ Tier 4 relics: Nunchaku, OrnamentalFan, Kunai, Shuriken, Kusarigama, LetterOpener, TuningFork, GremlinHorn, Pocketwatch

## Directions

| Direction            | Effort | Payoff                                  |
| -------------------- | ------ | --------------------------------------- |
| More cards           | Medium | High (more interesting decisions)       |
| More relics (Tier 4) | Medium | High (relics make runs feel different)  |
| More enemies         | Medium | Medium (adds variety)                   |
| Branching map        | High   | High (core roguelike feel)              |
| Potions              | Medium | Medium                                  |
| Shop / Merchant      | High   | Medium                                  |
| Events               | High   | Low (mostly text content)               |
| TUI polish           | Low    | Medium (HP bars, colour-coded statuses) |

### More cards

`plans/ironclad_cards.json` has the full Ironclad set. Remaining cards likely include multi-hit attacks, self-mill, exhaust-matters cards, and powers.

### More relics

`plans/relic-tiers.md` maps 88 relics. Remaining tiers need:

- ✅ **Tier 4**: card-play counters (Nunchaku, OrnamentalFan, Kunai, Shuriken, Kusarigama, LetterOpener, TuningFork, GremlinHorn, Pocketwatch) — `attacks_this_turn/combat`, `skills_this_turn/combat`, `cards_played_this_turn`, `extra_draws_next_turn` on `CombatState`
- **Tier 5**: HP-reactive (Lizard Tail, Red Skull) — need an HP-change hook
- **Tier 6**: new status types (Thorns, Plating, Vigor)

### More enemies

`plans/monsters.md` has the updated plan. 6 enemies are already live (Louse, Fungibeast, Cultist, Jaw Worm, Small Spike Slime, Red Louse). The next four require little or no new infrastructure:

1. **Green Louse** — Bite / Spit Web (2 Weak), zero new infra
2. **Small Acid Slime** — Tackle / Lick (1 Weak), zero new infra
3. **Blue Slaver** — Stab / Rake (7 dmg + Weak), probabilistic only
4. **Red Slaver** — Stab / Scrape (Vuln) / Entangle (can't play Attacks this turn); needs `StatusEffect::Entangle`

### Branching map

Biggest architectural change — graph replacing `MAP_NODES: &[MapNode]`. Node types: Combat, Elite, Rest, Merchant, Event, Boss. Player chooses path at each branch.

### TUI polish

The ratatui UI works but has room to grow:

- Inline HP bar widget for the player (top bar) and per-enemy
- Colour-coded statuses (red Vulnerable, yellow Weak, green Poison)
- Card cost coloured by affordability (red border for unaffordable)
- Animations: brief flash on damage taken/dealt
- Mouse support: click a card to play, click an enemy to target

### Potions ✅

9 potions live. `Player.potions: Vec<Potion>` (max 3). `Command::UsePotion(slot, target)` during combat. `Command::AddPotion` for debug/rewards. Potions persist between floors. `Command::DiscardPotion(slot)` works in all states (Map, Combat, RestSite, CardReward). When slots are full on victory, potion is stored as `CardRewardState.offered_potion` — player can `discard N` to make room and auto-collect it, or skip to lose the offer.

### Merchant

`MapNode::Merchant`, `GameState::Shop` — buy/remove cards, buy relics/potions.

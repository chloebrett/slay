# Plan: Ironclad Powers

**Status**: Active

## Reference

`plans/ironclad_cards.json` — full card list

## Already Implemented

| Card | What it does |
|------|-------------|
| Inflame | Gain 2/3 Strength on play — uses `StatusEffect::Strength` directly, no new hooks needed |

## Full Power Survey

### A. Start-of-turn power effects
*Need a `StartPlayerTurn` phase that fires registered power effects before the player acts.*

| Card | Cost | Base / Plus | Effect |
|------|------|------------|--------|
| Demon Form | 3 | 2 / 3 Str | At start of turn, gain N Strength |
| Crimson Mantle | 1 | 8 / 12 Block | At start of turn, lose 1 HP and gain N Block |
| Pyre | 2 | 1 / 1 Energy | At start of turn, gain 1 Energy |
| Barricade | 3 | — | Block is not removed at start of turn *(flag: skip the block clear)* |
| Aggression | 1 | — | At start of turn, put a random Attack from Discard into Hand upgraded |
| Drum of Battle | 0 | — | Draw 2 cards now. At start of turn, Exhaust the top of your Draw Pile |
| Inferno | 1 | 6 / 9 dmg | At start of turn, lose 1 HP. Whenever you lose HP on your turn, deal N damage to all enemies |

### B. On-exhaust trigger
*Need an `exhaust_card(card, state, events, rng)` helper that fires hooks; replaces bare `exhaust_pile.push()` throughout `combat.rs` and card impls.*

| Card | Cost | Base / Plus | Effect |
|------|------|------------|--------|
| Feel No Pain | 1 | 3 / 4 Block | Whenever a card is Exhausted, gain N Block |
| Dark Embrace | 2 | — | Whenever a card is Exhausted, draw 1 card |

### C. Centralised block gain
*Need `gain_player_block(state, events, amount, rng)` helper that replaces scattered `block.0 +=` in card impls.*

| Card | Cost | Base / Plus | Effect |
|------|------|------------|--------|
| Juggernaut | 2 | 5 / 8 dmg | Whenever you gain Block, deal N damage to a random enemy |
| Unmovable | 2 | — | First time you gain Block from a card each turn, double the amount gained |

### D. On-HP-loss-during-player-turn trigger
*Need to distinguish card-driven HP loss from other sources. A `damage_player_from_card(state, events)` wrapper that also fires power hooks.*

| Card | Cost | Base / Plus | Effect |
|------|------|------------|--------|
| Rupture | 1 | 1 / 2 Str | Whenever you lose HP on your turn, gain N Strength |
| Inferno | 1 | — | *(also needs D — fires damage-to-all-enemies on HP loss)* |

### E. On-attack-played (3rd attack)
*`attacks_this_turn` already tracked; check it in PlayCard and add card to hand.*

| Card | Cost | Effect |
|------|------|--------|
| Juggling | 1 | Add a copy of the 3rd Attack you play each turn into your Hand |

### F. On-Vulnerable-applied trigger
*Hook into the `apply_status(Vulnerable, ...)` call.*

| Card | Cost | Effect |
|------|------|--------|
| Vicious | 1 | Whenever you apply Vulnerable, draw 1 card |

### G. On-draw trigger
*Hook into `draw_cards`.*

| Card | Cost | Effect |
|------|------|--------|
| Hellraiser | 2 | Whenever you draw a card containing "Strike", play it against a random enemy |

### H. End-of-turn power effects
*Separate from `EndOfTurnHook` (which is per-card-in-hand); need a power-level end-of-turn check using player statuses.*

| Card | Cost | Effect |
|------|------|--------|
| Stampede | 2 | At end of turn, play 1 random Attack from your Hand against a random enemy |

### I. Passive damage formula modifiers
*Change `resolve_damage` or intercept it via player/enemy statuses.*

| Card | Cost | Effect |
|------|------|--------|
| Cruelty | 1 | Vulnerable enemies take an additional 25% damage (175% instead of 150%) |
| Tank | 1 | You take double damage from enemies |

### J. Plating status (retained block)
*New `StatusEffect::Plating`. Block that is not cleared at start of turn (similar to Barricade, but stack-based rather than a flag).*

| Card | Cost | Effect |
|------|------|--------|
| Stone Armor | 1 | Gain 4/6 Plating |

---

## Architecture: Four Core Hooks

### Hook 1 — StartPlayerTurn phase
- Add `CombatPhase::StartOfPlayerTurn`
- `EndEnemyTurn` transitions to `StartOfPlayerTurn` (instead of `PlayerTurn`) after executing enemy moves
- New `Command::StartPlayerTurn` handles: fire start-of-turn power effects (via player statuses), clear block (unless Barricade), restore energy, draw cards, emit `TurnStarted`
- `apply_and_drain` in `slay-tui` drains *both* `EnemyTurn` and `StartOfPlayerTurn` automatically

New StatusEffects needed for group A: `DemonForm`, `CrimsonMantle`, `Pyre`, `Barricade`, `Aggression`, `DrumOfBattle`, `Inferno`

### Hook 2 — `exhaust_card` helper
- Add `pub(crate) fn exhaust_card(card: Card, state: &mut CombatState, events: &mut Vec<Event>, rng: &mut impl Rng)`
- Pushes to `exhaust_pile`, emits `Event::CardExhausted { card }` (already exists), then checks `FeelNoPain` and `DarkEmbrace` statuses
- Replace all bare `exhaust_pile.push(card)` calls with this helper

New StatusEffects: `FeelNoPain`, `DarkEmbrace`

### Hook 3 — `gain_player_block` helper
- Add `pub(crate) fn gain_player_block(state: &mut CombatState, events: &mut Vec<Event>, amount: i32, rng: &mut impl Rng)`
- Applies `resolve_block` (Frail, Dexterity), adds to `player.block`, emits `Event::PlayerBlocked`, then checks `Juggernaut` and `Unmovable` statuses
- Card impls (Defend, IronWave, ShrugItOff, Impervious, etc.) call this instead of setting `block.0` directly

New StatusEffects: `Juggernaut`, `Unmovable`

### Hook 4 — `damage_player_from_card`
- Add `pub(crate) fn damage_player_from_card(state: &mut CombatState, events: &mut Vec<Event>, amount: i32)`
- Subtracts HP directly (bypasses block, like `damage_player`), emits `Event::PlayerSelfDamaged`, then fires `Rupture` and `Inferno` hooks
- Used by card impls and start-of-turn effects that deal self-damage on the player's turn

New StatusEffects: `Rupture`, `Inferno`

---

## New StatusEffect Variants (full list)

Power-state variants (set to 1 when power is played, or N for stacked powers):

```
DemonForm       — stacks = Strength gained per turn
CrimsonMantle   — stacks = 1 (flag; block amount is 8/12 hardcoded per grade)
Pyre            — stacks = 1 (flag)
Barricade       — stacks = 1 (flag)
Aggression      — stacks = 1 (flag)
DrumOfBattle    — stacks = 1 (flag)
Inferno         — stacks = damage dealt per HP loss
FeelNoPain      — stacks = block gained per exhaust
DarkEmbrace     — stacks = 1 (flag; draw 1 card)
Juggernaut      — stacks = damage dealt per block gain
Unmovable       — stacks = 1 (flag)
Rupture         — stacks = Strength gained per HP loss
Juggling        — stacks = 1 (flag)
Vicious         — stacks = 1 (flag)
Hellraiser      — stacks = 1 (flag)
Stampede        — stacks = 1 (flag)
Cruelty         — stacks = 1 (flag)
Tank            — stacks = 1 (flag)
StonePlating    — stacks = retained block (decrements differently than normal block)
```

---

## Steps

### Step 1: New StatusEffects + TUI display entries

Add all power-state variants to `StatusEffect` enum. Add display entries in `engine.rs::status_display`. No behaviour yet — just the data model.

**Done when**: compiles, no existing tests broken.

### Step 2: StartPlayerTurn phase (Hook 1)

Split current `EndEnemyTurn` into two phases. `EndEnemyTurn` → `StartOfPlayerTurn`. New `Command::StartPlayerTurn` does: fire start-of-turn effects, clear block (unless Barricade), restore energy, draw cards, emit TurnStarted. Update `apply_and_drain`.

**Done when**: all existing tests pass with the refactored flow. No power effects yet — just the plumbing.

### Step 3: `exhaust_card` helper (Hook 2)

Replace bare `exhaust_pile.push()` everywhere with `exhaust_card()`. Hook fires FeelNoPain/DarkEmbrace effects if those statuses are active.

**Done when**: no bare exhaust pushes remain; existing Pummel/Impervious/SeeingRed exhaust tests still pass.

### Step 4: `gain_player_block` helper (Hook 3)

Centralise player block gain. Update all card impls that directly set `player.block.0`. Fire Juggernaut/Unmovable hooks.

**Done when**: existing block-gain tests still pass; no direct `block.0 +=` in card impls.

### Step 5: `damage_player_from_card` (Hook 4)

Add tagged self-damage function. Wire Rupture and Inferno hooks.

**Done when**: existing self-damage tests pass; Hemokinesis/Bloodletting/other self-damage cards use the new helper.

### Steps 6+: Individual power cards

Once all hooks are in place, each power card is a small RED-GREEN-MUTATE cycle:
- Add Card variant, card module, wire up mod.rs
- Card `apply` just sets `StatusEffect::XYZ` on player
- The hook in combat.rs reads the status and fires the effect

---

*Delete this file when all powers are implemented.*

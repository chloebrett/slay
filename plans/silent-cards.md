# Silent Cards

Cards for the Silent character. ✅ = implemented in Rust.

## Basic (starter deck)

| Card | Type | Cost | Effect | Impl |
|------|------|------|--------|------|
| Strike | Attack | 1 | Deal 6 damage. | ✅ (neutral) |
| Defend | Skill | 1 | Gain 5 Block. | ✅ (neutral) |
| Neutralize | Attack | 0 | Deal 3 damage. Apply 1 Weak. | |
| Survivor | Skill | 1 | Gain 8 Block. Discard 1 card. | |

## Common

| Card | Type | Cost | Effect | Impl |
|------|------|------|--------|------|
| Acrobatics | Skill | 1 | Draw 3 cards. Discard 1 card. | |
| Backflip | Skill | 1 | Gain 5 Block. Draw 2 cards. | |
| Backstab | Attack | 0 | Innate. Deal 11 damage. Exhaust. | |
| Bane | Attack | 1 | Deal 7 damage. If Poisoned, deal 7 again. | |
| Blade Dance | Skill | 1 | Add 3 Shivs to your hand. | |
| Cloak and Dagger | Skill | 1 | Gain 6 Block. Add 1 Shiv to hand. | |
| Dagger Spray | Attack | 1 | Deal 4 damage twice to ALL enemies. | |
| Dagger Throw | Attack | 1 | Deal 9 damage. Draw 1. Discard 1. | |
| Deadly Poison | Skill | 1 | Apply 5 Poison. | ✅ |
| Deflect | Skill | 0 | Gain 4 Block. Exhaust. | |
| Dodge and Roll | Skill | 1 | Gain 4 Block. Next turn, gain 4 Block. | |
| Endless Agony | Attack | 0 | Deal 4 damage. Add a copy to hand. Exhaust. | |
| Escape Plan | Skill | 0 | Draw 1. If it's a Skill, gain 3 Block. | |
| Flying Knee | Attack | 1 | Deal 8 damage. Next turn, gain 1 energy. | |
| Outmaneuver | Skill | 1 | Next turn, gain 2 energy. | |
| Piercing Wail | Skill | 1 | ALL enemies lose 6 Strength this turn. Exhaust. | |
| Poisoned Stab | Attack | 1 | Deal 6 damage. Apply 3 Poison. | |
| Prepared | Skill | 0 | Draw 1. Discard 1. | |
| Quick Slash | Attack | 1 | Deal 8 damage. Draw 1. | |
| Slice | Attack | 0 | Deal 6 damage. | |
| Sneaky Strike | Attack | 2 | Deal 12 damage. If you discarded a card this turn, gain 2 energy. | |
| Sucker Punch | Attack | 1 | Deal 7 damage. Apply 1 Weak. | |

## Uncommon

| Card | Type | Cost | Effect | Impl |
|------|------|------|--------|------|
| Accuracy | Power | 1 | Shivs deal 4 additional damage. | |
| Adrenaline | Skill | 0 | Gain 1 energy. Draw 2. Exhaust. | |
| After Image | Power | 1 | Whenever you play a card, gain 1 Block. | |
| Blur | Skill | 1 | Gain 5 Block. Block is not removed at end of turn. | |
| Bouncing Flask | Skill | 2 | Apply 3 Poison to a random enemy 3 times. | |
| Calculated Gamble | Skill | 0 | Discard your hand, then draw that many. Exhaust. | |
| Catalyst | Skill | 1 | Double target's Poison. Exhaust. | |
| Choke | Attack | 2 | Deal 12 damage. When enemy plays a card this turn, it loses 3 HP. | |
| Concentrate | Skill | 0 | Discard 3 cards. Gain 2 energy. Exhaust. | |
| Corpse Explosion | Skill | 2 | Apply 6 Poison. When target dies, deal 8 damage to ALL. | |
| Dash | Attack | 2 | Gain 10 Block. Deal 10 damage. | |
| Die Die Die | Attack | 1 | Deal 13 damage to ALL enemies. Exhaust. | |
| Distraction | Skill | 0 | Add a random Silent Skill from your class to hand. Exhaust. | |
| Expertise | Skill | 1 | Draw cards until you have 6 in hand. | |
| Finisher | Attack | 1 | Deal 6 damage for each Attack played this turn. | |
| Flechettes | Attack | 1 | Deal 5 damage for each Skill in your hand. | |
| Footwork | Power | 1 | Gain 2 Dexterity. | |
| Glass Knife | Attack | 1 | Deal 8 damage twice. Each time played, damage decreases by 4. | |
| Heel Hook | Attack | 1 | If enemy is Weak: deal 5 damage, gain 1 energy, draw 1. | |
| Infinite Blades | Power | 1 | At the start of your turn, add 1 Shiv to your hand. | |
| Leg Sweep | Skill | 2 | Apply 2 Weak. Gain 11 Block. | |
| Masterful Stab | Attack | 0 | Deal 12 damage. Costs 1 more per card in your discard pile. | |
| Noxious Fumes | Power | 1 | At the start of your turn, apply 2 Poison to ALL enemies. | |
| Predator | Attack | 2 | Deal 15 damage. Next turn, draw 2 extra cards. | |
| Riddle With Holes | Attack | 2 | Deal 3 damage 5 times. | |
| Setup | Skill | 1 | Exhaust a card in your hand. It costs 0 this combat. | |
| Storm of Steel | Skill | 1 | Discard your hand. Add 1 Shiv per discarded card. Exhaust. | |
| Tactician | Skill | 0 | Unplayable. When discarded, gain 1 energy. | |
| Terror | Skill | 1 | Apply 99 Vulnerable. Exhaust. | |
| Well-Laid Plans | Power | 1 | At end of turn, Retain up to 1 card. | |

## Rare

| Card | Type | Cost | Effect | Impl |
|------|------|------|--------|------|
| A Thousand Cuts | Power | 2 | Whenever you play a card, deal 1 damage to ALL. | |
| Alchemize | Skill | 1 | Obtain a random Potion. Exhaust. | |
| Bullet Time | Skill | 3 | You cannot draw this turn. Reduce cost of ALL cards in hand to 0. | |
| Burst | Skill | 1 | This turn, your next Skill is played twice. Exhaust. | |
| Doppelganger | Skill | X | Next turn, draw X extra. Gain X energy. Exhaust. | |
| Envenom | Power | 2 | Whenever an Attack deals unblocked damage, apply 1 Poison. | |
| Grand Finale | Attack | 0 | Can only be played if Draw pile is empty. Deal 50 damage to ALL. | |
| Malaise | Skill | X | Apply X Weak and X Poison to enemy. Exhaust. | |
| Nightmare | Skill | 2 | Choose a card. Next turn, add 3 copies to hand. Exhaust. | |
| Phantasmal Killer | Skill | 1 | This turn, your next Attack deals double damage. Exhaust. | |
| Tools of the Trade | Power | 1 | At the start of your turn, draw 1 and discard 1. | |
| Unload | Attack | 1 | Deal 14 damage. Discard all non-Attack cards in hand. | |

## Upgrade values

| Card | Base | Plus |
|------|------|------|
| Deadly Poison | Apply 5 Poison | Apply 7 Poison |
| Neutralize | 3 damage, 1 Weak | 3 damage, 2 Weak |
| Survivor | 8 Block, discard 1 | 11 Block, discard 1 |
| Slice | 6 damage | 9 damage |
| Sucker Punch | 7 dmg, 1 Weak | 9 dmg, 2 Weak |
| Poisoned Stab | 6 dmg, 3 Poison | 8 dmg, 3 Poison |
| Deflect | 4 Block | 7 Block |
| Prepared | Draw 1, discard 1 | Draw 2, discard 1 |
| Footwork | 2 Dexterity | 3 Dexterity |
| Noxious Fumes | 2 Poison/turn | 3 Poison/turn |
| After Image | on play: 1 Block | Innate + 1 Block |
| Envenom | 1 Poison | 2 Poison |
| Grand Finale | 50 damage | 60 damage |

## Notes on mechanics needed

These mechanisms don't yet exist in slay-core and must be added before implementing the cards:

- **Shiv** — generated attack card (0 cost, deal 4 damage), spawned by various Silent skills
- **Retain** — card persists in hand at end of turn (Well-Laid Plans, Blur)
- **Discard trigger** — events fired when a card is discarded (Tactician, Sneaky Strike)
- **Draw trigger** — events on draw (Escape Plan)
- **Energy carry-forward** — "next turn gain N energy" (Flying Knee, Outmaneuver)
- **Weak application to player** — already exists via Weak status; Piercing Wail applies to enemies only
- **Repeating Poison on death** — Corpse Explosion
- **Per-use state** — Glass Knife decreases damage each time it's played in a combat

## Implementation order

### Trivial — pure composition of existing mechanics

- **Neutralize** — deal 3 damage and apply 1 Weak; both exist (damage like Strike, Weak like Clothesline).
- **Slice** — deal 6 damage at 0 cost; same pattern as Swift Strike / every basic attack.
- **Sucker Punch** — deal 7 damage and apply 1 Weak; same dual-effect as Clothesline with different numbers.
- **Poisoned Stab** — deal 6 damage and apply 3 Poison; composes Strike-style damage with DeadlyPoison-style apply.
- **Deflect** — gain 4 block and exhaust; same as Impervious but a common card.
- **Backflip** — gain 5 block and draw 2; block exists, drawing 2 exists (BurningPact).
- **Quick Slash** — deal 8 damage and draw 1; same pattern as PommelStrike.
- **Footwork** — gain 2 Dexterity; identical to Inflame but applies Dexterity instead of Strength, both already exist as status effects.
- **Leg Sweep** — apply 2 Weak and gain 11 block; both exist, two separate `apply` calls.
- **Dash** — gain 10 block and deal 10 damage; same dual-effect structure as IronWave.
- **Die Die Die** — deal 13 damage to ALL enemies and exhaust; AoE and exhaust both exist (Cleave, Impervious).
- **Adrenaline** — gain 1 energy, draw 2, exhaust; all three exist (SeeingRed for energy, BurningPact for draw 2).

### Minor — one new mechanism required

- **Survivor** — gain 8 block and discard 1 chosen card; needs a card-selection prompt for the discard (random discard exists in TrueGrit, player-choice doesn't yet).
- **Prepared** — draw 1 and discard 1 chosen card; same selection mechanism as Survivor.
- **Dagger Throw** — deal 9 damage, draw 1, discard 1 chosen; same discard-selection mechanism.
- **Acrobatics** — draw 3 and discard 1 chosen card; same mechanism, just draws more first.
- **Bane** — deal 7 damage, and if the target has Poison deal 7 again; needs a conditional second hit that checks `get_stacks(statuses, Poison) > 0` at time of play.
- **Dagger Spray** — deal 4 damage twice to ALL enemies; needs AoE multi-hit (single-target multi-hit exists via Pummel/TwinStrike, AoE multi-hit does not).
- **Flying Knee** — deal 8 damage and gain 1 energy at the start of next turn; needs an "energy carry-forward" value in `CombatState` that is added during `StartPlayerTurn`.
- **Outmaneuver** — gain 2 energy next turn; same carry-forward mechanism as Flying Knee.
- **Sneaky Strike** — deal 12 damage; if a card was discarded this turn, gain 2 energy; needs a `discards_this_turn` counter in `CombatState`.
- **Endless Agony** — deal 4 damage, add a copy of this card to hand, exhaust; needs "copy this card into hand" — a new operation but simple to implement.
- **Blade Dance** — add 3 Shivs to hand; needs Shiv as a generated `Card` variant and an "add to hand" operation.
- **Cloak and Dagger** — gain 6 block and add 1 Shiv; gain block exists, Shiv needed (same as Blade Dance).
- **Escape Plan** — draw 1 card and if it's a Skill, gain 3 block; needs to inspect the type of the card just drawn, requiring the draw to yield the card back as an event or return value.
- **Bouncing Flask** — apply 3 Poison to a random enemy 3 times; needs random enemy selection distinct from the player's chosen target.
- **Predator** — deal 15 damage and draw 2 extra cards at the start of next turn; needs carry-forward extra draws, similar to the energy carry-forward for Flying Knee.
- **Piercing Wail** — all enemies lose 6 Strength this turn; needs a temporary per-turn Strength debuff for all enemies (same mechanism as Dark Shackles from neutral, but applied to every enemy).
- **Heel Hook** — deal 5 damage, gain 1 energy, draw 1, but only if the enemy is Weak; needs a conditional at play time checking `get_stacks(enemy.statuses, Weak) > 0`.
- **Calculated Gamble** — discard your entire hand then draw that many cards; needs discard-all-hand (counting cards first) then draw-N.
- **Expertise** — draw cards until you have 6 in hand; needs a draw loop with a hand-size check as the termination condition.
- **Finisher** — deal 6 damage for each Attack played this turn; needs an `attacks_played_this_turn` counter in `CombatState`.
- **Flechettes** — deal 5 damage for each Skill currently in hand; needs a hand scan counting `CardType::Skill` at time of play.
- **After Image** — whenever you play a card, gain 1 block; needs an on-play-card trigger registered by powers, similar to the existing `relic_on_card_played` hook but for powers.
- **A Thousand Cuts** — whenever you play a card, deal 1 damage to ALL enemies; same on-play trigger as After Image, but deals AoE damage instead.
- **Noxious Fumes** — at the start of each turn, apply 2 Poison to ALL enemies; needs a start-of-turn power trigger (a new hook in `StartPlayerTurn` that powers can register with).
- **Infinite Blades** — at the start of each turn, add 1 Shiv to hand; same start-of-turn trigger as Noxious Fumes, plus Shiv.
- **Tools of the Trade** — at start of turn draw 1 and discard 1; same start-of-turn trigger, plus the player-choice discard.
- **Catalyst** — double the target's Poison stacks; needs a "multiply existing status" operation (`statuses.entry(Poison).and_modify(|v| *v *= 2)`).
- **Blur** — gain block that is not removed at end of turn; needs block to be tagged as persistent, bypassing the `block = 0` reset in `EndPlayerTurn`.
- **Well-Laid Plans** — at end of turn, retain up to 1 card instead of discarding it; needs a Retain flag per card-in-hand so that `EndPlayerTurn` skips discarding it.
- **Masterful Stab** — deal 12 damage but costs 1 more energy per card in the discard pile; needs dynamic cost that re-evaluates at display/play time based on `discard_pile.len()`.
- **Grand Finale** — deal 50 damage to ALL if the draw pile is empty; needs a playability guard that checks `draw_pile.is_empty()` and blocks play otherwise.
- **Envenom** — whenever an Attack deals unblocked damage, apply 1 Poison to that enemy; needs an on-unblocked-damage event hook fired from the damage application path.

### Major — significant new architecture

- **Concentrate** — discard 3 chosen cards and gain 2 energy; needs a multi-card hand-selection prompt (choosing 3 distinct cards), distinct from the single-card discard-selection.
- **Tactician** — unplayable; when discarded from hand, gain 1 energy; needs an on-discard event that checks which card was discarded and fires card-specific effects.
- **Storm of Steel** — discard your entire hand and add 1 Shiv per card discarded; needs discard-all-hand (counting cards) + Shiv generation + the on-discard hook (or just counts before discarding).
- **Setup** — exhaust a chosen card in hand and permanently set its cost to 0 for this combat; needs per-card-instance cost mutation stored alongside the card in the deck data structure.
- **Accuracy** — Shivs deal 4 more damage; needs a per-card-type damage modifier applied during `resolve_damage`, looking up whether the card being played is a Shiv.
- **Choke** — deal 12 damage; whenever the enemy plays a card this turn, it loses 3 HP; needs an "enemy card played" event hook that fires during `apply_combat_command` for enemy moves.
- **Corpse Explosion** — apply 6 Poison; when the target dies, deal 8 damage to ALL other enemies; needs an on-kill trigger in the damage application path that fires additional AoE damage.
- **Glass Knife** — deal 8 damage twice; damage decreases by 4 each time this card is played this combat; needs mutable per-card-instance state (a damage counter) stored with the card.
- **Burst** — your next Skill is played twice this turn; needs a "pending double-play" flag on `CombatState` that the play-card path checks and clears after consuming it.
- **Phantasmal Killer** — your next Attack deals double damage this turn; same pending-modifier pattern as Burst.
- **Doppelganger** — X-cost; next turn draw X extra and gain X energy; needs X-cost on a Skill (only Whirlwind uses it on an Attack today) and two carry-forward values keyed on the X spent.
- **Malaise** — X-cost; apply X Weak and X Poison to target; needs X-cost on a Skill and X used as the status amount.
- **Nightmare** — choose a card; next turn add 3 copies of it to hand; needs a card-selection prompt, storage of the chosen card across the turn boundary, and card-spawning at `StartPlayerTurn`.
- **Bullet Time** — don't draw cards this turn; reduce cost of all cards currently in hand to 0; needs a "draw suppressed" flag for `EndPlayerTurn`/draw phase and a temporary per-card cost override for every card in hand.
- **Distraction** — add a random Silent Skill to hand; needs a Silent card pool filtered to `CardType::Skill`, which requires the class-tagging system on `Card`.
- **Alchemize** — obtain a random Potion; requires the potion system to be implemented.

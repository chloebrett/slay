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

## Implementation priority (Act 1 Silent)

Good first pass for a playable Silent run:

1. Neutralize, Survivor (starter cards — required)
2. Deadly Poison ✅
3. Poisoned Stab, Slice, Sucker Punch (simple attacks)
4. Prepared, Deflect, Backflip (simple skills)
5. Footwork, After Image (simple powers)
6. Noxious Fumes (requires start-of-turn hook)
7. Catalyst, Envenom (poison synergy)

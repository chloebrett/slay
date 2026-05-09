# Neutral Cards

Cards available to all characters. ✅ = implemented in Rust.

## Starters (in every character's starting deck)

| Card | Type | Cost | Effect | Impl |
|------|------|------|--------|------|
| Strike | Attack | 1 | Deal 6 damage. | ✅ |
| Defend | Skill | 1 | Gain 5 Block. | ✅ |

## Status cards (added to deck by enemies / events during combat)

| Card | Type | Cost | Effect | Impl |
|------|------|------|--------|------|
| Burn | Status | — | Unplayable. End of turn: take 2 damage. | ✅ |
| Dazed | Status | — | Unplayable. Ethereal. | ✅ |
| Slimed | Status | 1 | Exhaust. | ✅ |
| Void | Status | — | Unplayable. When drawn, lose 1 energy. | |
| Wound | Status | — | Unplayable. | ✅ |

## Curses (added to deck by events, relics, or enemy moves)

| Card | Type | Effect | Impl |
|------|------|--------|------|
| Ascender's Bane | Curse | Unplayable. Ethereal. Cannot be removed. | ✅ |
| Clumsy | Curse | Unplayable. Ethereal. | ✅ |
| Curse of the Bell | Curse | Unplayable. Cannot be removed. | ✅ |
| Decay | Curse | Unplayable. End of turn: take 2 damage. | ✅ |
| Doubt | Curse | Unplayable. End of turn: gain 1 Weak. | ✅ |
| Injury | Curse | Unplayable. | ✅ |
| Normality | Curse | Unplayable. You cannot play more than 3 cards this turn. | |
| Pain | Curse | Unplayable. When you play a card, lose 1 HP. | |
| Parasite | Curse | Unplayable. If transformed or removed, lose 3 Max HP. | ✅ |
| Regret | Curse | Unplayable. End of turn: lose 1 HP per card in hand. | ✅ |
| Shame | Curse | Unplayable. End of turn: gain 1 Frail. | ✅ |
| Writhe | Curse | Unplayable. Innate. | |

## Colorless reward cards (available to all characters in reward pools / shops)

Not yet implemented. These appear as combat rewards and in shops regardless of character.

### Common

| Card | Type | Cost | Effect |
|------|------|------|--------|
| Bandage Up | Skill | 0 | Heal 4 HP. Exhaust. |
| Blind | Skill | 0 | Apply 2 Weak to enemy. Exhaust. |
| Dark Shackles | Skill | 0 | Enemy loses 9 Strength this turn. Exhaust. |
| Deep Breath | Skill | 0 | Shuffle your discard pile into your draw pile. Draw 1. |
| Finesse | Skill | 0 | Gain 2 Block. Draw 1. |
| Flash of Steel | Attack | 0 | Deal 3 damage. Draw 1. |
| Good Instincts | Skill | 0 | Gain 6 Block. |
| Impatience | Skill | 0 | If you have no Attacks in hand, draw 2 cards. |
| Jack of All Trades | Skill | 0 | Add 1 random Colorless card to hand. Exhaust. |
| Madness | Skill | 1 | A random card in hand costs 0 this combat. Exhaust. |
| Panacea | Skill | 0 | Gain 2 Artifact. Exhaust. |
| Panic Button | Skill | 0 | Gain 30 Block. Cannot gain Block for 2 turns. Exhaust. |
| Purity | Skill | 0 | Exhaust up to 3 cards in hand. Exhaust. |
| Swift Strike | Attack | 0 | Deal 7 damage. |
| Thinking Ahead | Skill | 0 | Draw 2. Put 1 card from hand on top of draw pile. Exhaust. |
| Transmutation | Skill | X | Create X random Colorless cards in hand. Exhaust. |
| Violence | Skill | 0 | Put 3 random Attacks from draw pile into hand. Exhaust. |

### Uncommon

| Card | Type | Cost | Effect |
|------|------|------|--------|
| Discovery | Skill | 1 | Choose 1 of 3 random cards from your class. Add it to hand (costs 0 this turn). Exhaust. |
| Dramatic Entrance | Attack | 0 | Innate. Deal 8 damage to ALL enemies. Exhaust. |
| Enlightenment | Skill | 0 | Reduce cost of all cards in hand to 1 this turn. |
| Forethought | Skill | 0 | Place a card from hand at the bottom of your draw pile. |
| Hand of Greed | Attack | 2 | Deal 20 damage. If this kills a non-minion, gain 20 Gold. |
| Mind Blast | Attack | 2 | Innate. Deal damage equal to the size of your draw pile. |
| Panache | Power | 0 | Every time you play 5 cards in a turn, deal 10 damage to ALL enemies. |
| Sadistic Nature | Power | 0 | Whenever an enemy receives a debuff, deal 5 damage to them. |
| The Bomb | Skill | 2 | At the end of 3 turns, deal 40 damage to ALL enemies. |
| Trip | Skill | 0 | Apply 2 Vulnerable to ALL enemies. |

### Rare

| Card | Type | Cost | Effect |
|------|------|------|--------|
| Apotheosis | Skill | 2 | Upgrade ALL your cards for the rest of combat. Exhaust. |
| Chrysalis | Skill | 2 | Add 3 random Skills that cost 0 to hand. Exhaust. |
| Master of Strategy | Skill | 0 | Draw 3 cards. Exhaust. |
| Mayhem | Power | 2 | At the start of your turn, play the top card of your draw pile. |
| Metamorphosis | Skill | 2 | Add 3 random Attacks that cost 0 to hand. Exhaust. |
| Ritual Dagger | Attack | 1 | Deal 15 damage. If this kills a non-minion, permanently increase its damage by 3. Exhaust. |
| Secret Technique | Skill | 0 | Put a Skill from your draw pile into your hand. Exhaust. |
| Secret Weapon | Skill | 0 | Put an Attack from your draw pile into your hand. Exhaust. |

## Upgrade values (colorless)

| Card | Base | Plus |
|------|------|------|
| Blind | 2 Weak | 2 Weak to ALL |
| Dark Shackles | 9 Strength | 15 Strength |
| Enlightenment | Cost → 1 this turn | Cost → 1 permanent |
| Finesse | 2 Block, draw 1 | 4 Block, draw 1 |
| Flash of Steel | 3 dmg, draw 1 | 6 dmg, draw 1 |
| Good Instincts | 6 Block | 9 Block |
| Panache | 10 damage/5 cards | 14 damage/5 cards |
| Purity | exhaust 3 | exhaust 5 |
| Sadistic Nature | 5 damage | 7 damage |
| Swift Strike | 7 damage | 10 damage |
| The Bomb | 40 damage | 50 damage |

## Notes on mechanics needed

New mechanics required before implementing colorless reward cards:

- **Artifact** — prevents next debuff (Panacea)
- **On-kill trigger** — detect when an attack kills an enemy (Hand of Greed, Ritual Dagger)
- **Carry-forward Block lock** — cannot gain Block for N turns (Panic Button)
- **Delayed damage** — countdown before effect fires (The Bomb)
- **Discard → draw pile shuffle** — reshuffle (Deep Breath)
- **Innate** — already exists (Brutality+); Dramatic Entrance needs it for all grades
- **Play-count trigger** — track cards played this turn (Panache)
- **Per-combat card cost change** — permanent this-combat cost reduction (Madness)
- **Draw until condition** — draw until no Attacks remain? (Impatience just checks then draws)
- **Put card on draw pile top** — already needed for Thinking Ahead / Forethought

## Implementation priority

Lowest-effort first (no new mechanics):

1. **Flash of Steel** — deal damage, draw: both mechanisms exist
2. **Finesse** — gain block, draw: both exist
3. **Good Instincts** — gain block: exists (effectively just a 0-cost Defend variant)
4. **Swift Strike** — deal damage: exists (0-cost Slice variant)
5. **Blind** — apply Weak to target: exists (Inflame applies Strength, same pattern)
6. **Dark Shackles** — apply Strength debuff: similar to Disarm
7. **Purity** — exhaust cards: already in exhaust_card; needs a UI prompt
8. **Thinking Ahead** — draw then place on top: needs put-on-top mechanism
9. **Deep Breath** — shuffle discard to draw: needs reshuffle
10. **Dramatic Entrance** — innate AoE: same as Cleave but 0-cost innate

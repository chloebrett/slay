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
| Void | Status | — | Unplayable. When drawn, lose 1 energy. | ✅ |
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
| Normality | Curse | Unplayable. You cannot play more than 3 cards this turn. | ✅ |
| Pain | Curse | Unplayable. When you play a card, lose 1 HP. | ✅ |
| Parasite | Curse | Unplayable. If transformed or removed, lose 3 Max HP. | ✅ |
| Regret | Curse | Unplayable. End of turn: lose 1 HP per card in hand. | ✅ |
| Shame | Curse | Unplayable. End of turn: gain 1 Frail. | ✅ |
| Writhe | Curse | Unplayable. Innate. | ✅ |

## Colorless reward cards (available to all characters in reward pools / shops)

These appear as combat rewards and in shops regardless of character.

### Common

| Card | Type | Cost | Effect | Impl |
|------|------|------|--------|------|
| Bandage Up | Skill | 0 | Heal 4 HP. Exhaust. | ✅ |
| Blind | Skill | 0 | Apply 2 Weak to enemy. Exhaust. | ✅ |
| Dark Shackles | Skill | 0 | Enemy loses 9 Strength this turn. Exhaust. | ✅ |
| Deep Breath | Skill | 0 | Shuffle your discard pile into your draw pile. Draw 1. | ✅ |
| Finesse | Skill | 0 | Gain 2 Block. Draw 1. | ✅ |
| Flash of Steel | Attack | 0 | Deal 3 damage. Draw 1. | ✅ |
| Good Instincts | Skill | 0 | Gain 6 Block. | ✅ |
| Impatience | Skill | 0 | If you have no Attacks in hand, draw 2 cards. | ✅ |
| Jack of All Trades | Skill | 0 | Add 1 random Colorless card to hand. Exhaust. | ✅ |
| Madness | Skill | 1 | A random card in hand costs 0 this combat. Exhaust. | |
| Panacea | Skill | 0 | Gain 2 Artifact. Exhaust. | |
| Panic Button | Skill | 0 | Gain 30 Block. Cannot gain Block for 2 turns. Exhaust. | |
| Purity | Skill | 0 | Exhaust up to 3 cards in hand. Exhaust. | ✅ |
| Swift Strike | Attack | 0 | Deal 7 damage. | ✅ |
| Thinking Ahead | Skill | 0 | Draw 2. Put 1 card from hand on top of draw pile. Exhaust. | ✅ |
| Transmutation | Skill | X | Create X random Colorless cards in hand. Exhaust. | |
| Violence | Skill | 0 | Put 3 random Attacks from draw pile into hand. Exhaust. | ✅ |

### Uncommon

| Card | Type | Cost | Effect | Impl |
|------|------|------|--------|------|
| Discovery | Skill | 1 | Choose 1 of 3 random cards from your class. Add it to hand (costs 0 this turn). Exhaust. | |
| Dramatic Entrance | Attack | 0 | Innate. Deal 8 damage to ALL enemies. Exhaust. | ✅ |
| Enlightenment | Skill | 0 | Reduce cost of all cards in hand to 1 this turn. | ✅ |
| Forethought | Skill | 0 | Place a card from hand at the bottom of your draw pile. | ✅ |
| Hand of Greed | Attack | 2 | Deal 20 damage. If this kills a non-minion, gain 20 Gold. | ✅ |
| Mind Blast | Attack | 2 | Innate. Deal damage equal to the size of your draw pile. | ✅ |
| Panache | Power | 0 | Every time you play 5 cards in a turn, deal 10 damage to ALL enemies. | ✅ |
| Sadistic Nature | Power | 0 | Whenever an enemy receives a debuff, deal 5 damage to them. | |
| The Bomb | Skill | 2 | At the end of 3 turns, deal 40 damage to ALL enemies. | |
| Trip | Skill | 0 | Apply 2 Vulnerable to ALL enemies. | ✅ |

### Rare

| Card | Type | Cost | Effect | Impl |
|------|------|------|--------|------|
| Apotheosis | Skill | 2 | Upgrade ALL your cards for the rest of combat. Exhaust. | ✅ |
| Chrysalis | Skill | 2 | Add 3 random Skills that cost 0 to hand. Exhaust. | |
| Master of Strategy | Skill | 0 | Draw 3 cards. Exhaust. | ✅ |
| Mayhem | Power | 2 | At the start of your turn, play the top card of your draw pile. | |
| Metamorphosis | Skill | 2 | Add 3 random Attacks that cost 0 to hand. Exhaust. | |
| Ritual Dagger | Attack | 1 | Deal 15 damage. If this kills a non-minion, permanently increase its damage by 3. Exhaust. | |
| Secret Technique | Skill | 0 | Put a Skill from your draw pile into your hand. Exhaust. | ✅ |
| Secret Weapon | Skill | 0 | Put an Attack from your draw pile into your hand. Exhaust. | ✅ |

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

New mechanics required for remaining cards:

- **Artifact** — prevents next debuff (Panacea)
- **On-kill trigger** — detect when an attack kills an enemy (Hand of Greed, Ritual Dagger)
- **Carry-forward Block lock** — cannot gain Block for N turns (Panic Button)
- **Delayed damage** — countdown before effect fires (The Bomb)
- **Play-count trigger** — track cards played this turn (Panache)
- **Per-combat card cost change** — permanent this-combat cost reduction (Madness)

## Implementation order

### Minor — one new mechanism required

- **Panacea** — gain 2 Artifact; Artifact is a new status that absorbs the next debuff applied, intercepting `apply_status` for debuffs.
- **Sadistic Nature** — deal 5 damage to an enemy whenever they receive a debuff; needs an on-status-applied event hook that fires after `apply_status`.
- **Madness** — a random card in hand permanently costs 0 for this combat; needs a per-card-instance cost override stored alongside each `Card` in the hand.

### Major — significant new architecture
- **The Bomb** — deal 40 damage to ALL enemies after a 3-turn countdown; needs a delayed-effect system (a list of pending triggers with turn counters) that fires at end of player turn.
- **Mayhem** — at the start of each turn, automatically play the top card of the draw pile; needs an auto-play step in `StartPlayerTurn` that runs a full play-card cycle including targeting.
- **Panic Button** — gain 30 block but block gain is suppressed for the next 2 turns; needs a "block gain locked" flag with a turn countdown that intercepts `gain_player_block`.
- **Chrysalis / Metamorphosis** — add 3 random Skills (or Attacks) from any class that cost 0 to hand; needs temporary 0-cost card instances and a broad cross-class card pool to sample from.
- **Ritual Dagger** — deal 15 damage; if this kills, permanently increase its own damage by 3; needs mutable per-card-instance stats that persist across combats (run-level card data).
- **Transmutation** — X-cost: create X random colorless cards in hand; needs the X-cost mechanism (only Whirlwind uses it now) extended to Skills, plus a colorless card pool.
- **Discovery** — choose 1 of 3 random cards from your class; needs a class-specific card pool and a three-option choice UI.
- **Sadistic Nature** — deal 5 damage to an enemy whenever they receive a debuff; needs an on-status-applied event hook that fires after `apply_status`.

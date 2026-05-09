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

## Implementation order

### Trivial — pure composition of existing mechanics

- **Flash of Steel** — deal damage and draw 1 card; identical pattern to PommelStrike.
- **Finesse** — gain block and draw 1; same dual-effect pattern, defensive variant.
- **Good Instincts** — gain 6 block at 0 cost; just a different number on Defend.
- **Swift Strike** — deal 7 damage at 0 cost; same as Slice but with different numbers.
- **Blind** — apply 2 Weak to target; Weak status and `apply_status` both exist, same pattern as Inflame.
- **Trip** — apply 2 Vulnerable to ALL enemies; Vulnerable exists, apply-to-all pattern exists (Thunderclap).
- **Dramatic Entrance** — innate, deal 8 damage to ALL, exhaust; AoE damage, exhaust, and innate (Brutality+) all exist.
- **Writhe** — innate unplayable curse; same as AscendersBane but with the innate flag set.

### Minor — one new mechanism required

- **Bandage Up** — heal N HP and exhaust; Feed already restores max HP so HP healing exists, just needs a Skill wrapper.
- **Dark Shackles** — enemy loses 9 Strength this turn only; needs a "this-turn-only" temporary Strength debuff that resets at end of the enemy's turn, unlike the permanent reduction Disarm applies.
- **Mind Blast** — deal damage equal to draw pile size; needs to read `draw_pile.len()` as the base damage value at the time of play rather than a fixed constant.
- **Impatience** — draw 2 only if no Attacks are in hand; needs a hand type-scan before drawing, checking `card.card_type() == CardType::Attack`.
- **Violence** — move 3 random Attacks from the draw pile to hand; needs a "search draw pile by card type, remove, and add to hand" operation.
- **Secret Technique / Secret Weapon** — put a Skill or Attack from draw pile into hand; same draw-pile search pattern as Violence but picks one card.
- **Deep Breath** — shuffle the discard pile back into the draw pile; needs a reshuffle step (move all discard → draw, then re-shuffle).
- **Forethought** — place a chosen card from hand at the bottom of the draw pile; needs "insert at index 0" into the draw pile and a card-selection prompt.
- **Thinking Ahead** — draw 2 then place a chosen card on top of the draw pile; needs "insert at tail" into the draw pile and a card-selection prompt.
- **Purity** — exhaust up to 3 cards in hand of your choice; `exhaust_card` already exists, needs a multi-card hand-selection prompt.
- **Enlightenment** — reduce the cost of all cards in hand to 1 this turn; needs a temporary per-card cost override that clears at the start of the next turn.
- **Jack of All Trades** — add 1 random colorless card to hand; needs a colorless card pool to sample from and an "add to hand" mechanism.
- **Panacea** — gain 2 Artifact; Artifact is a new status that absorbs the next debuff applied, intercepting `apply_status` for debuffs.
- **Panache** — deal 10 AoE damage every 5 cards played in a turn; needs a `cards_played_this_turn` counter in `CombatState` and a post-play trigger that checks it.
- **Sadistic Nature** — deal 5 damage to an enemy whenever they receive a debuff; needs an on-status-applied event hook that fires after `apply_status`.
- **Pain (curse)** — whenever the player plays a card, lose 1 HP; needs an on-play trigger checked against curses in hand (or a flag on `CombatState`).
- **Normality (curse)** — cannot play more than 3 cards this turn; needs the same `cards_played_this_turn` counter and enforcement in the play-card path.
- **Void (status)** — when drawn, immediately lose 1 energy; needs an on-draw hook that runs as cards enter hand.
- **Hand of Greed** — deal 20 damage and gain 20 gold if this kills the enemy; needs kill detection (checking if HP reached 0) and a gold gain side-effect.
- **Madness** — a random card in hand permanently costs 0 for this combat; needs a per-card-instance cost override stored alongside each `Card` in the hand.
- **Apotheosis** — upgrade every card in the player's hand and discard pile in-place for this combat; the run-level deck list is untouched, so no reversion is needed — just iterate both piles calling `card.upgrade()` and replacing the entry.

### Major — significant new architecture
- **The Bomb** — deal 40 damage to ALL enemies after a 3-turn countdown; needs a delayed-effect system (a list of pending triggers with turn counters) that fires at end of player turn.
- **Mayhem** — at the start of each turn, automatically play the top card of the draw pile; needs an auto-play step in `StartPlayerTurn` that runs a full play-card cycle including targeting.
- **Panic Button** — gain 30 block but block gain is suppressed for the next 2 turns; needs a "block gain locked" flag with a turn countdown that intercepts `gain_player_block`.
- **Chrysalis / Metamorphosis** — add 3 random Skills (or Attacks) from any class that cost 0 to hand; needs temporary 0-cost card instances and a broad cross-class card pool to sample from.
- **Ritual Dagger** — deal 15 damage; if this kills, permanently increase its own damage by 3; needs mutable per-card-instance stats that persist across combats (run-level card data).
- **Transmutation** — X-cost: create X random colorless cards in hand; needs the X-cost mechanism (only Whirlwind uses it now) extended to Skills, plus a colorless card pool.

# Monster Implementation Plan

## Current state (6 enemies implemented)

| Enemy           | HP  | Moves                                              |
|-----------------|-----|----------------------------------------------------|
| Louse           | 20  | Bite (8 dmg) / Block (5 block) — alternating       |
| Fungibeast      | 22  | Chomp (6 dmg) / Slam (10 dmg) — alternating        |
| Cultist         | 50  | Incantation (Ritual +3) → Dark Strike (6 dmg) loop |
| Jaw Worm        | 40  | Chomp (11) turn 1; then Bellow/Thrash/Chomp (prob) |
| Small Spike Slime | 10 | Flame Tackle (5 dmg + Dazed to discard) — always  |
| Red Louse       | 12  | Bite (6 dmg, 75%) / Grow (+3 Str, 25%), no-repeat  |

Infrastructure in place: `Intent` (Attack/Defend/AttackDefend/Buff), `Effect` (DealDamage/GainBlock/GainStatus/ApplyStatus/AddToDiscard), `StatusEffect` (Vulnerable/Weak/Poison/Strength/Ritual/Dexterity), probabilistic RNG in `next_move`, `Card::Dazed`.

---

## Next enemies — in recommended order

### 1. Green Louse (zero new infra)

- **HP:** 12 (midpoint of 11–17)
- **Moves:** Bite (6 dmg, 75%) / Spit Web (2 Weak to player, 25%), no-repeat Grow-equivalent
- **New infra:** none — `ApplyStatus(Weak, 2)` already works
- **File:** `enemies/green_louse.rs`
- **Notes:** Mirrors Red Louse; Spit Web uses `Effect::ApplyStatus(StatusEffect::Weak, 2)`.
  Add both Louse variants to encounter pool alongside the existing `Louse`.
  Curl Up (gains block equal to first hit) deferred — reactive hook not yet supported.

### 2. Small Acid Slime (zero new infra)

- **HP:** 10 (midpoint of 8–12)
- **Moves:** Tackle (3 dmg) / Lick (1 Weak to player) — alternating or probabilistic
- **New infra:** none
- **File:** `enemies/small_acid_slime.rs`
- **Encounter:** pair with Small Spike Slime for a "Small Slimes" encounter

### 3. Blue Slaver (probabilistic, no new infra)

- **HP:** 48 (midpoint of 46–50)
- **Moves:** Stab (12 dmg, 60%) / Rake (7 dmg + 1 Weak, 40%), no-repeat
- **New infra:** none — probabilistic RNG already used by Jaw Worm / Red Louse
- **File:** `enemies/blue_slaver.rs`

### 4. Red Slaver (introduces Entangle)

- **HP:** 48 (midpoint of 46–50)
- **Moves:**
  - Turn 1: Stab (13 dmg)
  - Subsequent: Stab (55%) / Scrape (8 dmg + 1 Vulnerable, 45%) / Entangle (one-time)
  - Entangle: prevents player playing Attack cards this turn
- **New infra:**
  - `StatusEffect::Entangle` — player variant, one-time use
  - Check in `play_card` handler: if player has Entangle, reject Attack cards
  - Entangle consumed (removed) at start of player turn after applying
- **File:** `enemies/red_slaver.rs`

---

## Multi-enemy encounters (deferred until map work)

The following are interesting but require fighting multiple distinct enemies simultaneously.
Worth deferring until the branching map / encounter system is more developed:

- **Slavers** — Red + Blue Slaver together
- **Gang of Gremlins** — 5 different gremlin types (Fat, Mad, Shield, Sneaky, Wizard)
  - Shield Gremlin Protect (blocks a random ally) needs target selection among enemies
  - Mad Gremlin Angry (gains Strength when hit) needs a reactive "on-hit" hook
- **Lots of Slimes** — Small Acid Slime × 2 + Small Spike Slime

---

## Reactive powers (deferred)

These require a new hook in the damage pipeline and are lower priority:

| Power        | Enemy          | Description                                     |
|--------------|----------------|-------------------------------------------------|
| Curl Up      | Red/Green Louse | Gains block equal to first hit received        |
| Angry        | Mad Gremlin    | Gains Strength each time it takes attack damage |
| Split        | Large Slimes   | Spawns two medium slimes on death               |

---

## What to skip for now

- Medium/Large Acid and Spike Slimes (split mechanic — spawns new enemies mid-combat)
- Gremlin Nob, Lagavulin, Sentries (Act 1 elites — need multi-phase or multi-enemy logic)
- Act 2 / Act 3 enemies (Snecko, Chosen, Darkling, etc.) — fun but low priority while Act 1 is thin

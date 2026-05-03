# Monster Implementation Plan

## Current state

Two monsters exist (`Louse`, `Fungibeast`), both deterministic alternating-intent enemies.
Each lives in `enemies/<name>.rs` with a `DEF: EnemyDef` and a `next_intent(turn: u32) -> Intent`.
`Intent` currently has two variants: `Attack(i32)` and `Defend(i32)`.

`next_intent` is purely turn-based and deterministic — no RNG, no per-enemy state.

---

## Infrastructure gaps

The three monsters below are all achievable within the existing `Intent` model, but two
new `Intent` variants are needed:

| New variant | Used by |
|---|---|
| `Intent::Buff { strength: i32, block: i32 }` | Jaw Worm (Bellow) |
| `Intent::Debuff` | Cultist (Incantation — purely cosmetic, no damage) |

`Intent::Buff` needs wiring in `combat.rs`'s `EndEnemyTurn` handler: apply strength to the
enemy's status map and block to `enemy.block`, same as player buffs.

Cultist's Incantation applies **Ritual** (gain N Strength at the *end* of each enemy turn).
That is a new `StatusEffect::Ritual(i32)` — needs a tick in the enemy status processing path.
This can be deferred; Cultist is still interesting as Attack-only if Ritual is out of scope.

---

## Monsters — in recommended order

### 1. Cultist (simplest, no new infra needed if Ritual deferred)

- **HP:** 48–54 (use midpoint 50 for now)
- **Move pattern:** fixed sequence
  - Turn 1: Incantation — in the simplified version, `Intent::Defend(0)` (cosmetic skip)
  - Turn 2+: Dark Strike — `Intent::Attack(6)`
- **New infra:** none (Ritual deferred)
- **File:** `enemies/cultist.rs`

With Ritual in scope:
- Cultist gains `Strength +3` at the end of every enemy turn after Incantation.
- Needs `StatusEffect::Ritual(i32)` ticked in `apply_enemy_end_of_turn_statuses`.

### 2. Jaw Worm (introduces Buff intent)

- **HP:** 40–44 (use 40)
- **Move pattern:** probabilistic, needs RNG in `next_intent`
  - Turn 1: always Chomp — `Intent::Attack(11)`
  - Turn 2+: Bellow (45%) / Thrash (30%) / Chomp (25%), no repeat of same move
- **New infra:**
  - `Intent::Buff { strength: i32, block: i32 }` variant
  - `next_intent` signature changes to `next_intent(kind, turn, rng, last_intent) -> Intent`
  - `EndEnemyTurn` handler applies buff intents before enemy acts

### 3. Small Spike Slime (simplest slime, single move, introduces Dazed status)

- **HP:** 10–14 (use 10)
- **Move pattern:** always Flame Tackle — `Intent::AttackAndDebuff { damage: 5, effect: AddDazed }`
  - Dazed: adds an unplayable 0-cost card to the player's discard pile
- **New infra:**
  - `StatusCard::Dazed` (unplayable, exhausts on draw or play)
  - `Intent::AttackAndDebuff` variant (or just handle in card-play pipeline inline)
  - Adding status cards to discard pile

---

## Sequencing

1. **Cultist (no Ritual)** — zero new infra. Add `cultist.rs`, wire into `EnemyKind`, add to
   enemy pool for floors 1–3. Tests: HP, intent sequence.

2. **Jaw Worm** — adds `Intent::Buff` and probabilistic `next_intent`. Requires threading RNG
   into `next_intent`. Tests: Bellow applies strength + block, Chomp damages, no-repeat rule.

3. **Ritual status** — retroactively finish Cultist. Adds `StatusEffect::Ritual`. Tests: Cultist
   Strength grows each turn, damage scales accordingly.

4. **Small Spike Slime** — adds status card machinery (Dazed in discard). Tests: Dazed appears
   in discard, deck cycles through it.

---

## What to skip for now

- Medium/Large slime split mechanic (complex mid-combat enemy spawning)
- Green Louse Spit Web (Slimed cards — same status-card machinery as Dazed, easy follow-on)
- Probabilistic HP ranges (pick a fixed value per enemy until a random-HP system is wanted)

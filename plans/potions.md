# Potions Implementation Plan

Source: `~/c/spire-codex/data/eng/potions.json` — 48 total (16 Common, 16 Uncommon, 16 Rare)  
Extracted via: `plans/extract_potions.py` → `plans/potions.json`

## Already Implemented (9)

| Code name       | Codex ID            | Effect                                     |
|-----------------|---------------------|--------------------------------------------|
| FirePotion      | FIRE_POTION         | Deal 20 damage to one enemy                |
| ExplosivePotion | EXPLOSIVE_AMPOULE   | Deal 10 damage to ALL enemies              |
| BlockPotion     | BLOCK_POTION        | Gain 12 Block                              |
| StrengthPotion  | STRENGTH_POTION     | Gain 2 Strength                            |
| SwiftPotion     | SWIFT_POTION        | Draw 3 cards                               |
| FearPotion      | VULNERABLE_POTION   | Apply 3 Vulnerable to one enemy (targeted) |
| WeakPotion      | WEAK_POTION         | Apply 3 Weak to one enemy (targeted)       |
| BloodPotion     | BLOOD_POTION        | Heal 20% of Max HP                         |
| EnergyPotion    | ENERGY_POTION       | Gain 2 Energy                              |

> **Note:** `FearPotion` in code corresponds to `VULNERABLE_POTION` in the codex (applies Vulnerable).
> Rename to `VulnerablePotion` when convenient.

---

## Tier 1 — Simple Additions (no new status effects, no new mechanics)

These can be added with a single match arm in `potions.rs::apply`.

| Codex ID          | Name               | Effect                                        | Targeted |
|-------------------|--------------------|-----------------------------------------------|----------|
| POTION_OF_BINDING | Potion of Binding  | Apply 1 Weak + 1 Vulnerable to ALL enemies    | No       |
| FORTIFIER         | Fortifier          | Triple current Block                          | No       |
| CURE_ALL          | Cure All           | Gain 1 Energy + draw 2 cards                  | No       |
| FYSH_OIL          | Fysh Oil           | Gain 1 Strength + 1 Dexterity                 | No       |
| FRUIT_JUICE       | Fruit Juice        | Gain 5 Max HP (outside combat: heal + raise max) | No    |

---

## Tier 2 — Requires New Status Effects

Each needs a new `StatusEffect` variant and its turn-tick logic.

| Codex ID        | Name            | New Status Effect              | Effect                                            |
|-----------------|-----------------|-------------------------------|---------------------------------------------------|
| REGEN_POTION    | Regen Potion    | `Regen(n)` — heal n/turn       | Gain 5 Regen                                      |
| LIQUID_BRONZE   | Liquid Bronze   | `Thorns(n)` — reflect damage   | Gain 3 Thorns                                     |
| HEART_OF_IRON   | Heart of Iron   | `Plating(n)` — block per hit   | Gain 7 Plating                                    |
| DEXTERITY_POTION| Dexterity Potion| `Dexterity(n)` — reduces cost  | Gain 2 Dexterity (similar to Strength for Block)  |
| FLEX_POTION     | Flex Potion     | `StrengthDown(n)` at EoT       | Gain 5 Strength, lose 5 Strength at end of turn   |
| SPEED_POTION    | Speed Potion    | `DexterityDown(n)` at EoT      | Gain 5 Dexterity, lose 5 Dexterity at end of turn |
| POWDERED_DEMISE | Powdered Demise | `Burn(n)` — lose n HP/EoT      | Enemy loses 9 HP at end of each of its turns      |
| SHACKLING_POTION| Shackling Potion| `StrengthDown(n)` on enemies   | ALL enemies lose 7 Strength this turn             |

---

## Tier 3 — Requires New Mechanics

Needs engine-level work beyond status effects.

| Codex ID              | Name                  | Blocker / New Mechanic                                       |
|-----------------------|-----------------------|--------------------------------------------------------------|
| GAMBLERS_BREW         | Gambler's Brew        | Interactive discard: player chooses cards to discard, then draws that many |
| LIQUID_MEMORIES       | Liquid Memories       | Retrieve card from Discard Pile to Hand (needs discard pile access) |
| BOTTLED_POTENTIAL     | Bottled Potential     | Shuffle ALL cards into Draw Pile, draw 5                    |
| DROPLET_OF_PRECOGNITION| Droplet of Precognition| Choose from Draw Pile (needs pile peek UI)                 |
| DISTILLED_CHAOS       | Distilled Chaos       | Auto-play top 3 cards of Draw Pile                          |
| FAIRY_IN_A_BOTTLE     | Fairy in a Bottle     | Passive: triggers on lethal damage (needs potion passive hooks) |
| BLESSING_OF_THE_FORGE | Blessing of the Forge | Upgrade all cards in Hand for rest of combat                |
| TOUCH_OF_INSANITY     | Touch of Insanity     | Make one hand card free for the rest of combat              |
| DUPLICATOR            | Duplicator            | Next card played this turn plays an extra time              |
| STABLE_SERUM          | Stable Serum          | Retain Hand for 2 turns                                     |
| GIGANTIFICATION_POTION| Gigantification Potion| Next Attack deals triple damage                            |
| SNECKO_OIL            | Snecko Oil            | Draw 7, randomize costs of all Hand cards this turn         |
| CLARITY               | Clarity Extract       | Draw 1 now + 1 extra at start of next 3 turns               |
| RADIANT_TINCTURE      | Radiant Tincture      | Gain 1 Energy now + 1 at start of next 3 turns              |
| BEETLE_JUICE          | Beetle Juice          | Enemy attacks deal 30% less damage for 4 turns (needs Frail-like debuff) |
| SHIP_IN_A_BOTTLE      | Ship in a Bottle      | Gain 10 Block now + 10 Block at start of next turn          |
| ASHWATER              | Ashwater              | Exhaust any number of cards from Hand (player choice)       |

---

## Tier 4 — Card-Choice Potions (complex, skip for now)

| Codex ID         | Name             | Effect                                                  |
|------------------|------------------|---------------------------------------------------------|
| ATTACK_POTION    | Attack Potion    | Choose 1 of 3 random Attack cards, add free to Hand    |
| SKILL_POTION     | Skill Potion     | Choose 1 of 3 random Skill cards, add free to Hand     |
| POWER_POTION     | Power Potion     | Choose 1 of 3 random Power cards, add free to Hand     |
| COLORLESS_POTION | Colorless Potion | Choose 1 of 3 random Colorless cards, add free to Hand |
| OROBIC_ACID      | Orobic Acid      | Add random Attack + Skill + Power to Hand (all free)   |

These require a full card-choice UI flow mid-combat — similar complexity to Neow or card rewards.

---

## Out of Scope

| Codex ID          | Name             | Reason                                         |
|-------------------|------------------|------------------------------------------------|
| ENTROPIC_BREW     | Entropic Brew    | Fills empty potion slots with random potions — meta potion logic |
| LUCKY_TONIC       | Lucky Tonic      | Buffer status not yet in game                  |
| MAZALETHS_GIFT    | Mazaleth's Gift  | Ritual status not yet in game                  |
| SOLDIERS_STEW     | Soldier's Stew   | Replay mechanic not yet in game                |

---

## Recommended Implementation Order

1. **Tier 1** (5 potions) — pure logic, no engine changes, TDD straightforward
2. **Dexterity** + `Dexterity` status effect — high impact, parallels Strength
3. **Regen** — common enough to be worth adding; simple tick logic
4. **Thorns** — useful for Ironclad build diversity
5. **Flex/Speed** — requires `StrengthDown`/`DexterityDown` which also unlocks end-of-turn buff expiry for cards
6. **Tier 3** items as individual features once prerequisites exist

---

## Notes

- `random_potions()` pool should grow as each potion is added.
- Potions that require status effects need corresponding `describe_status` and `format_status` entries in `engine.rs`.
- All potions that work outside combat (e.g. Fruit Juice for Max HP) need handling in `run.rs` `apply_command` for the `UsePotion` command in Map/Neow states.

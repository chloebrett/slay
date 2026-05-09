# Potions Implementation Plan

Extracted via: `plans/extract_potions.py` → `plans/potions.json`

31 potions total (17 Common, 8 Uncommon, 6 Rare) — ironclad + shared pools only.

## Already Implemented (15)

| Code enum       | JAR class id    | Effect                                                   |
| --------------- | --------------- | -------------------------------------------------------- |
| FirePotion      | FirePotion      | Deal 20 damage to one enemy                              |
| ExplosivePotion | ExplosivePotion | Deal 10 damage to ALL enemies                            |
| BlockPotion     | BlockPotion     | Gain 12 Block                                            |
| StrengthPotion  | StrengthPotion  | Gain 2 Strength                                          |
| SwiftPotion     | SwiftPotion     | Draw 3 cards                                             |
| FearPotion      | FearPotion      | Apply 3 Vulnerable to one enemy (targeted)               |
| WeakPotion      | WeakenPotion    | Apply 3 Weak to one enemy (targeted)                     |
| BloodPotion     | BloodPotion     | Heal 20% of Max HP (Ironclad pool)                       |
| EnergyPotion    | EnergyPotion    | Gain 2 Energy                                            |
| DexterityPotion | DexterityPotion | Gain 2 Dexterity                                         |
| FruitJuice      | FruitJuice      | Gain 5 Max HP (and 5 current HP)                         |
| RegenPotion     | RegenPotion     | Gain 5 Regen (heals n HP at start of each player turn)   |
| LiquidBronze    | LiquidBronze    | Gain 3 Thorns (reflect damage when hit)                  |
| EssenceOfSteel  | EssenceOfSteel  | Gain 2 Metallicize (gain 2 Block at start of each turn)  |
| HeartOfIron     | HeartOfIron     | Gain 3 Metallicize (Ironclad pool)                       |

---

## Tier 2 — Requires New Status Effects

Each needs a new `StatusEffect` variant and tick logic in `combat.rs`.

| JAR class id  | Name          | Effect                                        | New status        |
| ------------- | ------------- | --------------------------------------------- | ----------------- |
| AncientPotion | Ancient Potion | Gain 1 Artifact (negate next debuff)          | `Artifact` exists |
| SteroidPotion | Flex Potion   | Gain 5 Strength, lose 5 at end of turn        | `StrengthDown` exists |
| SpeedPotion   | Speed Potion  | Gain 5 Dexterity, lose 5 at end of turn       | `DexterityDown` (new) |

> `StrengthDown` is already consumed by `tick_strength_modifiers()` in status.rs. `DexterityDown` needs the same treatment.  
> `Artifact` status exists in status.rs but the negate-debuff mechanic in `apply_status()` is not yet wired.

---

## Tier 3 — Requires New Mechanics

| JAR class id         | Name               | Blocker                                                            |
| -------------------- | ------------------ | ------------------------------------------------------------------ |
| GamblersBrew         | Gambler's Brew     | Interactive: player chooses cards to discard, then draws that many |
| LiquidMemories       | Liquid Memories    | Retrieve card from discard pile to hand                            |
| DistilledChaosPotion | Distilled Chaos    | Auto-play top 3 cards of draw pile                                 |
| DuplicationPotion    | Duplication Potion | Next card played this turn plays twice                             |
| FairyPotion          | Fairy in a Bottle  | Passive trigger: fires on lethal damage instead of dying           |
| SneckoOil            | Snecko Oil         | Draw 7 + randomize costs of all hand cards this turn               |
| SmokeBomb            | Smoke Bomb         | Escape non-boss combat (EscapeCombat effect, no rewards)           |

---

## Tier 4 — Card-Choice Potions (complex, skip for now)

Requires a full card-choice UI flow mid-combat — similar complexity to Neow or card rewards.

| JAR class id    | Name             | Effect                                                 |
| --------------- | ---------------- | ------------------------------------------------------ |
| AttackPotion    | Attack Potion    | Choose 1 of 3 random Attack cards, add free to Hand    |
| SkillPotion     | Skill Potion     | Choose 1 of 3 random Skill cards, add free to Hand     |
| PowerPotion     | Power Potion     | Choose 1 of 3 random Power cards, add free to Hand     |
| ColorlessPotion | Colorless Potion | Choose 1 of 3 random Colorless cards, add free to Hand |

---

## Out of Scope / Later

| JAR class id       | Name                  | Reason                                                                   |
| ------------------ | --------------------- | ------------------------------------------------------------------------ |
| EntropicBrew       | Entropic Brew         | Fills empty potion slots with random potions — meta potion logic         |
| BlessingOfTheForge | Blessing of the Forge | Upgrade all hand cards for rest of combat — card upgrade mechanic needed |

---

## Recommended Order

1. ~~**DexterityPotion**~~ ✅
2. ~~**FruitJuice**~~ ✅
3. ~~**RegenPotion**~~ ✅
4. ~~**LiquidBronze** + **EssenceOfSteel/HeartOfIron**~~ ✅
5. **SteroidPotion/SpeedPotion** — StrengthDown exists; SpeedPotion needs DexterityDown
6. **AncientPotion** — Artifact status exists; need negate-debuff logic in `apply_status()`
7. **Tier 3** items individually as features

---

## Notes

- `random_potions()` pool should grow as each potion is added.
- All new statuses need `describe_status` + display entries in `engine.rs`.
- Potions usable outside combat (FruitJuice) need handling in `run.rs::apply_command` for Map/Neow states.

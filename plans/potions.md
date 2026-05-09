# Potions Implementation Plan

Extracted via: `plans/extract_potions.py` → `plans/potions.json`

31 potions total (17 Common, 8 Uncommon, 6 Rare) — ironclad + shared pools only.

## Already Implemented (19)

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
| HeartOfIron       | HeartOfIron       | Gain 3 Metallicize (Ironclad pool)                       |
| SteroidPotion     | SteroidPotion     | Gain 5 Strength, lose 5 at end of turn (Flex Potion)     |
| SpeedPotion       | SpeedPotion       | Gain 5 Dexterity, lose 5 at end of turn                  |
| AncientPotion     | AncientPotion     | Gain 1 Artifact (negate next debuff)                     |
| DuplicationPotion | DuplicationPotion | Next card played this turn fires twice                   |

---

## Tier 3 — Requires New Mechanics

| JAR class id         | Name               | Blocker                                                            |
| -------------------- | ------------------ | ------------------------------------------------------------------ |
| GamblersBrew         | Gambler's Brew     | Discard all hand cards, draw that many (simplified: no card choice)|
| LiquidMemories       | Liquid Memories    | Random card from discard pile to hand (simplified: no choice UI)   |
| DistilledChaosPotion | Distilled Chaos    | Auto-play top 3 cards of draw pile                                 |
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
5. ~~**SteroidPotion/SpeedPotion**~~ ✅
6. ~~**AncientPotion** + **DuplicationPotion**~~ ✅
7. **GamblersBrew/LiquidMemories** — simplified no-choice versions (next)
8. **Tier 3 hard** items: DistilledChaos, SmokeBomb, FairyPotion, SneckoOil

---

## Notes

- `random_potions()` pool should grow as each potion is added.
- All new statuses need `describe_status` + display entries in `engine.rs`.
- Potions usable outside combat (FruitJuice) need handling in `run.rs::apply_command` for Map/Neow states.

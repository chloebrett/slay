# Potions Implementation Plan

Source: STS1 Steam installation JAR (`desktop-1.0.jar` localization + class bytecode)  
Extracted via: `plans/extract_potions.py` → `plans/potions.json`

31 potions total (17 Common, 8 Uncommon, 6 Rare) — ironclad + shared pools only.

## Already Implemented (9)

| Code enum       | JAR class id        | Effect                                      |
|-----------------|---------------------|---------------------------------------------|
| FirePotion      | FirePotion          | Deal 20 damage to one enemy                 |
| ExplosivePotion | ExplosivePotion     | Deal 10 damage to ALL enemies               |
| BlockPotion     | BlockPotion         | Gain 12 Block                               |
| StrengthPotion  | StrengthPotion      | Gain 2 Strength                             |
| SwiftPotion     | SwiftPotion         | Draw 3 cards                                |
| FearPotion      | FearPotion          | Apply 3 Vulnerable to one enemy (targeted)  |
| WeakPotion      | WeakenPotion        | Apply 3 Weak to one enemy (targeted)        |
| BloodPotion     | BloodPotion         | Heal 20% of Max HP (Ironclad pool)          |
| EnergyPotion    | EnergyPotion        | Gain 2 Energy                               |

---

## Tier 1 — Simple Additions

No new status effects or engine mechanics needed. Each is a single match arm in `potions.rs::apply`.

| JAR class id  | Name              | Effect                                              | Targeted |
|---------------|-------------------|-----------------------------------------------------|----------|
| DexterityPotion | Dexterity Potion | Gain 2 Dexterity (parallels Strength for Block)     | No       |
| FruitJuice    | Fruit Juice       | Gain 5 Max HP                                       | No       |

> **Dexterity** needs a `StatusEffect::Dexterity` that adds to Block calculation — same pattern as Strength adds to damage.

---

## Tier 2 — Requires New Status Effects

Each needs a new `StatusEffect` variant and tick logic in `combat.rs`.

| JAR class id    | Name              | Effect                                                   | New status           |
|-----------------|-------------------|----------------------------------------------------------|----------------------|
| RegenPotion     | Regen Potion      | Gain 5 Regen (heal n HP at end of each player turn)      | `Regen(n)`           |
| LiquidBronze    | Liquid Bronze     | Gain 3 Thorns (reflect damage when hit)                  | `Thorns(n)`          |
| EssenceOfSteel  | Essence of Steel  | Gain 2 Plated Armor (gain n Block at start of each turn) | `PlatedArmor(n)`     |
| HeartOfIron     | Heart of Iron     | Gain 3 Metallicize (same as Plated Armor, Ironclad pool) | `PlatedArmor(n)` (same) |
| AncientPotion   | Ancient Potion    | Gain 1 Artifact (negate next debuff)                     | `Artifact(n)`        |
| SteroidPotion   | Flex Potion       | Gain 5 Strength, lose 5 at end of turn                   | `StrengthDown(n)`    |
| SpeedPotion     | Speed Potion      | Gain 5 Dexterity, lose 5 at end of turn                  | `DexterityDown(n)`   |

> HeartOfIron and EssenceOfSteel both give Plated Armor / Metallicize — same status, different names in-game.  
> `StrengthDown` / `DexterityDown` also unlock end-of-turn expiry for the card Flex.

---

## Tier 3 — Requires New Mechanics

| JAR class id        | Name              | Blocker                                                          |
|---------------------|-------------------|------------------------------------------------------------------|
| GamblersBrew        | Gambler's Brew    | Interactive: player chooses cards to discard, then draws that many |
| LiquidMemories      | Liquid Memories   | Retrieve card from discard pile to hand                         |
| DistilledChaosPotion| Distilled Chaos   | Auto-play top 3 cards of draw pile                              |
| DuplicationPotion   | Duplication Potion| Next card played this turn plays twice                          |
| FairyPotion         | Fairy in a Bottle | Passive trigger: fires on lethal damage instead of dying        |
| SneckoOil           | Snecko Oil        | Draw 7 + randomize costs of all hand cards this turn            |
| SmokeBomb           | Smoke Bomb        | Escape non-boss combat (EscapeCombat effect, no rewards)        |

---

## Tier 4 — Card-Choice Potions (complex, skip for now)

Requires a full card-choice UI flow mid-combat — similar complexity to Neow or card rewards.

| JAR class id   | Name             | Effect                                                 |
|----------------|------------------|--------------------------------------------------------|
| AttackPotion   | Attack Potion    | Choose 1 of 3 random Attack cards, add free to Hand   |
| SkillPotion    | Skill Potion     | Choose 1 of 3 random Skill cards, add free to Hand    |
| PowerPotion    | Power Potion     | Choose 1 of 3 random Power cards, add free to Hand    |
| ColorlessPotion| Colorless Potion | Choose 1 of 3 random Colorless cards, add free to Hand|

---

## Out of Scope / Later

| JAR class id  | Name            | Reason                                              |
|---------------|-----------------|-----------------------------------------------------|
| EntropicBrew  | Entropic Brew   | Fills empty potion slots with random potions — meta potion logic |
| BlessingOfTheForge | Blessing of the Forge | Upgrade all hand cards for rest of combat — card upgrade mechanic needed |

---

## Recommended Order

1. **DexterityPotion** — simple, unlocks Dexterity status (important for many relics/cards)
2. **FruitJuice** — simple max HP gain, no new logic
3. **RegenPotion** — common Uncommon drop, Regen tick is straightforward  
4. **LiquidBronze** + **EssenceOfSteel/HeartOfIron** — Thorns + Plated Armor status pair
5. **SteroidPotion/SpeedPotion** — StrengthDown/DexterityDown (also needed for Flex card)
6. **AncientPotion** — Artifact (negate debuff mechanic)
7. **Tier 3** items individually as features

---

## Notes

- `random_potions()` pool should grow as each potion is added.
- All new statuses need `describe_status` + display entries in `engine.rs`.
- Potions usable outside combat (FruitJuice) need handling in `run.rs::apply_command` for Map/Neow states.

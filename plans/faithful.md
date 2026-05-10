# Faithfulness Backlog

Gaps between the current implementation and the real Slay the Spire Act 1.
Items are roughly ordered by impact on feel.

---

## 1. Act 1 map structure — 16 floors

**Current:** 10-floor map (floors 0–9), boss at floor 9.

**Real game:** Act 1 has exactly 16 floors. Three of them are fixed:

- **Floor 9:** Always a Treasure Room (guaranteed free relic, no combat)
- **Floor 15:** Always a Rest Site (guaranteed campfire before boss)
- **Floor 16:** Boss — one of Slime Boss, The Guardian, or Hexaghost

The remaining floors (1–8, 10–14) are generated from a weighted distribution:

- 53% Normal Combat
- 24% Unknown/Event (resolves randomly to event, combat, shop, or treasure on entry)
- 15% Merchant
- 8% Elite

**Additional constraint:** Elites cannot appear before floor 6, and no two
consecutive floors on a path can be Elite + Rest Site, Elite + Merchant, or
Rest Site + Merchant.

**Changes needed (`slay-core/src/run.rs`):**

- Expand `generate_map()` from 10 to 16 floors
- Floor 9 = Treasure, floor 15 = Rest Site, floor 16 = Boss (all fixed)
- Replace hardcoded floor layout with probability-weighted generation for floors 1–8, 10–14
- Elites gated to floor 6+
- Add Hexaghost and The Guardian to `boss_encounters()`

**Corrections from earlier draft:**

- ~~"17 nodes — 16 traversal floors and a boss floor"~~ — it's 16 floors total, boss is floor 16
- ~~"Treasure room placed after at least one elite path"~~ — treasure is always floor 9, fixed
- ~~"Rest site guaranteed in floors 15–16"~~ — rest site is always floor 15, fixed
- ~~"Floor 5–6: Shop guaranteed somewhere"~~ — shops are probabilistic, not guaranteed on a specific floor

---

## 2. Potion drop rate — 40% base with adaptive pity ✅

**Current:** Implemented. `potion_chance: f64` on `Player` (starts at 0.40).
`award_potion()` adjusts ±0.10 after each roll and clamps to [0.0, 1.0].

**Real game:** 40% base, but with an adaptive pity mechanic:

- If a potion drops: next combat's chance decreases by 10%
- If no potion drops: next combat's chance increases by 10%
- Resets to 40% at the start of each Act

**Also wrong in earlier draft:**

- ~~"Elites and bosses have a higher chance (~50%)"~~ — the same adaptive mechanic
  applies to all combat types; there is no separate elite/boss rate

**Note:** Per-act reset not yet implemented (no multi-act support).

---

## 3. Elites drop uncommon relics (not common)

**Current:** Implemented — elite victories now call `random_uncommon_relic()`. ✅

**Note:** The wiki describes the elite relic as coming from "common/uncommon/rare
pool" but this appears to be imprecise wiki wording. The accepted community
understanding is that Act 1 elites drop uncommon relics. Leaving as uncommon.

---

## 4. Gold ranges

**Current:** Flat values — need to audit (`GOLD_PER_COMBAT`, `GOLD_PER_ELITE`).

**Real game ranges:**

- Normal combat: 10–20 gold
- Elite combat: 25–35 gold
- Boss: 95–105 gold

**Changes needed (`slay-core/src/run.rs`):**

- Replace flat constants with `rng`-based ranges

## 5. Enemy encounter types ✅

**Easy pool** (first 3 combats): Cultist, JawWorm, 2 Louses, Small Slimes (MediumSpike + SmallAcid).

**Hard pool** (remaining combats): BlueSlaver, 2 FungiBeasts, 3 Louses, LargeSpike, LargeAcid,
Looter, Exordium Thugs (Looter + Mugger), Exordium Wildlife (JawWorm+GreenLouse /
Fungibeast+RedLouse / JawWorm+MediumSpike), RedSlaver, Gremlin Gang (4 of 5 gremlins),
Swarm of Slimes (3 SmallSpike + 2 SmallAcid).

**Elite pool**: GremlinNob, Lagavulin, 3 Sentries.

**Boss pool**: TheGuardian, SlimeBoss, Hexaghost (all three now included).

**Known gaps:**
- Encounter weights (wiki: 1/1.5/2) are approximated as uniform; pick_encounter shuffles and picks first
- Gremlin Gang should be 4 random of 5 gremlins each time; currently fixed to same 4
- Small Slimes internal randomness (MediumSpike+SmallAcid vs MediumAcid+SmallSpike) not implemented
- Louse color randomness (each louse independently 50% red/green) not implemented

---

## Future items (not yet scoped)

- **Unknown rooms resolving:** Unknown/? nodes randomly resolve to event, combat,
  shop, or treasure on entry — currently they are placed as static Event nodes
- **Card reward pool:** rewards should draw from the full Ironclad card set,
  weighted by rarity; currently draws from `reward_pool()` which may not match
- **Boss card rewards:** boss victories should offer rare cards (currently same as normal)
- **Burning Elite:** a forced elite room (marked with a flame) once per act
- **Campfire actions:** smith (upgrade) and rest (heal 30%) are implemented;
  "Recall" (Spire's Key) and "Lift" (Girya) are not
- **Boss relic choice:** after the boss, player picks 1 of 3 boss relics —
  currently not implemented
- **Multiple acts:** Act 2 and Act 3 are entirely absent

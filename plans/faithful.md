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

## 2. Potion drop rate — 40% base with adaptive pity

**Current:** Implemented as a flat 40% after all combats (done). ✅

**Real game:** 40% base, but with an adaptive pity mechanic:
- If a potion drops: next combat's chance decreases by 10%
- If no potion drops: next combat's chance increases by 10%
- Resets to 40% at the start of each Act

**Also wrong in earlier draft:**
- ~~"Elites and bosses have a higher chance (~50%)"~~ — the same adaptive mechanic
  applies to all combat types; there is no separate elite/boss rate

**Changes needed (`slay-core/src/run.rs`):**
- Track `potion_chance: f64` on the player or run state (starts at 0.40)
- After each combat, adjust ±0.10 based on whether a potion dropped

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

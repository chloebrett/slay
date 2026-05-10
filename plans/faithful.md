# Faithfulness Backlog

Gaps between the current implementation and the real Slay the Spire Act 1.
Items are roughly ordered by impact on feel.

---

## 1. Act 1 map structure — 16 floors

**Current:** 10-floor map (floors 0–9), boss at floor 9.

### How the real map actually works (from community reverse-engineering)

The map is a 7×15 grid (7 columns, 15 traversal floors + 1 boss floor = 16 total).
Layout and node-type assignment are two separate phases.

**Phase 1 — Path layout:**
6 paths are traced through the grid from bottom to top. The first and second paths must
start from distinct columns; subsequent paths may reuse any starting column. Any two
starting nodes that merge into a shared path have one removed (so each visible starting
node fans out in exactly one direction).

**Phase 2 — Node type assignment (bucket system):**

Three nodes are pre-typed before the bucket is filled:
- Floor 9 (1-indexed): Treasure
- Floor 15 (1-indexed): Rest Site
- Floor 1 (1-indexed): Normal enemy (the starting floor)

Remaining untyped nodes are filled from a shuffled bucket. The bucket contains:
- 5% Shops
- 12% Rest Sites
- 22% Events
- 8% Elites
- Remainder: Normal enemies

The bucket is shuffled and nodes are assigned in order. If a type would violate a
constraint, the next bucket entry is tried. If the bucket is exhausted the node
becomes a Normal enemy.

**Adjacency / placement constraints:**
- No consecutive Rests, Shops, or Elites on the same path (if two nodes share a
  parent, they can't both be the same special type — Normal enemies are exempt)
- If a node has multiple exits, no two exits can be the same special type
  (Normal enemies again exempted)
- Elites and Rest Sites cannot appear before floor 6 (1-indexed)
- Rest Sites cannot appear on floor 14 (the floor directly below the guaranteed
  Rest Site on floor 15)

**Corrections from earlier draft:**
- ~~"53/24/15/8% per-node probability roll"~~ — it's a bucket system, not per-node rolls
- ~~"Elite+RestSite consecutive forbidden"~~ — Elite then Rest Site IS allowed (common path)
- ~~"Floor 16 = Boss"~~ — Boss is floor 16 (1-indexed) = index 15

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

## 4. Gold ranges ✅

**Current:** Implemented — `combat_gold(is_elite, is_boss, rng)` rolls 10–20 / 25–35 / 95–105.

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

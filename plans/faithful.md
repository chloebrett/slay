# Faithfulness Backlog

Gaps between the current implementation and the real Slay the Spire Act 1.
Items are roughly ordered by impact on feel.

---

## 1. Act 1 map structure — 16 floors + boss

**Current:** 10-floor map (floors 0–9), boss at floor 9.

**Real game:** Act 1 has 17 nodes — 16 traversal floors and a boss floor.
Typical structure guarantees:
- Floors 1–4: Easy combats, events
- Floor 5–6: Shop guaranteed somewhere in here
- Floors 6–10: Mix of hard combats, events, elites (2–3 elite rooms)
- Treasure room placed after at least one elite path
- Floor 15–16: Rest site guaranteed before boss
- Floor 17: Boss (Slime Boss, Hexaghost, or The Guardian — chosen at run start)

**Changes needed (`slay-core/src/run.rs`):**
- Expand `generate_map()` from 10 to 17 rows
- Add more branching paths (the real game has 2–6 options per floor in some spots)
- Guarantee ≥ 2 elite rooms in the act
- Guarantee a rest site in the last 2–3 floors before the boss
- Add Hexaghost and The Guardian to `boss_encounters()`
- Treasure room should follow elite paths, not be hardcoded at floor 8

---

## 2. Potion drop rate — 40% chance after normal combat

**Current:** `award_potion()` is called unconditionally after every combat
(`run.rs` lines ~605, ~674). Every fight gives a potion.

**Real game:** Potions drop with 40% base chance after normal combats.
Elites and bosses have a higher chance (~50%). The `Alchemize` relic and
`White Beast Statue` give guaranteed drops but those are out of scope for now.

**Changes needed (`slay-core/src/run.rs`):**
- Add `rng.next_f32() < 0.40` (or equivalent) gate in `award_potion()` calls
  after normal combat
- Keep potion award unconditional after elite/boss for now (or use 50% —
  low priority)

---

## 3. Elites drop uncommon relics (not common)

**Current:** Elite victories call `random_common_relic(rng)` — they award
common-tier relics (`run.rs` lines ~519, ~600, ~669).

**Real game:** Elites award **uncommon** relics. Common relics come from
shops and events; uncommon relics are the elite reward.

**Changes needed (`slay-core/src/relics/mod.rs` + `run.rs`):**
- Confirm `Relic::uncommon_pool()` exists (or add it) with the uncommon-tier relics
- Replace `random_common_relic` calls in elite victory blocks with
  `random_uncommon_relic`

---

## Future items (not yet scoped)

- **Gold scaling:** real game gives 10–20 gold for easy combats, 15–25 for hard,
  100 for elites, 300 for boss — current values need audit
- **Card reward pool:** rewards should draw from the full Ironclad card set,
  weighted by rarity; currently draws from `reward_pool()` which may not match
- **Event rooms:** events are placed on the map but no event logic is implemented
- **Burning Elite:** an elite room the player is forced into (marked with a flame)
  once per act
- **Campfire actions:** smith (upgrade) and rest (heal 30%) are implemented;
  "Recall" (Spire's Key) and "Lift" (Girya) are not
- **Boss relic choice:** after the boss, the player picks 1 of 3 boss relics —
  currently not implemented
- **Multiple acts:** Act 2 and Act 3 are entirely absent

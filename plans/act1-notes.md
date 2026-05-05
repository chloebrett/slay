# Act 1 Notes — Slay the Spire

Reference material for making the map a faithful Act 1 clone.
Source: https://slay-the-spire.fandom.com/wiki/Act_1

---

## Map Structure

- **16 playable floors** (floors 1–16 in STS1 display, 0–15 in our 0-indexed rows)
- Floor 17 is the post-boss chest / act transition — not a map node the player chooses
- Up to **6 nodes per floor row**, connected by edges (irregular isometric grid)
- Each node connects to 1–3 nodes on the row above and 1–3 below

### Fixed floors

| STS floor | Our index | Content |
|-----------|-----------|---------|
| 1         | 0         | Easy combat only (drawn from easy pool) |
| 9         | 8         | Treasure rooms only (free relic, no combat) |
| 15        | 14        | Rest sites only (guaranteed campfire before boss) |
| 16        | 15        | Boss (one of: Slime Boss, Hexaghost, The Guardian) |

### Variable floor weights (all other floors)

| Room type      | Weight |
|----------------|--------|
| Normal combat  | 53%    |
| Unknown/Event  | 22%    |
| Rest site      | 12%    |
| Elite combat   | 8%     |
| Shop           | 5%     |

**Elite rooms do not appear before floor 6 (our index 5).**

The first 3 combat encounters are drawn from the **easy pool**; all subsequent combats use the **hard pool**.

---

## Enemy Pools

### Easy pool (floor 1 and first 3 encounters)
- Cultist
- Jaw Worm
- 2× Louses (Red + Green)
- Small Slimes (Spike Slime + Acid Slime)

### Hard pool (floors 2+ after 3 easy encounters used)
- Blue Slaver
- Red Slaver
- Gremlin Gang (3–5 Gremlins) — *not yet implemented*
- 3 Louses — *not yet implemented*
- 2 Fungi Beasts
- Exordium Thugs — *not yet implemented*
- Exordium Wildlife — *not yet implemented*
- Large Slime (Medium Slime + Small Slimes) — *not yet implemented*
- Looter — *not yet implemented*

### Elite pool (floor 6+)
- **Gremlin Nob** — *not yet implemented*
- **Lagavulin** — *not yet implemented*
- **3 Sentries** — *not yet implemented*

### Boss pool (floor 16, one chosen at random)
- **Slime Boss** — *not yet implemented*
- **Hexaghost** — *not yet implemented*
- **The Guardian** — *not yet implemented*

---

## Treasure Rooms (floor 9)

- Player enters the room and receives a **relic** (drawn from the relic pool)
- No combat, no choices beyond picking up the relic
- Our `MapNode` enum needs a `Treasure` variant
- State transition: `Map → Treasure → (player gets relic) → Map` (or just advance floor)

---

## Event Rooms (Unknown / Question Mark)

When a player enters an unknown room, it resolves dynamically to one of:
- Event (unique encounter with choices)
- Monster
- Shop
- Treasure

In practice for a first pass we can treat all unknowns as Events and implement a small
subset of the Act 1 event pool. The simplest events to implement:

| Event name           | Summary |
|----------------------|---------|
| Big Fish             | Gain 14 gold / heal 5 HP / +3 max HP (but lose a card) |
| Dead Adventurer      | Gain relic or fight an enemy |
| Fountain of Cleansing | Remove a curse |
| Golden Idol          | Gain 250 gold, but debuff — or leave |
| Living Wall          | Remove, transform, or upgrade a card |
| Mysterious Sphere    | Fight two elites for a relic |
| Scrap Ooze           | 25/37.5/50/62.5% chance per attempt to get a relic, lose HP each try |
| Shining Light        | Upgrade 2 random cards, lose 15 HP |
| The Cleric           | Pay 35g to remove a card / pay 50g to heal |
| The Ssserpent        | 50g for a curse added to deck |
| Treasure in the House | Open a chest — relic or monsters |
| Vampires             | Lose all Burning Blood stacks, gain 5× Bite card |
| Ominous Forge        | Upgrade a card (lose 3 HP) or leave |

Events are optional scope — a stub "nothing happens, advance" is acceptable for v1.

---

## Rewards

| Encounter type | Gold         | Cards     | Other         |
|----------------|-------------|-----------|---------------|
| Normal combat  | 10–20g      | 3 options | 40% potion    |
| Elite combat   | 25–35g      | 3 options | 1 relic       |
| Boss           | 95–105g     | 3 rare options | —        |

---

## Current Implementation Gaps

| Gap | Priority | Complexity |
|-----|----------|------------|
| Floor count (10 → 16) | High | Low |
| Fixed floors (Treasure at 9, Rest at 15) | High | Low |
| Probabilistic room type distribution | High | Medium |
| Multiple nodes per floor (up to 6) with edges | High | High |
| `MapNode::Elite` variant | High | Low |
| `MapNode::Event` variant | Medium | Low (stub ok) |
| `MapNode::Treasure` variant | High | Low |
| Act 1 elite enemies (Nob, Lagavulin, Sentries) | High | High |
| Act 1 boss enemies (Slime Boss, Hexaghost, Guardian) | High | High |
| Hard combat pool completeness | Medium | Medium |
| Elite combat rewards (relic) | Medium | Low |
| Boss combat rewards (gold + 3 rare cards) | Medium | Low |
| Gold rewards from combat (10–20g random) | Medium | Low |
| Potion drop from combat (40% chance) | Low | Low |

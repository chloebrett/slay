# Act 1 Notes — Slay the Spire

Reference material for Act 1 implementation.
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

**Implemented.** `MapNode::Treasure` → `GameState::TreasureRoom` → `Command::LeaveTreasure` → back to map.
- Player enters, sees the relic inside, uses `leave` / `take` to collect it and advance.
- Relic is chosen randomly from the full relic pool on entry.
- Fixed at floor index 8 (0-indexed) in our 10-floor map (floor 7 combat → floor 8 treasure → floor 9 boss).

---

## Event Rooms (Unknown / Question Mark)

When a player enters an unknown room it resolves to one of: event, monster, shop, or
treasure. For our first pass, unknown rooms are always events.

---

### Act 1 (Exordium) Event Pool

#### Big Fish
- **[Banana]** Heal ~30% of max HP. *(healAmt derived from max HP)*
- **[Donut]** Gain 3 Max HP.
- **[Box]** Obtain a random Relic. Become Cursed — Regret.
- **[Leave]**
- *Needs: curse system for Box option.*

#### The Cleric
- **[Heal]** Pay 35 gold → heal 8 HP.
- **[Purify]** Pay 50 gold → remove a card from deck. *(card selection needed)*
- **[Leave]**
- *Locked options if player has insufficient gold.*
- *Needs: card-selection sub-state for Purify.*

#### Dead Adventurer
- **[Search]** Find loot (gold or relic). X% chance monster returns and attacks.
  - Chance increases with each search. At 99% the monster always returns.
- **[Leave]**
- *Needs: combat trigger mid-event (complex). Likely skip for v1.*

#### Golden Idol
- **[Take]** Obtain Golden Idol, then choose escape method:
  - [Outrun] Become Cursed — Injury.
  - [Smash] Take some damage.
  - [Hide] Lose some Max HP.
- **[Leave]**
- *Multi-step with sub-choices. Complex. Skip for v1.*

#### Golden Wing (Wing Statue)
- **[Pray]** Remove a card from deck. Lose some HP. *(card selection needed)*
- **[Destroy]** Gain 50–150 gold. *(only if player has a card with high enough damage)*
- **[Leave]**
- *Needs: card-selection sub-state.*

#### Living Wall
- **[Forget]** Remove a card from deck. *(card selection)*
- **[Change]** Transform a card. *(card selection + transform mechanic)*
- **[Grow]** Upgrade a card. *(card selection)*
- **[Leave]**
- *Needs: card selection and card transform. Skip for v1.*

#### Mushrooms
- **[Stomp]** Triggers combat against the mushrooms.
- **[Eat]** Heal some HP. Become Cursed — Parasite.
- **[Leave]**
- *Needs: combat trigger mid-event, curse system. Skip for v1.*

#### Scrap Ooze
- **[Reach Inside]** Lose 3 HP. 25% chance → obtain a relic.
  - Each failed attempt: take 3 more HP (cumulative), chance +10%.
  - At 99% the relic is guaranteed.
- **[Leave]**
- *Multi-step (repeatable choice). Needs: persistent event state between picks.*
- *Medium complexity. Could simplify to a single one-shot attempt.*

#### Shining Light
- **[Enter]** Upgrade 2 random cards. Lose 8 HP. *(locked if no upgradeable cards)*
- **[Leave]**
- *Simple. No card selection — picks randomly from upgradeable cards.*
- **Priority: implement first.**

#### The Ssssserpent (Liars Game)
- **[Agree]** Gain 150 gold *(175 on lower ascension)*. Become Cursed — Doubt.
- **[Disagree]** Nothing.
- **[Leave]**
- *Curse is flavourful but skippable for v1 — just give the gold.*
- **Priority: implement first (simplest).**

#### World of Goop
- **[Gather Gold]** Gain 75 gold. Lose 11 HP.
- **[Leave It]** Lose 20–50 gold *(random; capped at current gold)*.
- **[Leave]** Nothing happens.
- *Entirely gold/HP — no other systems needed.*
- **Priority: implement first.**

---

### Complexity tiers

| Tier | Constraint | Events |
|------|-----------|--------|
| Simple | HP / gold / relic only | World of Goop, Shining Light, Ssssserpent, Big Fish (Banana+Donut only) |
| Needs cursing | Curse mechanic | Big Fish (Box), Ssssserpent (Agree) |
| Needs card-select | Card selection sub-state | Living Wall, Golden Wing, Cleric (Purify) |
| Needs combat | Mid-event combat trigger | Mushrooms, Dead Adventurer |
| Multi-step | Persistent event sub-state | Scrap Ooze, Golden Idol |

---

### Architecture notes

- `EventKind` enum — one variant per event (e.g. `WorldOfGoop`, `ShiningLight`, …)
- `EventRoomState { player, floor, graph, available_cols, event: EventKind }` — analogous to `RestSiteState`
- `GameState::EventRoom(EventRoomState)`
- `MapNode::Event` — event kind is selected randomly on entry (not stored in node)
- `Command::ChooseEventOption(usize)` — player picks option 0 / 1 / 2 / …
- Each event handler: `fn apply(event, option, player, rng) → (Player, Vec<Event>)`
- Events that need card selection will require an additional sub-state phase (later)

We can start with a simple fixed pool and pick randomly on entry.

---

## Rewards

| Encounter type | Gold         | Cards     | Other         |
|----------------|-------------|-----------|---------------|
| Normal combat  | 10–20g      | 3 options | 40% potion    |
| Elite combat   | 25–35g      | 3 options | 1 relic       |
| Boss           | 95–105g     | 3 rare options | —        |

---

## Current Implementation Gaps

| Gap | Status | Priority | Complexity |
|-----|--------|----------|------------|
| Floor count (10 → 16) | Open | High | Low |
| Fixed floors (Treasure at 9, Rest at 15) | Partial *(treasure done at idx 8)* | High | Low |
| Probabilistic room type distribution | Open | High | Medium |
| Multiple nodes per floor (up to 6) with edges | Open | High | High |
| `MapNode::Event` variant + event room pipeline | Open | High | Medium |
| Simple events (World of Goop, Shining Light, Ssssserpent) | Open | High | Low |
| Events with card selection (Living Wall, Cleric Purify) | Open | Medium | Medium |
| Curse mechanic | Open | Medium | Medium |
| `MapNode::Elite` variant + elite rewards (relic) | Open | High | Low |
| Act 1 elite enemies (Nob, Lagavulin, Sentries) | Open | High | High |
| Act 1 boss enemies (Slime Boss, Hexaghost, Guardian) | Open | High | High |
| Hard combat pool completeness | Open | Medium | Medium |
| Boss combat rewards (gold + 3 rare cards) | Open | Medium | Low |
| Gold rewards from combat (10–20g random) | Open | Medium | Low |
| Potion drop from combat (40% chance) | Open | Low | Low |

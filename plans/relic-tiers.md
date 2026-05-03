There are 88 relics total. Here's how they break down by what new infrastructure they need:

---

Tier 1 — Pickup effects only (on-acquire, no combat hooks)

- Strawberry / Pear / Mango — raise max HP
- Old Coin — gain gold
- Whetstone / War Paint — upgrade 2 random Attacks/Skills from deck on pickup

---

Tier 2 — Combat-start hook (fire once at combat init)

- Burning Blood / Black Blood — heal 6/12 HP at end of combat (needs end-combat hook too)
- Anchor — start with 10 block
- Vajra — start with 1 Strength (status already exists)
- Lantern — start with +1 max energy for the combat
- Blood Vial — heal 2 on combat start
- Bag of Marbles — apply 1 Vulnerable to all enemies
- Red Mask — apply 1 Weak to all enemies
- Festive Popper — deal 9 damage to all enemies
- Pantograph — heal 25 at Boss combat start
- Bag of Preparation — draw 2 extra cards

All of these need the same hook: a apply_relic_combat_start(player, enemies) pass. The end-of-combat counterpart (Burning Blood, Meat on the Bone) hooks into the existing
WinCombat/victory path.

---

Tier 3 — Turn boundary effects (fire on TurnStarted / TurnEnded)

- Mercury Hourglass — deal 3 to all on turn start
- Orichalcum — end turn with no block → gain 6 block
- Cloak Clasp — gain 1 block per card in hand at turn end
- Captain's Wheel — turn 3: gain 18 block
- Chandelier — turn 3: gain 3 energy
- Candelabra — turn 2: gain 2 energy
- Horn Cleat — turn 2: gain 14 block
- Happy Flower — every 3 turns: gain 1 energy (needs counter)
- Pendulum — every 3 turns: draw 1 card (needs counter)
- Stone Calendar — turn 7: deal 52 to all
- Regal Pillow — +15 HP on rest (hooks into rest, not combat)

---

Tier 4 — Card-play counters (track attacks/skills/powers played per turn)

- Nunchaku — every 10 attacks total → gain 1 energy
- Ornamental Fan — every 3 attacks this turn → gain 4 block
- Kunai — every 3 attacks this turn → gain 1 Dexterity (needs Dexterity status)
- Shuriken — every 3 attacks this turn → gain 1 Strength
- Kusarigama — every 3 attacks this turn → deal 6 to random enemy
- Letter Opener — every 3 skills this turn → deal 5 to all
- Tuning Fork — every 10 skills total → gain 7 block
- Pen Nib — every 10th attack: double damage (complex: modifies damage mid-card)
- Pocketwatch — ≤3 cards played → draw 3 extra next turn
- Game Piece — play Power → draw 1 card
- Intimidating Helmet — play card costing ≥2 → gain 4 block
- Permafrost — first Power played → gain 7 block
- Charon's Ashes — exhaust a card → deal 3 to all
- Joss Paper — every 5 exhausts → draw 1 card
- Gremlin Horn — enemy dies → gain 1 energy + draw 1 card

---

Tier 5 — HP-reactive or complex single-combat state

- Lizard Tail — death prevention, heal to 50% (one-time, needs "used" flag)
- Centennial Puzzle — first HP loss → draw 3 (needs "fired" flag per combat)
- Self-Forming Clay — lose HP → gain 3 block next turn (deferred effect)
- Beating Remnant — cap HP loss to 20 per turn
- Demon Tongue — first HP loss on your turn → heal that amount
- Red Skull — while HP ≤50%, gain +3 Strength (dynamic, recalculated each turn)
- Vambrace — first block gain → double it
- Ruined Helmet — first Strength gain → double it
- Art of War — no attacks played → +1 energy next turn
- Ripple Basin — no attacks played → gain 4 block at turn end
- Ice Cream — energy persists between turns
- Sturdy Clamp — up to 10 block persists across turns

---

Tier 6 — Needs new status types

- Akabeko — Vigor (next attack deals extra damage, then consumed)
- Bronze Scales — Thorns (deal damage when hit)
- Oddly Smooth Stone / Kunai — Dexterity (block cards give more block, or reduce incoming damage)
- Gorget — Plating (absorbs damage, doesn't expire)

---

Out of scope (potions, shop, ? rooms, elites): Potion Belt, Meal Ticket, Petrified Toad, The Courier, White Beast Statue, Prayer Wheel, White Star, Juzu Bracelet, Lasting Candy,
Tiny Mailbox, Planisphere, Shovel, Girya.

---

Recommended starting point: Implement the relic infrastructure (a Vec<Relic> on the player, a start_combat hook) and knock out all of Tier 1 + Tier 2 in one batch. That's ~15
relics with one new hook and gives the full acquisition → combat-start flow working end to end.

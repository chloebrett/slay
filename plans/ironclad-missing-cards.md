# Missing Ironclad Cards

All STS1 Ironclad cards not yet implemented. Cards marked ⚠️ need new infrastructure.

---

## Attacks (9 missing)

| Card | Cost | Description | Dependencies |
|------|------|-------------|--------------|
| **All-Out Attack** | 1 | Deal 10 damage to ALL enemies. Discard 1 card at random. | Random discard from hand |
| ~~**Immolate**~~ | ~~2~~ | ~~Deal 21 damage to ALL enemies. Add a Burn into your discard pile.~~ | ~~Burn card already exists~~ |
| ~~**Perfected Strike**~~ | ~~2~~ | ~~Deal 6 damage. Deals 2 additional damage for ALL your cards containing "Strike".~~ | ~~Scan all piles for "Strike" in name~~ |
| ~~**Feed**~~ | ~~2~~ | ~~Deal 10 damage. If Fatal, raise your Max HP by 3. Exhaust.~~ | ~~Kill-check + max HP raise~~ |
| **All for One** | 1 | Deal 10 damage. Put all cost-0 cards from your discard pile into your hand. | Filter discard by energy cost |
| ~~**Reaper**~~ | ~~2~~ | ~~Deal 4 damage to ALL enemies. Heal HP equal to unblocked damage. Exhaust.~~ | ~~AoE with per-hit heal~~ |
| ~~**Fiend Fire**~~ | ~~2~~ | ~~Exhaust your hand. Deal 7 damage for each card Exhausted. Exhaust.~~ | ~~Exhaust-hand loop~~ |
| ~~**Whirlwind**~~ | ~~X~~ | ~~Deal 5 damage to ALL enemies X times.~~ | ~~X-cost mechanic (spend all energy)~~ |
| **Searing Blow** ⚠️ | 2 | Deal 12 damage. Can be Upgraded any number of times. | Grade enum only has Base/Plus |

---

## Skills (10 missing)

| Card | Cost | Description | Dependencies |
|------|------|-------------|--------------|
| ~~**Flex**~~ | ~~0~~ | ~~Gain 2 Strength. At the end of this turn, lose 2 Strength.~~ | ~~Temporary-strength end-of-turn hook~~ |
| ~~**Intimidate**~~ | ~~0~~ | ~~Apply 1 Weak to ALL enemies. Exhaust.~~ | ~~AoE status already works~~ |
| ~~**Shockwave**~~ | ~~2~~ | ~~Apply 3 Weak and Vulnerable to ALL enemies. Exhaust.~~ | ~~AoE status already works~~ |
| **Second Wind** | 1 | Exhaust all non-Attack cards in your hand. Gain 5 Block for each card Exhausted. | Exhaust-by-type loop |
| **Sentinel** | 1 | Gain 5 Block. If this card is Exhausted, gain 2 Energy. | On-exhaust energy gain (new hook) |
| ~~**Power Through**~~ | ~~1~~ | ~~Add 2 Wounds into your hand. Gain 15 Block.~~ | ~~Wound card exists; add-to-hand~~ |
| ~~**Ghostly Armor**~~ | ~~1~~ | ~~Ethereal. Gain 13 Block.~~ | ~~Ethereal mechanic already exists~~ |
| ~~**Burning Pact**~~ | ~~1~~ | ~~Exhaust 1 card. Draw 2 cards.~~ | ~~Choose-a-card-to-exhaust UI~~ |
| ~~**Armaments**~~ | ~~1~~ | ~~Gain 5 Block. Upgrade a card in your hand for the rest of combat.~~ | ~~Choose-a-card-to-upgrade UI~~ |
| ~~**Warcry**~~ | ~~0~~ | ~~Draw 1 card. Put a card from your hand onto the top of your draw pile. Exhaust.~~ | ~~Choose-a-card-to-topdeck UI~~ |

---

## Powers (0 missing)

All Ironclad powers are implemented.

---

## Infrastructure gaps (required by multiple cards)

- **AoE status application** — Apply a status to all enemies. Needed by: Intimidate, Shockwave. (Thunderclap already does AoE damage; same pattern.)
- **Temporary strength** — Strength that expires at end of turn. Needed by: Flex. Could use a new `StrengthLoss` end-of-turn hook or a `TemporaryStrength` status.
- **X-cost mechanic** — Card costs all remaining energy; X = energy spent. Needed by: Whirlwind.
- **Infinite upgrades** — Card grade beyond Base/Plus. Needed by: Searing Blow.
- **On-exhaust energy gain** — Sentinel grants energy when exhausted (not when played). Needed by: Sentinel.
- ~~**Choose-a-card UI**~~ — ~~Player selects a card from hand to exhaust/upgrade/topdeck. Needed by: Burning Pact, Armaments, Warcry.~~ Done: `CombatPhase::ChooseCard(ChooseCardContext)` + `Command::ChooseHandCard`.
- **Kill-check** — Did this hit kill the enemy? Needed by: Feed.

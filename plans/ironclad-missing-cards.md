# Missing Ironclad Cards

All STS1 Ironclad cards not yet implemented. Cards marked ⚠️ need new infrastructure.

---

## Attacks (2 missing)

| Card | Cost | Description | Dependencies |
|------|------|-------------|--------------|
| **All-Out Attack** | 1 | Deal 10 damage to ALL enemies. Discard 1 card at random. | Random discard from hand |
| **All for One** | 1 | Deal 10 damage. Put all cost-0 cards from your discard pile into your hand. | Filter discard by energy cost |
| **Searing Blow** ⚠️ | 2 | Deal 12 damage. Can be Upgraded any number of times. | Grade enum only has Base/Plus |

---

## Skills (2 missing)

| Card | Cost | Description | Dependencies |
|------|------|-------------|--------------|
| ~~**Second Wind**~~ | ~~1~~ | ~~Exhaust all non-Attack cards in your hand. Gain 5 Block for each card Exhausted.~~ | ~~Exhaust-by-type loop~~ |
| ~~**Sentinel**~~ | ~~1~~ | ~~Gain 5 Block. If this card is Exhausted, gain 2 Energy.~~ | ~~On-exhaust energy gain (new hook)~~ |

---

## Powers (0 missing)

All Ironclad powers are implemented.

---

## Infrastructure gaps (required by remaining cards)

- **On-exhaust energy gain** — Sentinel grants energy when exhausted (not when played). Needed by: Sentinel.
- **Infinite upgrades** — Card grade beyond Base/Plus. Needed by: Searing Blow.

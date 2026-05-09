# Plan: Snecko

**Branch**: main
**Status**: Active

## Goal

Implement Snecko as a playable Act 2 enemy, including its Confused debuff that randomises card costs on draw.

## Background

Snecko (114–120 HP) is a notable Act 2 encounter whose passive — Confused — inverts the usual tension of a low-cost deck. It always opens with Perplexing Glare (applies Confused to the player), then alternates Bite (15 damage, 60%) and Tail Whip (8 damage + 2 Vulnerable, 40%), with no 3-in-a-row Bite constraint.

**Confused mechanics**
- Applies to the *player* for the rest of the combat (never ticks down).
- Each card has its cost randomised to 0, 1, 2, or 3 when drawn.
- The randomised cost is set on draw and stays stable while the card is in hand.
- It resets to base cost if the card is shuffled back to the draw/discard pile and redrawn.

## Acceptance Criteria

- [ ] `EnemyKind::Snecko` appears in combat with `max_hp = Hp(114)` (baseline, no ascension).
- [ ] Snecko's first move is always Perplexing Glare.
- [ ] After turn 1, Snecko uses Bite (15 damage) with ~60% probability and Tail Whip (8 damage + 2 Vulnerable) with ~40% probability.
- [ ] Bite cannot occur three turns in a row; Tail Whip has no such constraint.
- [ ] `EnemyKind::Snecko` round-trips through `id()` / `from_id()` as `"snecko"`.
- [ ] `StatusEffect::Confused` exists and does not tick down at end of turn.
- [ ] Perplexing Glare applies `Confused` to the player (visible as a debuff event).
- [ ] While the player has `Confused`, each card drawn is assigned a random cost in {0, 1, 2, 3} that is used when checking affordability and deducting energy.
- [ ] The randomised cost is determined at draw time and does not change while the card remains in hand.
- [ ] A card whose cost was randomised plays at the randomised cost, not its base cost.

## Architecture note: per-card cost overrides

`CardCost` is currently computed on demand from `CardDef` — nothing on `Card` stores an override. Confused requires a stable, per-instance cost that is set at draw time.

Recommended approach: add `hand_cost_overrides: Vec<Option<Energy>>` to `Player`, maintained parallel to `hand`. When Confused is active, `draw_cards` (or `draw_with_triggers`) assigns a random value; `apply_play_card` reads the override instead of `card.card_cost()` when present; removing a card from hand also removes its override entry.

This is contained to `Player` and `combat.rs` without touching the `Card` enum.

## Steps

Every step follows RED → GREEN → MUTATE → KILL MUTANTS → REFACTOR.

---

### Step 1: Register Snecko — EnemyKind, moves, and move selection

**RED**: Write tests in `enemies/mod.rs` tests section:
- `snecko_has_114_hp`
- `snecko_id_round_trips` (id = "snecko")
- `snecko_name_is_snecko`
- `snecko_first_move_is_perplexing_glare`
- `snecko_after_first_turn_uses_bite_or_tail_whip`
- `snecko_never_bites_three_times_in_a_row`
- `perplexing_glare_intent_is_debuff`
- `snecko_bite_is_attack_15`
- `snecko_tail_whip_is_attack_debuff_8`

**GREEN**:
- Create `crates/slay-core/src/enemies/snecko.rs` with `DEF` and `next_move`:
  - First move (empty history): `SneckoPerplexingGlare`.
  - Subsequent: weighted 60/40 with constraint — if the last two moves were both `SneckoBite`, force `SneckoTailWhip`.
- Add `mod snecko;` to `enemies/mod.rs`.
- Add `Snecko` to `EnemyKind` enum, wire `def()` / `id()` / `from_id()` / `next_move()`.
- Add `SneckoPerplexingGlare`, `SneckoBite`, `SneckoTailWhip` to `Move` enum.
- Wire `Move::def()` for each:
  - `SneckoPerplexingGlare` → `effects: vec![Effect::ApplyStatus(StatusEffect::Confused, 1)]`
    (Note: `StatusEffect::Confused` does not exist yet — stub with a placeholder or add it now.)
  - `SneckoBite` → `effects: vec![Effect::DealDamage(15)]`
  - `SneckoTailWhip` → `effects: vec![Effect::DealDamage(8), Effect::ApplyStatus(StatusEffect::Vulnerable, 2)]`

**MUTATE**: Run `cargo mutants -p slay-core` filtered to `snecko.rs` and the new `next_move` dispatch. Produce killed/survived report.

**KILL MUTANTS**: Address surviving mutants; ask if value is ambiguous (e.g. off-by-one in the 3-in-a-row guard).

**REFACTOR**: Assess — is the no-triple-bite logic clear? If not, extract a named helper.

**Done when**: All new tests pass; `EnemyKind::Snecko` is a valid, fully registered enemy.

---

### Step 2: Add `StatusEffect::Confused` and apply it via Perplexing Glare

**RED**: Write tests in `status.rs` tests section (or `enemies/mod.rs`):
- `confused_does_not_tick_at_end_of_turn` — after `tick_statuses`, Confused stacks unchanged.
- `perplexing_glare_applies_confused_to_player` — `Move::SneckoPerplexingGlare.def().effects` contains `Effect::ApplyStatus(StatusEffect::Confused, 1)`.
- `perplexing_glare_intent_is_debuff` (already listed in Step 1; confirm it still passes with real variant).

**GREEN**:
- Add `Confused` to `StatusEffect` enum in `status.rs`.
- `ticks_at_end_of_turn()` must **not** include `Confused` (it persists for the whole combat).
- `SneckoPerplexingGlare.def()` already references `StatusEffect::Confused` from Step 1 — remove any placeholder.

**MUTATE**: Run mutation testing on `status.rs` (tick logic) and the Perplexing Glare def.

**KILL MUTANTS**: Address survivors.

**REFACTOR**: Assess.

**Done when**: `StatusEffect::Confused` exists, doesn't tick, and is applied by Perplexing Glare in combat.

---

### Step 3: Implement Confused — randomise card costs on draw

This step adds `hand_cost_overrides` to `Player` and wires it through draw and play.

**RED**: Write tests in `combat.rs` tests section:
- `confused_player_gets_randomised_cost_on_draw` — construct a `CombatState` with `StatusEffect::Confused` pre-applied to player; draw a card; assert that the entry in `hand_cost_overrides` is in [0, 3].
- `confused_card_plays_at_randomised_cost_not_base` — a Strike (base cost 1) drawn with Confused and assigned cost 0 is playable with 0 energy; deducts 0 energy.
- `confused_card_plays_at_randomised_cost_higher_than_base` — a Strike drawn with Confused cost 3 requires 3 energy and deducts 3.
- `non_confused_player_has_no_cost_overrides` — drawing without Confused leaves `hand_cost_overrides` empty / all `None`.
- `cost_override_cleared_when_card_played` — after playing a card, its override entry is removed.
- `cost_override_cleared_when_card_discarded` — e.g. end of turn discard removes overrides.

**GREEN**:
- Add `hand_cost_overrides: Vec<Option<Energy>>` to `Player`.
- Extend `draw_cards` (or `draw_with_triggers`) to call `rng.choose(&mut [0,1,2,3])` for each card when player has `Confused`, and push the result into `hand_cost_overrides`.
- In `apply_play_card`, resolve effective cost as:
  ```
  hand_cost_overrides[index].map(CardCost::Fixed).unwrap_or_else(|| card.card_cost())
  ```
- Keep `hand_cost_overrides` in sync with `hand` wherever cards are removed (play, discard, exhaust).
- Add `Rng::choose_from_four` or extend existing `choose` to handle `[Energy; 4]`, or reuse `rng.choose(&mut [Energy(0), Energy(1), Energy(2), Energy(3)])`.

**MUTATE**: Run mutation testing on the cost-override path in `combat.rs`.

**KILL MUTANTS**: Address survivors; ask on any ambiguous mutation in bounds checks.

**REFACTOR**: Assess — can the sync logic be centralized? Does `hand_cost_overrides` need to be part of serialization (for save/load)?

**Done when**: A player with `Confused` in `CombatState` plays cards at their randomised costs; UI (both plain and TUI renderers) shows the overridden cost if it reads from the same `CombatState`.

---

## Pre-PR Quality Gate

Before each PR:
1. Mutation testing — run `mutation-testing` skill; report killed/survived/score.
2. Refactoring assessment — run `refactoring` skill.
3. `cargo clippy --all-targets -- -D warnings` passes.
4. `cargo test -p slay-core` passes (modulo pre-existing failures in neow tests).
5. `cargo test -p slay-tui --test scripts` — update any affected snapshots with `INSTA_UPDATE=always`.

---
*Delete this file when the plan is complete.*

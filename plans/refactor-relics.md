# Plan: Refactor Relics into Per-File Modules

**Branch**: main
**Status**: Active — Step 1 complete, Step 2 pending

## Goal

Split the monolithic `relics.rs` (1600 lines) into a `relics/` directory where each relic lives in its own file, and `mod.rs` is a thin router — mirroring the `cards/` architecture.

## Why

The card refactor taught us that putting each thing in its own file makes the codebase:
- Easier to navigate (find a relic's full behavior in one place)
- Easier to extend (add a relic = create one file, not edit 8 match blocks)
- Easier to read (each dispatcher is a thin router, not a wall of logic)

The analogous problem in relics: a relic's behavior is currently scattered across up to 8 match arms in 8 different hook functions. To understand what Pocketwatch does, you scan the whole file.

## Target Structure

```
src/
  relics/
    mod.rs          ← Relic enum, impl (id/all/from_id), hook dispatchers, shared helpers
    tests.rs        ← all tests (moved verbatim from mod.rs)
    anchor.rs
    bag_of_marbles.rs
    bag_of_preparation.rs
    black_blood.rs
    blood_vial.rs
    burning_blood.rs
    candelabra.rs
    captains_wheel.rs
    chandelier.rs
    cloak_clasp.rs
    festive_popper.rs
    gremlin_horn.rs
    happy_flower.rs
    horn_cleat.rs
    kunai.rs
    kusarigama.rs
    lantern.rs
    letter_opener.rs
    mango.rs
    mercury_hourglass.rs
    nunchaku.rs
    old_coin.rs
    orichalcum.rs
    ornamental_fan.rs
    pantograph.rs
    pear.rs
    pendulum.rs
    pocketwatch.rs
    red_mask.rs
    regal_pillow.rs
    shuriken.rs
    stone_calendar.rs
    strawberry.rs
    tuning_fork.rs
    vajra.rs
    war_paint.rs
    whetstone.rs
```

## Per-File Contract

Each relic file implements only the hooks it needs, plus `id()`:

```rust
// strawberry.rs
use crate::combat::Player;

pub(super) fn id() -> &'static str { "strawberry" }

pub(super) fn on_grant(player: &mut Player) {
    super::raise_max_hp(player, 7);
}
```

```rust
// burning_blood.rs
use crate::combat::{Event, Player};

pub(super) fn id() -> &'static str { "burning-blood" }

pub(super) fn on_combat_end(player: &mut Player, events: &mut Vec<Event>) {
    super::heal_player(player, 6, events);
}
```

```rust
// nunchaku.rs
use crate::cards::CardType;
use crate::combat::{CombatState, Event};
use crate::rng::Rng;

pub(super) fn id() -> &'static str { "nunchaku" }

pub(super) fn on_card_play(state: &mut CombatState, events: &mut Vec<Event>, card_type: CardType, _rng: &mut impl Rng) {
    if card_type == CardType::Attack && state.attacks_this_combat.is_multiple_of(10) {
        state.player.energy.0 += 1;
        events.push(Event::EnergyGained { amount: 1 });
    }
}
```

## Hook Signatures

Every relic file uses these exact signatures for the hooks it implements:

| Hook | Signature |
|------|-----------|
| `on_grant` | `fn on_grant(player: &mut Player, events: &mut Vec<Event>, rng: &mut impl Rng)` |
| `on_combat_start` | `fn on_combat_start(state: &mut CombatState, events: &mut Vec<Event>, rng: &mut impl Rng, is_boss: bool)` |
| `on_turn_start` | `fn on_turn_start(state: &mut CombatState, events: &mut Vec<Event>, rng: &mut impl Rng)` |
| `on_turn_end` | `fn on_turn_end(state: &mut CombatState, events: &mut Vec<Event>, hand_size_before_discard: usize)` |
| `on_card_play` | `fn on_card_play(state: &mut CombatState, events: &mut Vec<Event>, card_type: CardType, rng: &mut impl Rng)` |
| `on_enemy_died` | `fn on_enemy_died(state: &mut CombatState, events: &mut Vec<Event>, rng: &mut impl Rng)` |
| `on_combat_end` | `fn on_combat_end(player: &mut Player, events: &mut Vec<Event>)` |
| `on_rest` | `fn on_rest(player: &mut Player, events: &mut Vec<Event>)` |

Hooks that don't need a parameter (e.g. `rng` for `on_grant` for Strawberry) still use the full signature — unused parameters are named `_rng`. This keeps the dispatcher in mod.rs uniform.

Exception: `on_grant` for Tier 1 relics (Strawberry, Pear, Mango, OldCoin, Whetstone, WarPaint) doesn't need `events` or `rng` in most cases, but the signature stays consistent with the full form so mod.rs can call them all uniformly.

Actually simpler: each hook function takes only what it needs. mod.rs passes all available params; each file ignores what it doesn't use with `_`.

## mod.rs After Refactor

### Relic enum and id dispatch

```rust
impl Relic {
    pub fn id(&self) -> &'static str {
        match self {
            Relic::Strawberry       => strawberry::id(),
            Relic::BurningBlood     => burning_blood::id(),
            // ... one arm per relic
        }
    }
}
```

### Hook dispatchers (thin routers)

```rust
pub fn apply_turn_start_relics(state: &mut CombatState, events: &mut Vec<Event>, rng: &mut impl Rng) {
    let relics = state.player.relics.clone();
    for relic in &relics {
        match relic {
            Relic::MercuryHourglass => mercury_hourglass::on_turn_start(state, events, rng),
            Relic::CaptainsWheel    => captains_wheel::on_turn_start(state, events, rng),
            Relic::Chandelier       => chandelier::on_turn_start(state, events, rng),
            Relic::Candelabra       => candelabra::on_turn_start(state, events, rng),
            Relic::HornCleat        => horn_cleat::on_turn_start(state, events, rng),
            Relic::HappyFlower      => happy_flower::on_turn_start(state, events, rng),
            Relic::Pendulum         => pendulum::on_turn_start(state, events, rng),
            Relic::StoneCalendar    => stone_calendar::on_turn_start(state, events, rng),
            _ => {}
        }
    }
    if state.enemies.iter().all(|e| e.hp <= Hp(0)) {
        state.phase = CombatPhase::Victory;
    }
}
```

### Shared helpers stay in mod.rs

`heal_player`, `raise_max_hp`, `damage_all_enemies`, `damage_random_living_enemy`, `upgrade_random_of_type` stay in `mod.rs` as `pub(super) fn` — used by multiple relic files via `super::heal_player(...)`.

## Hook-to-Relic Mapping

Which relics implement which hooks (for writing each file):

| Hook | Relics |
|------|--------|
| `on_grant` | Strawberry, Pear, Mango, OldCoin, Whetstone, WarPaint |
| `on_combat_start` | Anchor, Vajra, Lantern, BloodVial, BagOfMarbles, RedMask, FestivePopper, Pantograph, BagOfPreparation |
| `on_turn_start` | MercuryHourglass, CaptainsWheel, Chandelier, Candelabra, HornCleat, HappyFlower, Pendulum, StoneCalendar |
| `on_turn_end` | Orichalcum, CloakClasp, Pocketwatch |
| `on_card_play` | Nunchaku, OrnamentalFan, Shuriken, Kunai, Kusarigama, LetterOpener, TuningFork |
| `on_enemy_died` | GremlinHorn |
| `on_combat_end` | BurningBlood, BlackBlood |
| `on_rest` | RegalPillow |

## Steps

### Step 1: Split relics.rs into relics/

Pure refactor — no logic changes, all tests already exist.

**Acceptance criteria**:
- `relics.rs` is gone; `relics/mod.rs` has the Relic enum, `id()`/`all()`/`from_id()` (dispatching to per-file `id()`), the 8 thin hook dispatchers, and the 5 shared helpers
- Each of the 37 relic files exists with its `id()` and hook implementation(s)
- `relics/tests.rs` has all tests moved verbatim
- All existing tests pass, `cargo clippy` is clean

**Approach**: Mechanical split — no logic changes. Create `relics/mod.rs` from the existing file, extract each relic into its own file, replace inline match-arm logic with calls to `relic_name::on_hook(...)`.

**Done when**: `cargo test` green, `cargo clippy` clean, human approves commit.

---

### Step 2: Add RelicDef and fix relic name display

**Acceptance criteria**:
- `RelicDef { name: &'static str }` exists in `relics/mod.rs`
- Each relic file has `pub(super) fn def() -> RelicDef` returning its proper display name (e.g. `"Burning Blood"`, not `"burning-blood"`)
- `Relic::name()` is a convenience method (delegates to `def().name`)
- Both renderers (plain text `game.rs` and ratatui `tui.rs`) use `relic.name()` instead of `relic.id()` wherever relics are shown to the player
- New test: `relic_name_is_human_readable` — asserts a sample of relics return non-id-style names (no hyphens, properly capitalised)
- All existing tests still pass, snapshot tests updated if output changed

**RED**: Write the test asserting `Relic::BurningBlood.name() == "Burning Blood"` (and a few others). It fails because `name()` doesn't exist.

**GREEN**: Add `RelicDef`, add `def()` to each relic file, add `Relic::name()`, update renderers.

**MUTATE**: Run `cargo mutants -p slay-core` on the relics module — produce report.

**KILL MUTANTS**: Address survivors.

**REFACTOR**: Assess.

**Done when**: Tests green, display shows proper names, mutation report reviewed, human approves commit.

## Pre-PR Quality Gate

1. `cargo test --all` passes
2. `cargo clippy --all -- -D warnings` passes
3. Snapshot tests pass: `cargo test -p slay-tui --test scripts`

---
*Delete this file when the plan is complete. If `plans/` is empty, delete the directory.*

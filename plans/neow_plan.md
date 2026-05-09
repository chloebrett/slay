# Neow Plan

Neow is the whale NPC who greets the player at the start of every run and offers a blessing before the map begins.

## What Neow Offers

The blessing menu has two modes depending on the previous run:

### First run (no meta save) — 2 options
1. Max HP +8
2. Neow's Lament — enemies in the next 3 combats have 1 HP

### Subsequent runs — 4 options (one from each category)

| # | Category | Possible choices (one picked randomly) |
|---|---|---|
| 1 | Card boost | Remove a card · Transform a card · Upgrade a card · Choose from 3 rare cards |
| 2 | Safe reward | Max HP +8 · Neow's Lament · Random common relic · 100 gold · 3 random potions |
| 3 | Risk/reward | Lose HP + (remove 2 cards / transform 2 cards / 250 gold / random rare relic) · Obtain a curse + big reward |
| 4 | Boss relic swap | Replace starter relic with a random boss relic |

The category 4 option only appears if the **previous run reached the Act 1 boss** (`is_boss = true` at some point in the run). Otherwise slot 4 is omitted and there are 3 options.

---

## New GameState Variant

```rust
// run.rs
GameState::Neow(NeowState)
```

```rust
pub struct NeowState {
    pub player: Player,
    pub graph: MapGraph,
    pub blessings: Vec<NeowBlessing>,
}
```

Inserted between `new_run` and the first `Map` state. `new_run` returns `GameState::Neow(...)` instead of `GameState::Map(...)`.

The blessings are generated once at run start (seeded RNG) and stored in `NeowState` — no re-roll on resume.

---

## NeowBlessing Enum

```rust
pub enum NeowBlessing {
    // Safe options
    GainMaxHp(i32),          // +8 HP
    NeowsLament,             // next 3 combats: enemies at 1 HP
    GainGold(i32),           // 100 gold
    GainRelic(Relic),        // common relic
    GainPotions(Vec<Potion>),// 3 random potions

    // Card options
    RemoveCard,              // choose a card to remove from deck
    TransformCard,           // choose a card; replace with random other card of same type
    UpgradeCard,             // choose a card to upgrade
    ChooseRareCard(Vec<Card>),// pick one of 3 rare cards

    // Risk/reward
    LoseHpGainGold { hp_loss: i32, gold: i32 },        // -7 HP, +250 gold
    LoseHpRemoveCards { hp_loss: i32, count: usize },   // -7 HP, remove 2 cards
    LoseHpTransformCards { hp_loss: i32, count: usize },// -7 HP, transform 2 cards
    LoseHpGainRareRelic { hp_loss: i32, relic: Relic }, // -7 HP, random rare relic
    ObtainCurseGainRareRelic { curse: Card, relic: Relic },

    // Boss swap
    SwapStarterRelic(Relic), // replace starter relic with a boss relic
}
```

Multi-step blessings (RemoveCard, TransformCard, UpgradeCard) need a sub-selection phase. These are handled by a `NeowSubPhase` on `NeowState` — the player picks a blessing, if it requires a sub-choice the state enters sub-phase, then resolves.

---

## Neow's Lament Mechanic

"Next 3 combats, enemies have 1 HP" requires tracking a counter across combats. Two options:

**Option A — Counter on Player:** add `neow_lament_combats_remaining: u32` to `Player`. Decrements each time a combat starts, active while > 0. Automatically serialized as part of `Player`.

**Option B — Counter in MetaSave or run-level state:** requires a new run-level field.

**Decision: Option A.** Keeps the effect self-contained in the player snapshot that already travels through every state variant.

---

## MetaSave Changes

Add two fields to track what the previous run achieved:

```rust
pub struct MetaSave {
    pub runs_completed: u32,
    pub runs_won: u32,
    pub prev_run_reached_boss: bool, // enables category-4 blessing
}
```

`prev_run_reached_boss` is set to `true` in `on_run_end` whenever the player entered a boss combat (detected from the `GameOver` path: a run that reached `is_boss = true` at any point). The simplest implementation: set a flag on `Player` (`reached_boss: bool`) that gets set to `true` when entering a boss combat, then read at run end.

---

## Command

```rust
Command::ChooseNeowBlessing(usize)   // 0-indexed
```

Added to the existing `Command` enum. In the Neow phase, `apply_command` resolves the blessing and transitions to `GameState::Map`.

For blessings requiring sub-selection (RemoveCard, UpgradeCard, TransformCard), the state stays in `GameState::Neow` but with `sub_phase: Some(NeowSubPhase::RemoveCard)` set, and `ChooseHandCard` / a new `ChooseNeowCard(usize)` command resolves the sub-step.

---

## Generation Logic

```rust
fn generate_blessings(rng: &mut impl Rng, meta: &MetaSave) -> Vec<NeowBlessing>
```

Lives in `run.rs`. Called from `new_run`, which now takes `&MetaSave` as a parameter.

- First run (`meta.runs_completed == 0`): return `[GainMaxHp(8), NeowsLament]`
- Subsequent runs: pick one from each of categories 1–3 randomly, add category 4 only if `meta.prev_run_reached_boss`

---

## new_run Signature Change

```rust
// Before
pub fn new_run(rng: &mut impl Rng) -> GameState

// After
pub fn new_run(rng: &mut impl Rng, meta: &MetaSave) -> GameState
```

`main.rs` loads meta before calling `new_run` and passes it in. This is a breaking change to the public API — all callers (tests, `main.rs`) need updating. Test callers can use `&MetaSave::default()`.

---

## UI

### Plain text (`game.rs`)

```
🐳 Neow's Blessings
Choose a blessing to begin your run:

  [1] ❤️  Gain 8 Max HP
  [2] 👁️  Neow's Lament — enemies have 1 HP for the next 3 combats
  [3] 💰 Gain 100 gold
  [4] 🔄 Swap starter relic for a random boss relic
```

### TUI (`tui.rs`)

New render branch in `render_frame` for `GameState::Neow`. List of blessings with numeric selection, same as card reward screen in style.

---

## Snapshot Tests

Add a `scripts/simple/neow-*.slay` script per interesting path:
- `neow-first-run.slay` — picks max HP on a first run
- `neow-lament.slay` — picks Neow's Lament, enters combat, verifies enemy at 1 HP
- `neow-remove-card.slay` — picks RemoveCard, chooses a card, verifies deck shrunk

---

## Implementation Order

1. Add `prev_run_reached_boss` to `MetaSave` + `MetaSaveFile`. Update `on_run_end` to set it.
2. Add `neow_lament_combats_remaining: u32` to `Player`.
3. Add `NeowBlessing` enum to `slay-core` (new file `neow.rs`). Derive serde.
4. Add `NeowState` + `GameState::Neow` variant. Derive serde (free — types already derived).
5. Add `Command::ChooseNeowBlessing(usize)`.
6. Implement `generate_blessings(rng, meta)` and update `new_run` signature.
7. Implement `apply_command` arm for `GameState::Neow`.
8. Wire Neow's Lament: hook in `apply_combat_start_relics` (or a new `apply_neow_lament` called at combat start) that sets enemy HP to 1 when counter > 0, then decrements.
9. Add plain text renderer for Neow (`game.rs`).
10. Add TUI renderer for Neow (`tui.rs`).
11. Add snapshot scripts and integration tests.

Each step is independently committable.

---

## Open Questions

- Sub-selection UX for RemoveCard/TransformCard/UpgradeCard: reuse the existing `ChooseCard` flow (already exists for BurningPact/Warcry) or a dedicated Neow sub-phase? Leaning toward reusing `ChooseCardContext`.
- Should the boss relic pool exclude relics already in the relic pool for this run? (In real STS, yes.) Decision deferred to implementation.
- Neow's Lament counter: cap at 3 combats. What if the player has already fought 3 combats before picking it? (Can't happen — Neow resolves before the first combat.)

# Plan: Damage Flash

**Status**: Pending

## Goal

Brief red flash on an HP bar when its owner takes damage — enemy bars when the player attacks, player bar when an enemy attacks. Gives immediate tactile feedback that a hit landed.

## Design

### Flash state

```rust
pub struct TuiState {
    // ...
    pub player_flash: Option<std::time::Instant>,
    pub enemy_flashes: Vec<Option<std::time::Instant>>,
}
```

`enemy_flashes` is indexed by enemy slot (same order as `CombatState.enemies`). It is reset to `vec![None; enemies.len()]` whenever combat is entered.

### Triggering

After `apply_and_drain` returns, scan the event list:

- `Event::EnemyAttacked { damage, index, .. }` where `damage > 0` → set `enemy_flashes[index]`
- `Event::PlayerAttacked { damage, .. }` where `damage > 0` → set `player_flash`
- `Event::PlayerSelfDamaged { .. }` → set `player_flash`

### Duration

```rust
const FLASH_DURATION: Duration = Duration::from_millis(200);
```

Same auto-expiry pattern as `wipe_start`: check all timers before `event::poll` in the event loop, clear any that have elapsed.

### Rendering

When a flash is active, override the normal HP bar colour to `Color::Red` (bright, not the same dim red used for low HP). This only affects the styled HP span — the rest of the line is unchanged.

- `render_enemies` receives `enemy_flashes: &[Option<Instant>]` and uses `enemy_flashes[i]` to decide whether to override colour for enemy `i`.
- `render_top_bar` receives `player_flash: Option<Instant>` and overrides the HP span colour when set.

The flash colour (`Color::LightRed`) is distinct from the low-HP danger colour (`Color::Red`) so the two signals don't blend together.

### Blocker: EnemyAttacked needs an index

`Event::EnemyAttacked` currently has no enemy index, so it's impossible to know which enemy was hit. This must be fixed in `slay-core` first:

```rust
// Before
EnemyAttacked { raw: i32, damage: i32 }

// After
EnemyAttacked { index: usize, raw: i32, damage: i32 }
```

All call sites and match arms must be updated. Snapshot tests will need regenerating if `describe_event` renders the index.

---

## Acceptance Criteria

- [ ] `Event::EnemyAttacked` carries `index: usize`; all existing tests still pass
- [ ] `enemy_flashes` defaults to empty `vec![]`; resets to correct length when combat is entered
- [ ] `player_flash` defaults to `None`
- [ ] Playing a card that deals damage > 0 sets `enemy_flashes[target_index]` for the hit enemy
- [ ] An enemy attack that deals damage > 0 to the player sets `player_flash`
- [ ] Self-damage (e.g. `PlayerSelfDamaged`) also sets `player_flash`
- [ ] Attacks that deal 0 damage (fully blocked) do NOT set a flash
- [ ] Flash colour is `Color::LightRed`, distinct from the low-HP `Color::Red`
- [ ] Flash expires automatically after 200ms without a keypress
- [ ] In a multi-enemy fight, only the struck enemy's bar flashes; others are unaffected

---

## Steps

Every step follows RED-GREEN-MUTATE-KILL MUTANTS-REFACTOR.

### Step 1: Add index to EnemyAttacked (slay-core)

Add `index: usize` to `Event::EnemyAttacked`. Fix all call sites (pass the correct enemy index). Fix all match arms in tests and `describe_event`. Regenerate snapshots if needed.

**Done when**: `cargo test` all green, no compilation errors.

### Step 2: Flash state on TuiState

Add `player_flash: Option<Instant>` and `enemy_flashes: Vec<Option<Instant>>` to `TuiState`. Default both to empty/None in `new()`. When handle_enter transitions to Combat, reset `enemy_flashes` to `vec![None; n_enemies]`.

**Tests**: default is None/empty; reset happens on combat entry.

### Step 3: Trigger flash from events

In `handle_enter`, after `apply_and_drain`, scan returned events. Set `player_flash` on `PlayerAttacked`/`PlayerSelfDamaged` with `damage > 0`. Set `enemy_flashes[index]` on `EnemyAttacked` with `damage > 0`.

**Tests**: after a damaging play command, correct flash field is set; zero-damage hit does not set flash.

### Step 4: Flash expiry

Add flash timer expiry to the event loop (before `event::poll`), mirroring the `wipe_start` pattern.

**Tests**: set flash, advance past FLASH_DURATION, verify cleared (using elapsed-check logic in unit test).

### Step 5: Render the flash

Pass flash state into `render_enemies` and `render_top_bar`. When flash is active, use `Color::LightRed` for the HP span instead of the normal `hp_color` result.

**Tests**: render with flash active → HP text has different colour signal (assert `LightRed` used); render without flash → normal colour unchanged.

---

*Delete this file when the plan is complete.*

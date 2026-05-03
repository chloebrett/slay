# Architecture Reference

This document describes the current implementation as a spec. Read it to understand where everything lives, what contracts each piece holds, and which design decisions are load-bearing versus intentionally left open for future extension.

---

## Crate Layout

```
slay/
  Cargo.toml                  ← workspace root
  crates/
    slay-core/                ← pure game engine; no I/O, no terminal
      src/
        lib.rs                ← public re-exports; the crate's API surface
        run.rs                ← GameState, Command, CommandError, apply_command, MAP_NODES, Scenario
        combat.rs             ← CombatState, apply_combat_command, Player, Enemy, Event
        cards/
          mod.rs              ← Card enum, CardDef, CardDescription, dispatch, reward_pool, starter_deck
          strike.rs           ← strike::apply
          defend.rs           ← defend::apply
          bash.rs             ← bash::apply
          clothesline.rs      ← clothesline::apply
          inflame.rs          ← inflame::apply
          deadly_poison.rs    ← deadly_poison::apply
          disarm.rs           ← disarm::apply
        enemies/
          mod.rs              ← EnemyKind, EnemyDef, Intent, next_intent dispatch; id()/from_id()
          louse.rs            ← louse::DEF, louse::next_intent
        status.rs             ← StatusEffect, StatusMap, resolve_damage, tick_statuses, drain_poison
        types.rs              ← Hp, Block, Energy newtypes
        rng.rs                ← Rng trait, ThreadRng, NoOpRng, AnyRng
    slay-tui/
      src/
        main.rs               ← CLI entry point; routes to tui or game based on flags + TTY
        engine.rs             ← apply_and_drain + event/intent/status/icon formatters (shared)
        game.rs               ← run_game(state, reader, writer, rng, debug) — plain text loop
        tui.rs                ← run_tui(state, rng, debug) — ratatui interactive UI
        command.rs            ← text + GameState context → Command (parse)
        lib.rs                ← re-exports command, engine, game, tui modules
      tests/
        integration.rs        ← TestHarness; command sequences → GameState assertions
        scripts.rs            ← insta snapshot harness; discovers tests/scripts/*.slay
        scripts/              ← deterministic .slay scripts (test fixtures)
          01-louse-end-turn.slay
          02-add-strike-kill-louse.slay
          ...
        snapshots/            ← committed insta snapshot files (auto-managed)
```

**Hard constraint on `slay-core`:** No terminal, I/O, or display code. No `println!`. A hypothetical `slay-gui` could depend on `slay-core` unchanged.

**Hard constraint on `slay-tui`:** No game logic. It only translates: text → `Command`, `Command` → `slay-core`, `GameState` → terminal output.

---

## The Public API (`slay-core`)

Everything the TUI needs is re-exported from `lib.rs`. Nothing else is public to external crates.

### Top-level entry point

```rust
// run.rs
pub fn apply_command(
    state: GameState,
    command: Command,
    rng: &mut impl Rng,
) -> Result<(GameState, Vec<Event>), CommandError>
```

`GameState` is fully owned. `apply_command` returns a new state — the original is consumed. This makes the engine trivially testable and replayable. The TUI clones state before calling when it needs to retain the old state on error.

### Commands

```rust
// run.rs
pub enum Command {
    // Combat
    PlayCard(usize),       // 0-indexed hand position
    EndTurn,
    EndEnemyTurn,          // internal — TUI auto-drains; player never issues this

    // Map navigation
    ChooseNode(usize),     // currently always 0 (only one choice per floor)
    Spawn(Vec<EnemyKind>), // debug/Simple: override next combat's enemies

    // Rest site
    Rest,

    // Card reward
    ChooseCardReward(usize), // 0-indexed option
    SkipReward,

    // Debug
    WinCombat,             // instantly win the current combat
    SkipFloor,             // advance past current floor without fighting
    AddCard(Card),         // add a card to hand mid-combat
    AddRelic(Relic),       // grant a relic
}
```

**`Command` lives in `run.rs`** (not `combat.rs`) because the full set of variants is dispatched at the run level. `combat.rs` only handles the three combat variants and rejects the rest with `InvalidPhase`.

### Errors

```rust
// run.rs
pub enum CommandError {
    CombatOver,       // command issued after Victory or Defeat
    InvalidCard,      // out-of-bounds index, or unknown map node
    NotEnoughEnergy,  // PlayCard when player can't afford it
    InvalidPhase,     // command not valid in current GameState
}
```

---

## Scenario

```rust
// run.rs
pub enum Scenario { Main, Simple }
```

`Scenario` is stored on `MapState` and `GameState::Combat` so it flows through the whole run.

| Scenario | Starter deck | RNG | After combat win |
|---|---|---|---|
| `Main` | `starter_deck()` (5×Strike, 3×Defend, …) | `ThreadRng` | `CardReward` |
| `Simple` | empty | `NoOpRng` | back to `Map` (no reward) |

**`Simple` is for deterministic snapshot testing.** Scripts use `spawn <enemy...>` to control which enemies appear. Because the deck starts empty, scripts add cards explicitly via `add <card>` (debug command) before entering combat. Because RNG is `NoOpRng`, card draw order, shuffle order, and reward generation are fully deterministic.

Entry points:
- `new_run(rng)` → `Scenario::Main`, populated deck
- `new_simple_run()` → `Scenario::Simple`, empty deck

---

## GameState and Transitions

```rust
// run.rs
pub enum GameState {
    Map(MapState),
    Combat { state: CombatState, floor: usize, scenario: Scenario },
    RestSite(RestSiteState),
    CardReward(CardRewardState),
    GameOver { victory: bool },
}
```

### Transition table

| Current state | Command | Next state (Main) | Next state (Simple) |
|---|---|---|---|
| `Map` | `ChooseNode(0)` → Combat node | `Combat { floor }` | `Combat { floor }` |
| `Map` | `ChooseNode(0)` → Rest node | `RestSite { floor }` | `RestSite { floor }` |
| `Map` | `Spawn(enemies)` | stays `Map` (queues enemies) | stays `Map` (queues enemies) |
| `Combat` | any → `Victory` (non-boss) | `CardReward { floor+1 }` | `Map { same floor }` |
| `Combat` | any → `Victory` (boss) | `GameOver { victory: true }` | `GameOver { victory: true }` |
| `Combat` | any → `Defeat` | `GameOver { victory: false }` | `GameOver { victory: false }` |
| `RestSite` | `Rest` | `Map { floor+1 }` | `Map { floor+1 }` |
| `CardReward` | `ChooseCardReward(i)` | `Map { same floor }` | — (never reached) |
| `CardReward` | `SkipReward` | `Map { same floor }` | — (never reached) |
| `GameOver` | any | `Err(CombatOver)` | `Err(CombatOver)` |

### Enemy spawn queue

`MapState.next_enemies: Option<Vec<EnemyKind>>` holds the override set by `Command::Spawn`. `ChooseNode(0)` consumes it (or falls back to `enemies_for_floor(floor)` if `None`). After use, `next_enemies` is cleared back to `None`.

### Map layout

```rust
// run.rs
pub const MAP_NODES: &[MapNode] = &[
    MapNode::Combat,    // floor 0
    MapNode::Combat,    // floor 1
    MapNode::Combat,    // floor 2
    MapNode::RestSite,  // floor 3
    MapNode::Boss,      // floor 4
];
```

Boss victory → `GameOver { victory: true }` directly (no card reward).

**To add floors or branches:** extend `MAP_NODES`. The floor index drives `enemy_for_floor` and the boss check (`MAP_NODES.get(floor) == Some(MapNode::Boss)`).

---

## Player

```rust
// combat.rs
pub struct Player {
    pub hp: Hp,
    pub max_hp: Hp,
    pub block: Block,
    pub energy: Energy,
    pub max_energy: Energy,
    pub hand: Vec<Card>,
    pub draw_pile: Vec<Card>,
    pub discard_pile: Vec<Card>,
    pub exhaust_pile: Vec<Card>,   // exhausted cards; never return to draw pile
    pub statuses: StatusMap,
    pub deck: Vec<Card>,           // master deck; source of truth between combats
    pub gold: i32,
}
```

`deck` is the persistent master copy. At combat start, `from_player` clones `deck` into `draw_pile` (resetting hand/discard/exhaust/block/statuses). `player_after_combat` returns a post-combat `Player` with piles cleared and gold added — `deck` and `hp` are preserved.

**`exhaust_pile` is scoped to one combat.** `player_after_combat` extends `deck` with `exhaust_pile` then clears the pile — exhausted cards are back in the master deck for the next fight. A relic that permanently removes cards (e.g. a "Feed" that destroys minions) would need a separate permanent-exhaust mechanism.

---

## Enemy

```rust
// combat.rs
pub struct Enemy {
    pub kind: EnemyKind,
    pub hp: Hp, pub max_hp: Hp, pub block: Block,
    pub intent: Intent,
    pub statuses: StatusMap,
}
```

### Adding a new enemy

1. Create `enemies/<name>.rs` with a `DEF: EnemyDef` constant and `pub fn next_intent(turn: u32) -> Intent`.
2. Add a variant to `EnemyKind` in `enemies/mod.rs`.
3. Add arms to `EnemyKind::def()` and `next_intent()` dispatch in `enemies/mod.rs`.
4. Wire up in `run.rs`'s `enemy_for_floor(floor)`.

Nothing else needs to change. The combat engine treats all enemies identically — it calls `next_intent` and `execute_intent` without caring which enemy it is.

---

## Combat Engine (`combat.rs`)

### Internal entry point

```rust
pub(crate) fn apply_combat_command(
    mut state: CombatState,
    command: Command,
    rng: &mut impl Rng,
) -> Result<(CombatState, Vec<Event>), CommandError>
```

This is `pub(crate)` — only `run.rs` calls it. External code goes through `run::apply_command`.

### Turn structure

1. **PlayerTurn** — player plays cards and ends turn.
2. **EnemyTurn** — `EndEnemyTurn` fires: enemy block resets, intent executes, statuses tick, player block expires, energy resets, cards drawn, turn counter increments, new intent computed. Transitions to `PlayerTurn`.
3. **Victory / Defeat** — terminal; no further commands accepted.

The TUI auto-drains `EnemyTurn` by issuing `EndEnemyTurn` in a loop until the phase is no longer `EnemyTurn`.

### PlayCard handler (fixed logic, not per-card)

The handler in `apply_combat_command` owns: energy deduction, hand removal, `CardPlayed` event, calling `cards::apply`, exhaust/discard routing, `CardExhausted` event, victory check. Per-card modules own only their effect.

**Death check is centralised here.** Card modules do not check `enemy.hp <= 0`. This ensures the kill event always fires exactly once and in the right order, regardless of which card dealt the lethal blow.

### Discard vs exhaust routing

After `cards::apply`, the handler checks `card.exhausts()`:
- `false` → pushed to `discard_pile`
- `true` → pushed to `exhaust_pile`, `CardExhausted` event emitted

**`exhausts()` is the seam to extend.** Currently only `Disarm` returns `true`. Future random-exhaust or player-choice exhaust would require a different mechanism (probably a return value from `cards::apply` or a flag on `CombatState`).

---

## Cards (`cards/`)

### Card enum and CardDef

```rust
pub enum Card { Strike, Defend, Bash, Clothesline, Inflame, DeadlyPoison, Disarm }

pub enum CardDescription {
    Static(&'static str),
    WithDamage { template: &'static str, base: i32 },
}

pub struct CardDef {
    pub name: &'static str,
    pub description: CardDescription,
    pub energy_cost: Energy,
}
```

`CardDescription::WithDamage` bundles the template and base damage together — you cannot have one without the other. The `{damage}` placeholder in the template is replaced at display time with either the base value (`description()`) or the status-modified value (`effective_description()`).

Card methods on `Card`:
- `def() -> CardDef` — static data
- `name() -> &'static str`
- `energy_cost() -> Energy`
- `description() -> String` — base damage substituted
- `effective_description(attacker, defender) -> String` — live damage with `*N*` emphasis if modified
- `effective_damage(attacker, defender) -> Option<i32>` — `None` for non-damage cards
- `exhausts() -> bool` — currently only `Disarm`

### Per-card modules

Each card's effect lives in `cards/<name>.rs` as `pub fn apply(state: &mut CombatState, events: &mut Vec<Event>)`. The module may call:
- `deal_damage(amount, &mut hp, &mut block) -> i32` — returns damage dealt after block
- `apply_status(statuses, target, effect, stacks, events)` — applies stacks and emits `StatusApplied`
- `crate::status::resolve_damage(base, attacker, defender) -> i32` — damage formula

**Card damage values are hardcoded in each module** (e.g. `resolve_damage(6, ...)` in `strike.rs`). They are also present in `CardDef` for display. These must be kept in sync manually — there is no single source of truth for damage numbers yet.

### Adding a new card

1. Create `cards/<name>.rs` with `pub fn apply(state, events)`.
2. Add a variant to `Card` in `cards/mod.rs`.
3. Add a `CardDef` arm in `Card::def()`.
4. Add `Card::exhausts()` arm if the card exhausts itself.
5. Add dispatch arm in `cards::apply()`.
6. Add `mod <name>;` at the top of `cards/mod.rs`.
7. Add to `reward_pool()` if the card should appear as a reward.
8. Add to `starter_deck()` if it should be in the starting deck.

Tests for the card go in `cards/mod.rs`'s test module.

### Starter deck and reward pool

```rust
// cards/mod.rs
pub fn starter_deck() -> Vec<Card>  // 5×Strike, 3×Defend, Bash, Inflame, DeadlyPoison, Disarm
pub fn reward_pool() -> Vec<Card>   // Bash, Clothesline, Inflame, DeadlyPoison, Strike, Defend
```

`reward_pool` lives in `cards/mod.rs` because it is card knowledge — it should be updated alongside card additions, not in the run orchestrator.

---

## Status System (`status.rs`)

```rust
pub enum StatusEffect { Vulnerable, Weak, Poison, Strength }
pub type StatusMap = IndexMap<StatusEffect, i32>;  // insertion-ordered; value is stack count
```

### Damage formula

```rust
// status.rs
pub fn resolve_damage(base: i32, attacker: &StatusMap, defender: &StatusMap) -> i32 {
    let dmg = base + strength(attacker);          // Strength: flat bonus, permanent
    let dmg = if weak(attacker)   { dmg * 3 / 4 } else { dmg };   // Weak: -25%
    let dmg = if vuln(defender)   { dmg * 3 / 2 } else { dmg };   // Vulnerable: +50%
    dmg.max(0)
}
```

Order is fixed: Strength → Weak → Vulnerable. Integer arithmetic (no floats).

### Status lifecycle

| Status | Ticks | When |
|---|---|---|
| `Vulnerable` | −1 per turn | end of enemy turn (`tick_statuses` on enemy) |
| `Weak` | −1 per turn | end of enemy turn (`tick_statuses` on enemy) |
| `Poison` | −1 per trigger | drains before enemy acts (`drain_poison`); also deals HP damage |
| `Strength` | never | permanent for the combat |

`tick_statuses` removes the status when stacks reach 0. `drain_poison` returns the damage to deal and decrements; caller applies it to HP (bypassing block).

**Extension point for relics:** `tick_statuses` and `drain_poison` are the natural hooks for relics that modify status behaviour (e.g. "Poison also applies Weak"). Currently they are pure functions with no hook mechanism.

---

## RNG (`rng.rs`)

```rust
pub trait Rng { fn shuffle<T>(&mut self, slice: &mut [T]); }

pub struct ThreadRng(rand::rngs::ThreadRng);  // production
pub struct NoOpRng;                            // identity shuffle — tests

pub enum AnyRng {
    Thread(ThreadRng),
    NoOp(NoOpRng),
}
```

All randomness routes through `shuffle`. Currently used for: shuffling draw pile, shuffling card reward pool. Production code passes `&mut ThreadRng`. Tests pass `&mut NoOpRng` for determinism.

**`AnyRng` is the runtime-selectable wrapper.** The `Rng` trait is not object-safe (`shuffle<T>` is generic), so `Box<dyn Rng>` doesn't work. `AnyRng` uses enum dispatch instead — each arm delegates to the inner type. `main.rs` and `run_game` accept `&mut AnyRng`; `slay-core` tests use `&mut NoOpRng` directly.

**`NoOpRng` is an identity shuffle** — slice order is unchanged. Test setup must account for this: draw order = reverse of deck construction order (drawn via `pop()`).

**Extension point:** The `Rng` trait only exposes `shuffle`. When random number generation beyond shuffling is needed (e.g. random damage ranges, proc chances), add methods to the trait and a new `AnyRng` dispatch arm.

---

## Events (`combat.rs`)

`Event` is the communication channel from the engine to the TUI. The engine emits events; the TUI renders them as strings.

```rust
pub enum Event {
    CardPlayed { card: Card },
    CardExhausted { card: Card },
    PlayerAttacked { raw: i32, damage: i32 },  // raw = post-formula, damage = post-block
    PlayerBlocked { amount: i32 },
    EnemyAttacked { raw: i32, damage: i32 },
    EnemyDefended { amount: i32 },
    StatusApplied { target: Target, status: StatusEffect, stacks: i32 },
    IntentRevealed { intent: Intent },
    PlayerBlockExpired { amount: i32 },
    TurnEnded,
    TurnStarted { turn: u32 },
    EnemyPoisoned { damage: i32 },
    EnemyDied,
    PlayerDied,
    GoldEarned { amount: i32 },
    Healed { amount: i32 },
    CardAdded { card: Card },
}
```

**`raw` vs `damage`**: `raw` is after the status formula; `damage` is after block absorption. Both are included so the TUI can describe partial blocks ("5 blocked, 3 damage").

**Extension point for relics:** In the planned relic system (`RelicEffect::on_event`), the event stream is the natural hook. The engine would pass each event to active relics before continuing. Currently no such hook exists — events are emitted and immediately returned.

---

## TUI (`slay-tui`)

`slay-tui` ships **two renderers** that share all game logic and formatting:

| Mode | Entry point | Backend | When used |
|---|---|---|---|
| Plain text | `game::run_game` | `impl Write` (stdout, `Vec<u8>`, file) | `--plain`, `--script <path>`, or stdout is not a TTY |
| Ratatui | `tui::run_tui` | `crossterm` raw mode + alternate screen | Default when running interactively (TTY) |

`main.rs` decides at startup which one to call. Both share `engine.rs` for command application and event/intent/icon formatting; only the rendering layer differs.

### `engine.rs` — shared layer

```rust
pub fn apply_and_drain(
    state: GameState, command: Command, rng: &mut AnyRng,
) -> Result<(GameState, Vec<Event>), CommandError>
```

Applies one player command, then auto-drains all `EnemyTurn` ticks (issuing `EndEnemyTurn` until the phase is no longer `EnemyTurn`). Returns the final state and a flat `Vec<Event>` containing all events from the command and every drained tick. Both `run_game` and `run_tui` call this — there is no other path that drives the engine forward.

Also exposes the formatting helpers used by both renderers:
- `describe_event(&Event) -> String` — long-form text for the log panel / stdout
- `describe_intent(&Intent) -> String` — what the enemy is about to do
- `status_display(StatusEffect) -> (icon, name)` — emoji + label
- `statuses_inline(&StatusMap) -> String` — compact `[💪3 🪫2]` rendering
- `card_type_icon(CardType) -> &'static str`, `enemy_icon(&Enemy) -> &'static str`

Test fixtures (`integration.rs::TestHarness::send`) also use `apply_and_drain` so the harness stays in sync with the game loop.

### `game.rs` — plain text loop

```rust
pub fn run_game(
    state: GameState,
    reader: impl BufRead,
    writer: &mut impl Write,
    rng: &mut AnyRng,
    debug: bool,
)
```

The plain text loop as a pure function over I/O types. `main.rs` calls it with stdin/stdout and `ThreadRng`. Snapshot tests call it with byte slices and `Vec<u8>`.

Loop logic:
1. Render current state.
2. Read a line. Skip blank lines and `#`-prefixed comment lines.
3. Echo `> {line}`.
4. Check for pile inspection shortcuts (`z`/`x`/`c` in combat) → print pile, continue.
5. Parse input → `Command`; on `None` print "Unknown command."
6. Call `engine::apply_and_drain`; on `Err` print error message via `Display`.
7. Check for `GameOver`; print outcome and break.
8. Print events, re-render state.

### `tui.rs` — ratatui loop

```rust
pub fn run_tui(state: GameState, rng: &mut AnyRng, debug: bool) -> std::io::Result<()>
```

Takes over the terminal: enables raw mode, enters the alternate screen, installs a panic hook that restores the terminal even on panic. The main loop polls crossterm key events, accumulates typed characters into `TuiState.input_buf`, and on Enter dispatches through `command::parse` and `engine::apply_and_drain` — exactly the same path as `run_game`.

`TuiState`:
- `game: GameState` — the current state
- `input_buf: String` — accumulated keystrokes
- `event_log: VecDeque<String>` — last 200 event descriptions for the log panel
- `last_error: Option<String>` — shown in red status line until next successful command
- `show_pile: Option<PileView>` — when set, draws an overlay showing the chosen pile
- `should_quit: bool` — set on game over

**Layout** (every screen):
```
┌─ TOP BAR ─ Length(1) ─ HP/energy/block/gold/deck ────────────┐
│                                                              │
│   MAIN AREA (screen-specific) — Min(0)                        │
│                                                              │
├─ STATUS LINE — Length(1) — last error (red) or last event ───┤
├─ INPUT BOX — Length(3) — "> " + input_buf ───────────────────┤
└──────────────────────────────────────────────────────────────┘
```

Combat splits the main area horizontally 55/45: enemies + hand + pile counts on the left; scrollable log on the right. Other screens (map, rest, card reward, game over) use a single block.

**Input handling:**
```
KeyCode::Char(c)        → input_buf.push(c)
KeyCode::Backspace      → input_buf.pop()
KeyCode::Enter          → handle_enter(rng)
KeyCode::Esc / Ctrl+C   → break
```

`handle_enter`: intercept `z`/`x`/`c` to open a pile overlay; otherwise call `command::parse` then `engine::apply_and_drain`. On parse failure or command error, set `last_error` and clear the input buffer. On success, append events to `event_log`.

**`render_frame(f: &mut Frame, tui: &TuiState)`** is a pure function. Tests use ratatui's `TestBackend` to render frames into an in-memory buffer and assert on cell contents — no real terminal needed.

### `main.rs` routing

```
--script <path>            → run_game with file reader
--plain                    → run_game with stdin (always)
stdout/stdin not a TTY     → run_game with stdin (auto-fallback)
otherwise                  → run_tui
```

`IsTerminal` is in `std::io` since Rust 1.70 — no external crate.

### `command.rs`

```rust
pub fn parse(input: &str, state: &GameState, debug: bool) -> Option<Command>
```

Context-aware: the same input ("1") means different things in different states. Returns `None` on unknown input; the main loop prints "Unknown command." and re-prompts. Does not mutate state.

| State | Input | Command | Debug only |
|---|---|---|---|
| Map | `""`, `"enter"` | `ChooseNode(0)` | |
| Map | `"spawn <ids...>"` | `Spawn(Vec<EnemyKind>)` | |
| Map | `"skip"` | `SkipFloor` | ✓ |
| Map | `"relic <id>"` | `AddRelic(relic)` | ✓ |
| Combat | `"1"`–`"N"`, `"play N"` | `PlayCard(N-1)` | |
| Combat | `"end"`, `"e"`, `"end turn"`, `"pass"` | `EndTurn` | |
| Combat | `"win"` | `WinCombat` | ✓ |
| Combat | `"add <id>"` | `AddCard(card)` | ✓ |
| Combat | `"relic <id>"` | `AddRelic(relic)` | ✓ |
| RestSite | `"rest"`, `"r"` | `Rest` | |
| RestSite | `"upgrade N"`, `"u N"` | `UpgradeCard(N-1)` | |
| CardReward | `"1"`–`"N"`, `"pick N"` | `ChooseCardReward(N-1)` | |
| CardReward | `"skip"`, `"s"` | `SkipReward` | |

Pile inspection (`"z"`, `"x"`, `"c"`) is handled in `game::run_game` before `parse` is called — it is a display operation, not a `Command`.

**`spawn` is always available on the Map** (not debug-gated) so that `.slay` scripts work without `--debug`. Other commands that bypass game mechanics require `--debug`.

### Integration tests (`tests/integration.rs`)

`TestHarness` wraps `GameState` and exposes `send(input: &str)` which parses and applies a command, then auto-drains `EnemyTurn` — mirroring the main loop. Used for end-to-end verification of the full stack.

### Snapshot tests (`tests/scripts.rs`)

Discovers every `scripts/simple/*.slay` file at test time and runs each through `run_game` with `NoOpRng` and a `Vec<u8>` writer. The full TUI output is compared against a committed snapshot via `insta`.

**Running snapshot tests:**
```
cargo test -p slay-tui --test scripts
```

**After changing TUI output** (e.g. new render field, new event description), snapshots will fail. Review and accept:
```
cargo insta review            # interactive review tool
# or, to accept all at once:
INSTA_UPDATE=always cargo test -p slay-tui --test scripts
```

**Adding a new scenario:** write a `scripts/simple/<name>.slay` file and run `INSTA_UPDATE=new cargo test -p slay-tui --test scripts` to generate its snapshot. Commit both the script and the `.snap` file.

**`.slay` script format:**
```
# Lines starting with # are comments (echoed to output)
# Blank lines are silently skipped

spawn louse cultist     # override next combat's enemies (Map phase only)
enter                   # enter the current node (same as pressing Enter)
add strike              # add a card to hand (debug; always available in scripts)
1                       # play card 1
end                     # end turn
win                     # instantly win combat (debug)
```

---

## Testing Strategy

| Layer | Location | What it tests |
|---|---|---|
| Damage formula | `status.rs` tests | `resolve_damage` math in isolation |
| Per-card effects | `cards/mod.rs` tests | What each card does when played |
| Combat engine | `combat.rs` tests | Drawing, energy, phases, block, turn cycle, status tick-down |
| Run/progression | `run.rs` tests | Map transitions, rest, rewards, gold, floor progression, Scenario |
| Enemy IDs | `enemies/mod.rs` tests | `id()`/`from_id()` round-trip for all variants |
| TUI integration | `slay-tui/tests/integration.rs` | Text commands → `GameState` assertions through the full stack |
| TUI snapshot | `slay-tui/tests/scripts.rs` | Full TUI output from `.slay` scripts, compared via `insta` |

All `slay-core` tests use `NoOpRng` directly. Snapshot tests use `AnyRng::NoOp`. No mocks — tests exercise real code paths.

**Run snapshot tests after any TUI output change.** They will catch regressions in render layout, event descriptions, or state transition messages.

---

## Key Architecture Decisions

### Fixed constraints

- **`GameState` is owned and cloned, never mutated in-place.** `apply_command` takes ownership and returns a new state. This makes every state transition a pure function, enabling deterministic replay and trivial testing.
- **Card modules cannot see whether they killed the enemy.** Death detection (`enemy.hp <= 0`) is centralised in the `PlayCard` handler in `combat.rs`. Card modules only apply their effect. This prevents double-emitting `EnemyDied`.
- **`exhaust_pile` is scoped to one combat.** `player_after_combat` returns exhausted cards to `deck` and clears the pile. For a relic that permanently exhausts cards, a separate permanent-exhaust mechanism would be needed.

### Flexible / designed to change

- **`enemy_for_floor(floor)`** maps floor index to `EnemyKind`: floor 1 → `Fungibeast`, all others → `Louse`. Extend this `match` as new floors or enemies are added.
- **`Card::exhausts()` covers only self-exhaust.** Random exhaust ("exhaust a random card in hand") or player-choice exhaust require a different mechanism — likely a return value from `cards::apply` or a flag set on `CombatState` during card execution.
- **`Rng` trait only exposes `shuffle`.** Extend with `next_u32` or similar when proc chances or random damage ranges are needed.
- **`MAP_NODES` is a fixed slice.** Phase 8 (branching map) will require replacing this with a graph structure on `MapState`.
- **`Intent` is `Attack(i32) | Defend(i32)`.** This covers all current enemies. Future intents (buff, debuff, multi-hit) require new variants. This is the most likely enum to grow.
- **The event stream is the relic hook point.** Phase 10's `RelicEffect::on_event(&Event, &mut CombatState)` slots in after each event is emitted. No plumbing change is needed in the engine — the caller (`run.rs`) would iterate active relics after each `apply_combat_command` call.
- **`resolve_damage` has no relic hooks.** If a relic needs to modify the formula (e.g. "Paper Krane: take 40% less damage"), the function signature or a context parameter would need to change.

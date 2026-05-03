# Plan: Ratatui TUI

## Context

The current plain-text renderer reprints the full game state after every command, scrolling past old turns. Ratatui replaces this with a persistent layout (enemy panel, player stats, scrollable event log, hand) that updates in place. The plain-text path (`run_game`) must stay fully intact — snapshot tests depend on it and it's the fallback for `--plain` and `--script` mode.

## Architecture

Two separate event loops, one shared core:

```
slay-core          ← 100% shared (unchanged)
command.rs         ← 100% shared (unchanged)
engine.rs          ← NEW: apply_and_drain + all formatting helpers
game.rs            ← plain text loop (keep run_game signature, delegate to engine)
tui.rs             ← NEW: ratatui loop (run_tui)
main.rs            ← picks between them
```

`engine.rs` is the shared layer extracted from `game.rs`:
- `pub fn apply_and_drain(state, command, rng) -> Result<(GameState, Vec<Event>), CommandError>` — applies one command then auto-drains EnemyTurn ticks until phase is no longer EnemyTurn. Replaces the inline drain loop that currently appears in `game.rs` and `integration.rs::TestHarness`.
- Formatting helpers promoted from private `game.rs` functions: `describe_event`, `describe_intent`, `status_display`, `statuses_inline`, `card_type_icon`, `enemy_icon`.

`game.rs` and `tui.rs` both call into `engine`. The public signature of `run_game` does not change.

## Dependencies to add (`crates/slay-tui/Cargo.toml`)

```toml
ratatui  = "0.29"
crossterm = "0.28"
```

Both are compatible with rustc 1.82 and edition 2021 (verified).

## Layout (`tui.rs`)

Four vertical chunks on every screen:
```
┌─────────────────────────────────────┐
│ TOP BAR: 🧙 HP 72/80  ⚡ 3/3  🪙 99 │  Length(1)
├─────────────────────────────────────┤
│ MAIN AREA (screen-specific)         │  Min(0)
├─────────────────────────────────────┤
│ STATUS LINE: last error / last event│  Length(1)
├─────────────────────────────────────┤
│ INPUT BOX: "> _"                    │  Length(3)
└─────────────────────────────────────┘
```

**Combat** — main area splits horizontally 55/45:
- Left: enemies list (name, HP, block, intent, statuses) + hand list (card name, cost, description; unaffordable = `Color::DarkGray`) + pile counts
- Right: scrollable event log (`VecDeque<String>` capped at 200 in `TuiState`)

**Map** — main area: `List` of floor nodes, current floor bold/marked `▶`

**Rest Site** — main area: heal amount + `List` of upgradeable cards

**Card Reward** — main area: `List` of three card options + skip

**Game Over** — centred `Paragraph`, green for victory, red for defeat, "Press any key to quit"

## Input handling (`tui.rs`)

Accumulate keystrokes into `TuiState.input_buf: String`. On Enter, call `command::parse(&input_buf, &state, debug)` — identical to the plain path, no changes to `command.rs`. On `None` parse result, set `TuiState.last_error` and show in status line. `z`/`x`/`c` pile shortcuts open an overlay popup.

```
KeyCode::Char(c)   → input_buf.push(c)
KeyCode::Backspace → input_buf.pop()
KeyCode::Enter     → handle_enter(rng)
KeyCode::Esc / Ctrl+C → break
```

## `run_tui` outline

```rust
pub fn run_tui(state: GameState, rng: &mut AnyRng, debug: bool) {
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    // main loop: terminal.draw → poll event → handle key
    // teardown: disable_raw_mode, LeaveAlternateScreen, show_cursor
}
```

Register a panic hook before entering raw mode so the terminal is always restored.

## `main.rs` routing

```rust
let plain  = args.iter().any(|a| a == "--plain");
let use_tui = !plain && script.is_none() && io::stdout().is_terminal();
// IsTerminal is in std::io since Rust 1.70 — no external crate needed
```

- `--script <path>` → always `run_game` with file reader
- `--plain` → always `run_game` with stdin
- stdout is not a TTY (piped) → `run_game` automatically
- otherwise → `run_tui`

## Files to change

| File | Change |
|---|---|
| `crates/slay-tui/Cargo.toml` | add ratatui, crossterm |
| `crates/slay-tui/src/engine.rs` | NEW: apply_and_drain + formatting helpers |
| `crates/slay-tui/src/tui.rs` | NEW: run_tui, TuiState, render functions |
| `crates/slay-tui/src/lib.rs` | add `pub mod engine; pub mod tui;` |
| `crates/slay-tui/src/game.rs` | delegate drain loop + formatters to engine |
| `crates/slay-tui/src/main.rs` | add --plain, TTY detection, route to run_tui |
| `crates/slay-tui/tests/integration.rs` | simplify TestHarness::send via apply_and_drain |

`run_game` signature and snapshot tests: **unchanged**.

## Steps (TDD)

### Step 1 — Extract `engine.rs`
RED: test `apply_and_drain` drains EnemyTurn and flattens events.
GREEN: extract from `game.rs`. Simplify `game.rs` drain loop and `integration.rs` harness.
All existing tests pass.

### Step 2 — Add ratatui dep + `tui.rs` skeleton
RED: test that `render_frame` in combat state renders enemy name (via `TestBackend`).
GREEN: minimal layout + widgets compiles and renders.

### Step 3 — Implement all screen renders
RED: tests for map, rest, card reward, game over screens via `TestBackend`.
GREEN: fill in each render function.

### Step 4 — Input handling + `handle_enter`
RED: tests for valid command updates state, unknown command sets last_error, error sets last_error.
GREEN: implement `handle_enter` calling `engine::apply_and_drain`.

### Step 5 — Wire `main.rs` + terminal setup/teardown
GREEN: `--plain` flag and TTY routing in main.
Manual test: `cargo run` launches TUI; `cargo run -- --plain` launches plain.

## Verification

```bash
cargo test                                         # all existing tests pass
cargo test -p slay-tui --test scripts             # snapshots unchanged
cargo run                                          # ratatui UI launches
cargo run -- --plain                               # plain text mode
cargo run -- --script crates/slay-tui/tests/scripts/01-louse-end-turn.slay  # script mode
```

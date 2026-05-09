# Plan: WASM Browser Build

**Branch**: main
**Status**: Active

## Goal

Ship the game as a static website where it runs entirely in the user's browser via WebAssembly, using xterm.js to render output in a terminal emulator.

## Two-phase approach

**Phase 1 (this plan)** uses the existing plain-text renderer (`game.rs`) — quick to ship, no wasted work.

**Phase 2 (optional extension)** upgrades to the full ratatui UI — the coloured HP bar, side-by-side combat panel, styled boxes. Phase 1 builds exactly the infrastructure Phase 2 reuses; nothing gets thrown away except ~50 lines of rendering glue inside `WasmSession::send`. The only additions are a `WasmBackend` (ratatui `Backend` impl that accumulates ANSI sequences) and a refactor of `run_tui`'s event loop from "block on keypress" to "handle one event, return" — the blocking call is illegal in WASM and is the real work of Phase 2.

## Background

`slay-core` is pure Rust with no I/O — it will compile to WASM unchanged. The existing `game.rs` plain-text renderer already produces terminal output to a `Vec<u8>` writer, which is easy to surface to JS. The approach is:

1. New `slay-wasm` crate that wraps a `WasmSession` (stateful game loop) and exposes it to JS via `wasm-bindgen`.
2. A minimal HTML/JS harness with xterm.js that forwards keystrokes to WASM and writes returned text to the terminal.
3. Build via `wasm-pack`, deploy as static files (GitHub Pages or equivalent).

`GameState`, `Command`, and all core types already derive `serde::Serialize` / `Deserialize`, so JSON round-trips are free if needed.

## Acceptance Criteria

- [ ] `cargo build -p slay-wasm --target wasm32-unknown-unknown` succeeds.
- [ ] Running `wasm-pack build crates/slay-wasm` produces a pkg/ directory with the WASM binary and JS bindings.
- [ ] Opening `index.html` in a browser starts a new game and renders the initial state in the xterm.js terminal.
- [ ] Typing a command (e.g. `1`) and pressing Enter sends it to the WASM session and renders the response.
- [ ] The game plays through at least one complete combat: draw, play card, end turn, win.
- [ ] The page works fully offline (no CDN — xterm.js bundled or vendored).
- [ ] `slay-core` and `slay-tui` are unchanged.

## Architecture

```
slay/
  crates/
    slay-wasm/
      Cargo.toml          ← wasm-bindgen, slay-core dep; target = wasm32-unknown-unknown
      src/
        lib.rs            ← WasmSession exposed via #[wasm_bindgen]
  www/
    index.html            ← loads xterm.js + WASM module; game UI
    main.js               ← wires xterm.js input/output to WasmSession
    xterm/                ← vendored or bundled xterm.js + addons
```

### `WasmSession` API

```rust
#[wasm_bindgen]
pub struct WasmSession { /* opaque */ }

#[wasm_bindgen]
impl WasmSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmSession;               // new_run, ThreadRng equivalent

    pub fn send(&mut self, input: &str) -> String;  // → rendered plain-text output
    pub fn is_over(&self) -> bool;
}
```

`send` reuses the plain-text rendering logic from `game.rs` — it accepts one line of input, runs the command through `engine::apply_and_drain`, renders the result to a `Vec<u8>`, and returns it as a `String`. The JS side writes that string directly into the xterm.js terminal.

### RNG in WASM

`ThreadRng` (from `rand`) is not available in WASM. Options:
- Use `getrandom` with the `js` feature, which is supported by `rand` on WASM targets.
- Or implement a simple PRNG (e.g. xorshift64) seeded from `Math.random()` via `wasm-bindgen`.

Recommended: add `getrandom = { version = "0.2", features = ["js"] }` to `slay-wasm`'s `Cargo.toml` and use `rand::thread_rng()` — it works transparently once `getrandom/js` is in the dep tree.

### Input model

xterm.js fires per-keystroke events. `main.js` buffers keystrokes into a line, sends the line to `WasmSession::send()` on Enter, then writes the returned string to `term.write()`. The xterm.js `LocalEchoAddon` handles backspace and line editing in the browser.

## Steps

Every step follows RED → GREEN → MUTATE → KILL MUTANTS → REFACTOR.

---

### Step 1: Create `slay-wasm` crate and verify it compiles to WASM

**RED**: Add an integration test (native target) in `slay-wasm/tests/` that:
- `new_session_returns_non_empty_output` — `WasmSession::new()` followed by `session.send("")` returns a non-empty string (the initial render).
- `send_win_combat_debug_command_returns_output` — `session.send("win")` (debug) after reaching a combat state returns non-empty output.

**GREEN**:
- Create `crates/slay-wasm/Cargo.toml` with `wasm-bindgen`, `slay-core`, `getrandom/js` deps.
- Create `crates/slay-wasm/src/lib.rs` with `WasmSession`:
  - `new()` calls `slay_core::new_run(rng, ctx)`, stores `GameState`, `AnyRng`, `debug: false`.
  - `send(input)` — port the inner loop of `game::run_game` to write into a `Vec<u8>`, return as `String`.
  - `is_over()` — returns `matches!(self.state, GameState::GameOver { .. })`.
- Add `slay-wasm` to the workspace `Cargo.toml`.
- Verify `cargo test -p slay-wasm` passes (native).
- Verify `cargo build -p slay-wasm --target wasm32-unknown-unknown` passes (WASM).

**MUTATE**: Run mutation testing on `slay-wasm/src/lib.rs`.

**KILL MUTANTS**: Address survivors.

**REFACTOR**: Assess. The `send` implementation will likely duplicate some of `game.rs` — consider extracting a shared `step(state, input, rng, debug) -> (GameState, String)` into `slay-tui/src/game.rs` that both can call.

**Done when**: Native tests pass; WASM binary builds without errors.

---

### Step 2: `wasm-pack` build and minimal `index.html`

No production code changes — this step is build infrastructure and the static web harness.

**Tasks**:
- Install `wasm-pack` (document in README, not a project dep).
- Add a `Makefile` target (or `justfile`) `make wasm` that runs `wasm-pack build crates/slay-wasm --target web --out-dir ../../www/pkg`.
- Create `www/index.html` with an xterm.js terminal div and `<script type="module" src="main.js">`.
- Create `www/main.js` that:
  1. Imports the WASM module from `./pkg/slay_wasm.js`.
  2. Creates a `Terminal` (xterm.js) and opens it on `#terminal`.
  3. Instantiates `WasmSession`.
  4. Writes the initial output (call `send("")` with empty string, or expose a `render_initial()` method).
  5. Buffers keystrokes; on Enter, calls `session.send(line)` and writes result to terminal.
- Vendor xterm.js (download from npm release or bundle via esbuild — keep it a single `<script>` tag, no build step for JS).

**Done when**: `python3 -m http.server` in `www/`, open browser, new game appears in terminal.

---

### Step 3: Full play-through smoke test

**RED**: Write a `slay-wasm` integration test `full_combat_round_trip`:
- New session → advance to first combat (send "1" to choose node) → send "win" (debug) → assert `is_over()` returns false and a card reward is shown → send "skip" → assert state is back on Map.

**GREEN**: Fix any issues found during the smoke test (likely: initial render on `new()`, NeowState handling before first map, debug mode flag).

**MUTATE**: Run mutation testing.

**KILL MUTANTS**: Address survivors.

**REFACTOR**: Assess.

**Done when**: Integration test passes; manual play-through in browser works end-to-end.

---

## Pre-PR Quality Gate

Before each PR:
1. Mutation testing — run `mutation-testing` skill; report killed/survived/score.
2. Refactoring assessment — run `refactoring` skill.
3. `cargo clippy --all-targets -- -D warnings` passes.
4. `cargo test -p slay-core --lib` passes.
5. `cargo test -p slay-tui` passes (including snapshots).
6. `cargo test -p slay-wasm` passes (native).
7. `cargo build -p slay-wasm --target wasm32-unknown-unknown` passes.

---

## Phase 2: ratatui UI in the browser (optional extension)

Once Phase 1 ships, this upgrade replaces the plain-text renderer with the full ratatui UI. Everything in `slay-wasm` and `www/` is reused; xterm.js already understands the ANSI escape sequences that ratatui produces.

### What changes

| Component | Phase 1 | Phase 2 |
|-----------|---------|---------|
| `WasmSession::send` rendering | calls `game.rs` plain-text loop | calls ratatui frame render, returns ANSI sequences |
| `tui.rs` event loop | untouched | refactored from blocking to single-step |
| New code | nothing | `WasmBackend` (~100 lines) + `run_tui` refactor |
| `slay-core` | unchanged | unchanged |
| `www/` JS harness | unchanged | unchanged |

### Step 4: Refactor `run_tui` to an event-driven model

The current `run_tui` loop calls `crossterm::event::read()` which blocks the thread — illegal in WASM. Refactor it to expose a `handle_event(event: CrosstermEvent) -> Option<RenderOutput>` function that processes one event and returns rendered output (or `None` if no re-render is needed). The native binary calls this in its existing loop; WASM calls it from a JS keyboard event handler.

**RED**: Write a test that constructs a `TuiState`, calls `handle_event` with a synthetic key event, and asserts the returned render output is non-empty.

**GREEN**: Extract the inner body of the `run_tui` loop into `handle_event`. The outer loop in `main.rs` becomes a thin wrapper that calls `crossterm::event::read()` then `handle_event`.

**MUTATE / KILL / REFACTOR**: Standard cycle.

**Done when**: Native binary behaviour is unchanged; `handle_event` is callable without a real terminal.

---

### Step 5: `WasmBackend` and ratatui render in `WasmSession`

**RED**: Write a test that creates a `WasmBackend`, draws a ratatui `Paragraph` widget to it, and asserts the output string contains ANSI escape sequences.

**GREEN**: Implement `WasmBackend` — a struct that implements ratatui's `Backend` trait and collects `draw()` calls into a `String` of ANSI sequences. Update `WasmSession::send` to:
1. Call `handle_event` (from Step 4) with the input.
2. Render the resulting `TuiState` to a `WasmBackend`.
3. Return the ANSI string.

**MUTATE / KILL / REFACTOR**: Standard cycle.

**Done when**: Opening the browser shows the full ratatui layout — HP bar, combat panels, coloured text — instead of plain text.

---

## Open Questions / Decisions Deferred

- **Deployment target**: GitHub Pages is the simplest (push `www/` + `pkg/` as a subtree or from CI). Not required for the plan to be complete.
- **JS bundling**: Using xterm.js from a local copy avoids a build step entirely. Esbuild is fast if a bundle step is ever wanted.
- **Save/load in browser**: `GameState` is already JSON-serializable — `localStorage` persistence would be a thin layer over `serde_json::to_string(&state)`. Out of scope for this plan.
- **Mobile / touch input**: xterm.js works on mobile but the on-screen keyboard experience is poor. A tap-based UI would require a separate frontend. Out of scope.

---
*Delete this file when the plan is complete.*

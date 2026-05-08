# Plan: Command Hints (B + C)

**Status**: Phase B complete

## Goal

Make it obvious to a new player what commands are available at any point in the game. Two-phase approach: first make the command box self-describing (Phase B), then add a full-screen help overlay accessible via `?` (Phase C).

## Problem

The `> Command` input box gives players no indication of what to type. The game phase changes (combat, map, rest site, shop, etc.) each have a different valid command set, but nothing surfaces this. New players stall at the prompt.

## Design

### Phase B — Dynamic command box title

Replace the static `" Command "` border title with a context-sensitive hint string derived from the current `GameState`. The hint shows the most important 1–3 shortcuts for the current phase, always including the most critical action (e.g. "end turn" in combat).

Examples:
- Combat (player turn): `" Command: play N [target] · end · use N "`
- Map: `" Command: N to choose node "`
- Rest site: `" Command: rest · upgrade N "`
- Card reward: `" Command: N to take card · skip "`
- Shop: `" Command: N card · r relic · p potion · leave "`
- Event room: `" Command: N to choose option "`

Once Phase C (help overlay) is implemented, a `· ? help` tail should be appended to every phase hint to seed discoverability.

Implementation: add `fn command_hint(state: &GameState) -> String` in `tui.rs`, called from `render_input`. No layout changes needed.

### Phase C — `?` help overlay

Pressing `?` opens a centred overlay listing all commands for the current phase, using the same overlay pattern already used for draw/discard/exhaust pile views.

The overlay:
- Has a titled border: `" Help — [?] or [esc] to close "`
- Lists commands in two columns: left = syntax, right = description
- Is state-aware: shows only commands valid in the current phase
- Dismisses on `?`, `esc`, or `enter`

New state on `TuiState`:
```rust
pub show_help: bool,
```

Key handling: `KeyCode::Char('?')` toggles `show_help`. When `show_help` is true, `esc`/`enter`/`?` all close it; other keys are ignored (no accidental command input while help is open).

Implementation: add `render_help_overlay(f, area, state)` in `tui.rs`, mirroring `render_pile_overlay`.

---

## Acceptance Criteria

### Phase B

- [x] In combat (player turn), the command box title contains `"end"` and `"play"`
- [x] In combat (enemy turn), the command box title indicates no player input is expected (or is blank/auto)
- [x] On the map, the command box title contains the node-selection hint
- [x] At a rest site, the title contains `"rest"` and `"upgrade"`
- [x] At a card reward, the title contains the pick/skip hint
- [x] In the shop, the title contains `"leave"` and a buy hint
- [x] In an event room, the title contains the choice hint
- [x] On game over, the title is blank or minimal
- [x] No layout changes; no new rows added to the frame

### Phase C

- [ ] Pressing `?` in any game phase opens the help overlay
- [ ] The overlay lists all valid commands for the current phase (syntax + description)
- [ ] Pressing `?`, `esc`, or `enter` closes the overlay
- [ ] While the overlay is open, other keypresses do not trigger game commands
- [ ] The overlay uses the same centred modal pattern as the pile overlay
- [ ] `show_help` starts as `false`; toggling works correctly from any phase

---

## Steps

Every step follows RED-GREEN-MUTATE-KILL MUTANTS-REFACTOR.

### Phase B

1. Add `fn command_hint(state: &GameState) -> String` — unit-test each phase variant
2. Wire `command_hint` into `render_input` — update snapshot tests that assert the command box title
3. Add TUI test asserting combat hint contains `"end"` and `"play"`

### Phase C

4. Add `show_help: bool` to `TuiState` — test default is `false`
5. Add `?` key handling to toggle `show_help` and suppress other input when open — unit test toggle logic
6. Add `fn help_lines(state: &GameState) -> Vec<(&str, &str)>` returning (syntax, description) pairs — unit test each phase
7. Add `render_help_overlay` and wire into `render_frame` — TUI test that overlay text appears when `show_help` is true
8. Test that pressing `esc`/`enter`/`?` closes the overlay

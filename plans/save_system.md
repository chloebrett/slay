# Save System Plan

## Goals

- **Within-run save**: suspend mid-run (including mid-combat) and resume later.
- **Between-run save** (meta): persist unlocked cards, character progression, run statistics across restarts.
- One save slot each; plaintext TOML; user-editable is fine.

---

## File Layout

```
~/.local/share/slay/          (XDG_DATA_HOME, see Save Location below)
  run.ron                     ← active run state (deleted on run end)
  meta.ron                    ← persistent meta-progression
```

`run.toml` is deleted (or zeroed) when a run ends (victory or defeat) so there's no stale save to resume. `meta.toml` is updated at run end.

---

## Two Save Files

### `run.toml` — within-run state

Serializes `GameState` in full. When the player quits mid-run the file is written; on next launch it is detected and the player is offered "Continue" or "New Run" (new run deletes the file).

**What must be captured:**

| Field | Notes |
|---|---|
| `GameState` variant (Map/Combat/RestSite/CardReward/Shop/Treasure/Event) | discriminant + payload |
| `Player` (hp, max_hp, gold, deck, relics, potions) | persistent player across all phases |
| `CombatState` | only when variant is `Combat`; includes hand, draw/discard/exhaust piles, statuses, counters, enemies |
| `MapGraph` (rows + edges) | the full DAG is needed to resume navigation |
| `floor`, `available_cols`, `next_floor_cols` | routing state |
| `Scenario` | `Main` always for real runs; included for completeness |

**RNG — see decision below.**

### `meta.toml` — between-run state

```toml
[meta]
runs_completed = 5
runs_won = 2

[unlocks]
cards = ["iron_wave", "cleave", "thunderclap"]   # ids of unlocked cards
relics = ["nunchaku", "whetstone"]
```

Currently the game has no unlock gates — `reward_pool()` returns everything. The meta save is forward infrastructure. To wire it up: `reward_pool()` accepts a `&MetaSave` and filters to unlocked cards; defeating a boss writes newly unlocked card ids to `meta.toml`.

---

## Serialisation Approach

### Use `serde` derives on core types

Add to `slay-core/Cargo.toml`:
```toml
serde = { version = "1", features = ["derive"] }
```

Derive `Serialize + Deserialize` on all game types:
`Card`, `Grade`, `EnemyKind`, `Move`, `Relic`, `Potion`, `StatusEffect`, `StatusMap`, `Player`, `CombatState`, `Enemy`, `GameState`, `MapGraph`, `MapNode`, `Scenario`, `CardRewardState`, `RestSiteState`, `ShopState`, `MapState`, …

This couples the on-disk format to the internal type names. That's fine — it's a single-player plaintext file and we're not building an API.

### Save/load lives in `slay-tui`

`slay-core` has no I/O (`println!` is banned). Serialization derives are pure data; that's acceptable in core. The file I/O (`fs::read_to_string`, `fs::write`, save-file path resolution) lives in `slay-tui::save` — a small new module.

```rust
// slay-tui/src/save.rs
pub fn load_run() -> Option<GameState> { … }
pub fn save_run(state: &GameState) { … }
pub fn delete_run() { … }
pub fn load_meta() -> MetaSave { … }
pub fn save_meta(meta: &MetaSave) { … }
```

`main.rs` calls `load_run()` at startup; `run_tui`/`run_game` call `save_run` at the right points.

### `ron` crate

```toml
# slay-tui/Cargo.toml
ron = "0.8"
```

`ron::ser::to_string_pretty` / `ron::de::from_str`. RON handles Rust enums natively (no tagged-table workarounds), which makes `GameState` and `Card` variants serialize cleanly. Output is human-readable and editable.

---

## Design Decisions / Tradeoffs

### 1. RNG state

**Problem:** `ThreadRng` is not serializable. Reloading without RNG state means future shuffles differ from what they would have been (different card draw order, reward options, etc.).

**Options:**

| Option | Behaviour on reload | Complexity |
|---|---|---|
| **Drop RNG state** (simplest) | Future shuffles are fresh-random | None |
| **Save seed; re-seed on load** | Deterministic future from saved seed | Switch to `StdRng`; save `u64` seed |
| **Save seed + replay log** | Byte-perfect replay from any checkpoint | Very high; not worth it |

**Decision: save the seed.** Replace `ThreadRng` with `rand::rngs::StdRng` (or add it as the second `AnyRng` variant alongside `NoOpRng`). At run creation, draw a `u64` seed from `thread_rng()` and store it in `GameState`. On save, write the seed. On load, re-seed `StdRng` from it. Future shuffle order is identical to what it would have been without a save. This changes `AnyRng` to three variants: `Seeded(StdRng)`, `Thread(ThreadRng)` (can drop), `NoOp(NoOpRng)`.

### 2. Mid-combat saves

Saving `CombatState` is straightforward if all types derive `Serialize`. No special handling needed — `hand`, `draw_pile`, enemy `statuses`, counters, etc. are all plain data.

**Tradeoff:** if we wanted to simplify, we could restrict saving to "safe points" (Map screen only) and disallow quitting mid-combat. Rejected — it's annoying UX and the engineering cost of full serialization is low once serde derives are in place.

### 3. Save on every command vs at transitions

**On every command:** maximally crash-safe (no progress lost). One `fs::write` per command — negligible cost for a turn-based game with tiny state.

**At transitions only** (floor change, end of combat): simpler to reason about; slightly less robust against crashes.

**Recommendation: save at every state transition** (i.e., every time `apply_and_drain` returns a new `GameState`). This is effectively every meaningful player action, is easy to implement in `apply_and_drain`'s caller, and is crash-safe enough for a game.

### 4. Save file location

| Option | Pro | Con |
|---|---|---|
| `~/.local/share/slay/` (XDG) | Standard on Linux/Mac; survives `cargo clean` | Need `dirs` crate or manual env lookup |
| `./save.toml` (cwd) | Trivial to implement | Breaks if run from different directories |
| `~/.config/slay/` | Also conventional | Slightly wrong semantics (config vs data) |

**Recommendation: XDG data dir.** Use the `dirs` crate (`dirs::data_dir()`) which returns `~/.local/share` on Linux/Mac and `%APPDATA%` on Windows. Fallback to cwd if `dirs::data_dir()` returns `None`.

```toml
dirs = "5"  # in slay-tui
```

### 5. Schema versioning

TOML saves will break if the `GameState` type changes (new field, renamed variant). Options:

- **Add a `version` field** to the save: `schema_version = 1`. On load, check it and either migrate or warn "incompatible save, starting new run."
- **No versioning** (simplest): just delete/ignore malformed saves. Fine for a personal project.

**Decision:** `schema_version: 1` in both save files. Parse it first; if mismatched, delete and ignore. No migration logic needed now — add it when the schema actually breaks.

### 6. `MetaSave` location

`MetaSave` is a new struct. It could live in `slay-core` (if unlocking gates `reward_pool`) or in `slay-tui` (if it's just persistence concern).

**Recommendation:** put `MetaSave` in `slay-core` alongside `reward_pool`, since unlock logic is game logic. Its serde derives are in-core. File I/O stays in `slay-tui::save`.

---

## RON Shape (sketch)

### `run.ron`

```ron
(
    schema_version: 1,
    rng_seed: 12345678901234567890,
    state: Combat(
        state: (
            player: (
                hp: 54,
                max_hp: 80,
                gold: 99,
                deck: [Strike(Base), Defend(Base), Bash(Base), IronWave(Base)],
                relics: [BurningBlood, Nunchaku],
                potions: [FirePotion],
                hand: [Strike(Base), Defend(Base)],
                draw_pile: [Bash(Base), IronWave(Base)],
                discard_pile: [Strike(Base)],
                exhaust_pile: [],
                statuses: {},
                // ...
            ),
            enemies: [
                (kind: JawWorm, hp: 40, max_hp: 44, /* ... */),
            ],
            // counters, phase, turn number ...
        ),
        floor: 3,
        is_boss: false,
        // graph, next_floor_cols, scenario ...
    ),
)
```

RON enum variants serialize exactly as Rust variant syntax — `Strike(Base)`, `Combat(...)` — with no string encoding or tagged-table indirection. The file is structurally self-documenting.

---

## Implementation Order

1. **Add serde derives to `slay-core` types** (behind a `serde` feature flag if you want to keep the dep optional for pure-logic users).
2. **Introduce seeded RNG** (`StdRng`) in `AnyRng`; store seed in `GameState`.
3. **Write `slay-tui::save`** with `load_run` / `save_run` / `delete_run` / `load_meta` / `save_meta`.
4. **Wire save into `main.rs`**: check for `run.ron` at startup; offer Continue/New.
5. **Wire auto-save** into `apply_and_drain` caller (after every successful command). Clone state, send to a `std::sync::mpsc` channel; a background thread owns the channel receiver and does the actual `fs::write`. Only one writer thread needed — channel naturally queues and the last write wins.
6. **Wire graceful shutdown**: use `ctrlc` crate to catch SIGINT; drop sender, join writer thread, restore terminal, exit. Hooks into the existing TUI terminal-restore path.
7. **Delete run save on `GameOver`**; update `meta.ron`.
8. **Add `MetaSave` and unlock gating** to `reward_pool` (can be deferred).

Each step is independently committable and leaves the game working.

---

## Resolved Decisions

- **Continue prompt**: always ask — "Continue run? [y/n]" — never silently resume.
- **Auto-save cadence**: after every decision (every successful `apply_and_drain` call). Write is done on a background thread so it never blocks the UI. The current `GameState` is cloned, sent to the thread via a channel, and serialized there. A dropped/failed write is logged but does not surface as an error to the player (the in-memory state is always authoritative).
- **Graceful shutdown (ctrl+c)**: with a naïve background-write approach, ctrl+c kills the process mid-flush and the save is one decision behind. Fix: use the `ctrlc` crate to register a SIGINT handler that sets a flag; the main loop detects it, drops the save-channel sender (closes the channel), joins the writer thread (draining any queued write), restores the terminal, then exits. The TUI already needs a terminal-restore cleanup path for ctrl+c — the save flush slots into the same place. `kill -9` or power loss will always lose the last decision; that's unavoidable without blocking the UI and is acceptable.
- **Cloud saves / multi-profile**: out of scope.

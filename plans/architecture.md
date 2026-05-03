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
        run.rs                ← GameState, Command, CommandError, apply_command, MAP_NODES
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
          mod.rs              ← EnemyKind, EnemyDef, Intent, next_intent dispatch
          louse.rs            ← louse::DEF, louse::next_intent
        status.rs             ← StatusEffect, StatusMap, resolve_damage, tick_statuses, drain_poison
        types.rs              ← Hp, Block, Energy newtypes
        rng.rs                ← Rng trait, ThreadRng, NoOpRng
    slay-tui/
      src/
        main.rs               ← event loop, render functions, describe(Event)
        command.rs            ← text + GameState context → Command (parse)
        lib.rs                ← re-exports command module for integration tests
      tests/
        integration.rs        ← TestHarness; command sequences → GameState assertions
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

    // Rest site
    Rest,

    // Card reward
    ChooseCardReward(usize), // 0-indexed option
    SkipReward,
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

## GameState and Transitions

```rust
// run.rs
pub enum GameState {
    Map(MapState),
    Combat { state: CombatState, floor: usize },
    RestSite(RestSiteState),
    CardReward(CardRewardState),
    GameOver { victory: bool },
}
```

### Transition table

| Current state | Command | Next state |
|---|---|---|
| `Map` | `ChooseNode(0)` → Combat node | `Combat { floor }` |
| `Map` | `ChooseNode(0)` → Rest node | `RestSite { floor }` |
| `Combat` | any → `Victory` (non-boss) | `CardReward { floor+1 }` |
| `Combat` | any → `Victory` (boss) | `GameOver { victory: true }` |
| `Combat` | any → `Defeat` | `GameOver { victory: false }` |
| `RestSite` | `Rest` | `Map { floor+1 }` |
| `CardReward` | `ChooseCardReward(i)` | `Map { same floor }` |
| `CardReward` | `SkipReward` | `Map { same floor }` |
| `GameOver` | any | `Err(CombatOver)` |

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
```

All randomness routes through `shuffle`. Currently used for: shuffling draw pile, shuffling card reward pool. Production code passes `&mut ThreadRng`. Tests pass `&mut NoOpRng` for determinism.

**`NoOpRng` is an identity shuffle** — slice order is unchanged. Test setup must account for this: draw order = reverse of deck construction order (drawn via `pop()`).

**Extension point:** The `Rng` trait only exposes `shuffle`. When random number generation beyond shuffling is needed (e.g. random damage ranges, proc chances), add methods to the trait.

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

### `command.rs`

```rust
pub fn parse(input: &str, state: &GameState) -> Option<Command>
```

Context-aware: the same input ("1") means different things in different states. Returns `None` on unknown input; the main loop prints "Unknown command." and re-prompts. Does not mutate state.

| State | Input | Command |
|---|---|---|
| Map | `"1"` | `ChooseNode(0)` |
| Combat | `"1"`–`"N"`, `"play N"` | `PlayCard(N-1)` |
| Combat | `"end"`, `"e"`, `"end turn"`, `"pass"` | `EndTurn` |
| RestSite | `"rest"`, `"r"` | `Rest` |
| CardReward | `"1"`–`"N"`, `"pick N"` | `ChooseCardReward(N-1)` |
| CardReward | `"skip"`, `"s"` | `SkipReward` |

Pile inspection (`"z"`, `"x"`, `"c"`) is handled directly in `main.rs` before `parse` is called — it is a display operation, not a `Command`.

### `main.rs`

The event loop:
1. Render current state.
2. Read a line.
3. Check for pile inspection shortcuts (`z`/`x`/`c` in combat) → print pile, re-prompt.
4. Parse input → `Command`; on `None` print "Unknown command."
5. Call `apply_command`; on `Err` print error.
6. Auto-drain `EnemyTurn`: loop `EndEnemyTurn` until phase is no longer `EnemyTurn`.
7. Check for `GameOver`; break if so.
8. Print events, re-render state.

### Integration tests (`tests/integration.rs`)

`TestHarness` wraps `GameState` and exposes `send(input: &str)` which parses and applies a command, then auto-drains `EnemyTurn` — mirroring the main loop. Used for end-to-end verification of the full stack.

---

## Testing Strategy

| Layer | Location | What it tests |
|---|---|---|
| Damage formula | `status.rs` tests | `resolve_damage` math in isolation |
| Per-card effects | `cards/mod.rs` tests | What each card does when played |
| Combat engine | `combat.rs` tests | Drawing, energy, phases, block, turn cycle, status tick-down |
| Run/progression | `run.rs` tests | Map transitions, rest, rewards, gold, floor progression |
| TUI integration | `slay-tui/tests/integration.rs` | Text commands → `GameState` assertions through the full stack |

All `slay-core` tests use `NoOpRng`. No mocks — tests exercise real code paths.

---

## Key Architecture Decisions

### Fixed constraints

- **`GameState` is owned and cloned, never mutated in-place.** `apply_command` takes ownership and returns a new state. This makes every state transition a pure function, enabling deterministic replay and trivial testing.
- **Card modules cannot see whether they killed the enemy.** Death detection (`enemy.hp <= 0`) is centralised in the `PlayCard` handler in `combat.rs`. Card modules only apply their effect. This prevents double-emitting `EnemyDied`.
- **`exhaust_pile` is scoped to one combat.** `player_after_combat` returns exhausted cards to `deck` and clears the pile. For a relic that permanently exhausts cards, a separate permanent-exhaust mechanism would be needed.

### Flexible / designed to change

- **`enemy_for_floor(floor)`** is a stub returning `EnemyKind::Louse` for all floors. Replace the `match` body when Phase 6 (Fungibeast) lands.
- **`Card::exhausts()` covers only self-exhaust.** Random exhaust ("exhaust a random card in hand") or player-choice exhaust require a different mechanism — likely a return value from `cards::apply` or a flag set on `CombatState` during card execution.
- **`Rng` trait only exposes `shuffle`.** Extend with `next_u32` or similar when proc chances or random damage ranges are needed.
- **`MAP_NODES` is a fixed slice.** Phase 8 (branching map) will require replacing this with a graph structure on `MapState`.
- **`Intent` is `Attack(i32) | Defend(i32)`.** This covers all current enemies. Future intents (buff, debuff, multi-hit) require new variants. This is the most likely enum to grow.
- **The event stream is the relic hook point.** Phase 10's `RelicEffect::on_event(&Event, &mut CombatState)` slots in after each event is emitted. No plumbing change is needed in the engine — the caller (`run.rs`) would iterate active relics after each `apply_combat_command` call.
- **`resolve_damage` has no relic hooks.** If a relic needs to modify the formula (e.g. "Paper Krane: take 40% less damage"), the function signature or a context parameter would need to change.

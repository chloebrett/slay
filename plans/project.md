# Project Plan: Slay the Spire Terminal Clone

## Guiding Principle

Build **vertically**, not horizontally. Each phase must produce a complete, playable game — albeit a limited one. We extend depth at each layer, never breadth without depth.

TDD is non-negotiable at every phase. The core is pure and testable. The TUI is thin but also integration-tested via its command interface.

---

## Architecture

### Two-Crate Workspace

```
slay/
  Cargo.toml           ← workspace manifest
  crates/
    slay-core/         ← pure game engine, no I/O, no terminal concepts
      Cargo.toml
      src/
        lib.rs
        combat.rs
        entities.rs
        cards.rs
        status.rs
        map.rs
        events.rs
    slay-tui/          ← terminal frontend
      Cargo.toml
      src/
        main.rs        ← entry point, event loop
        command.rs     ← text → Command parsing
        render.rs      ← GameState → terminal output
      tests/
        integration.rs ← command sequence → state assertions
```

**Hard constraint on `slay-core`:** It must contain zero terminal, I/O, or display concepts. No `println!`. No `crossterm`. No string rendering. A hypothetical `slay-gui` crate could depend on `slay-core` and build a 2D graphical game using the exact same engine. The core speaks only in game types.

**Hard constraint on `slay-tui`:** It must contain zero game logic. It only translates: text → `Command`, `Command` → `slay-core`, `GameState` → terminal output. All branching on game state belongs in `slay-core`.

---

### The Command Interface (Central Design Decision)

The core exposes a single entry point:

```rust
// slay-core
pub fn apply_command(state: GameState, command: Command) -> Result<(GameState, Vec<Event>), CommandError>
```

`GameState` is the complete, owned snapshot of the game at a point in time. It is immutable from the outside — `apply_command` returns a new state. This makes the game fully replayable from any point (replay a command log) and trivially testable.

`Command` is a semantic enum, not text:

```rust
pub enum Command {
    Attack,             // Phase 1: hardcoded
    Block,              // Phase 1: hardcoded
    EndTurn,
    PlayCard(usize),    // Phase 2+: index into hand
    ChooseCardReward(usize), // Phase 5+
    Rest,               // Phase 5+
    Smith(usize),       // Phase 5+
}
```

`slay-tui` parses text into `Command`. The LLM or human types `attack`, `block`, `end`, `play 2` — `slay-tui` maps these to `Command` variants and calls `apply_command`. The game logic never knows whether a human or an agent issued the command.

`Vec<Event>` is the log of what happened (damage dealt, block gained, enemy died, etc.). `slay-tui` uses this to produce human-readable output. An LLM integration could consume these events directly as structured data.

---

### Randomness Abstraction

All randomness is injected, never global:

```rust
// slay-core
pub trait Rng {
    fn next_u32(&mut self) -> u32;
    fn shuffle<T>(&mut self, slice: &mut [T]);
    fn choose_index(&mut self, len: usize) -> usize {
        (self.next_u32() as usize) % len
    }
}

pub struct ThreadRng(rand::rngs::ThreadRng);
pub struct SeededRng(rand::rngs::StdRng);
pub struct FixedRng(Vec<u32>, usize); // deterministic, for tests
```

Functions that need randomness take `&mut dyn Rng`. Tests pass a `FixedRng` or `SeededRng`. The production game passes `ThreadRng`.

---

### Core Data Model

```rust
// Phase 1 subset — grows without structural changes
pub struct Player {
    pub hp: i32,
    pub max_hp: i32,
    pub block: i32,
    // Phase 2+
    pub energy: i32,
    pub max_energy: i32,
    pub hand: Vec<Card>,
    pub draw_pile: Vec<Card>,
    pub discard_pile: Vec<Card>,
    pub deck: Vec<Card>,      // master deck, persists between combats
    // Phase 4+
    pub statuses: HashMap<StatusEffect, i32>,
    // Phase 5+
    pub gold: i32,
    pub relics: Vec<Relic>,
}

pub struct Enemy {
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub block: i32,
    pub intent: Intent,
    pub statuses: HashMap<StatusEffect, i32>,
}

pub enum Intent { Attack(i32), Defend(i32), Buff, Unknown }

pub struct CombatState {
    pub player: Player,
    pub enemy: Enemy,
    pub turn: u32,
    pub phase: CombatPhase,
}

pub enum CombatPhase { PlayerTurn, EnemyTurn, Victory, Defeat }

pub enum GameState {
    Combat(CombatState),
    // Phase 5+
    MapView(MapState),
    CardReward(CardRewardState),
    RestSite(RestSiteState),
    GameOver { victory: bool },
}
```

### Integer Types

Use `i32` for all game values — HP, block, energy, damage, gold, status stacks — even when the business logic says the value must be non-negative. Reasons:

- Unsigned subtraction in Rust underflows or panics; `saturating_sub` everywhere is noisy
- Damage math naturally produces intermediate negatives (e.g. over-block); `max(0)` at the end is cleaner than fighting the type system throughout
- Mixing `u32` and `i32` forces casts that obscure the logic

Non-negativity invariants are enforced by the damage pipeline and game logic, not the type system. If the codebase grows large enough that passing `block` where `hp` is expected becomes a real risk, introduce newtypes (`struct Hp(i32)`) at that point — the refactor is mechanical.

Exception: `usize` where Rust APIs require it (Vec indices, lengths).

### Card Representation

Closed enum — idiomatic Rust for a finite, known set. Exhaustive matching catches missing cases at compile time:

```rust
pub enum Card {
    Strike, Defend, Bash, // ...
}

impl Card {
    pub fn energy_cost(&self) -> i32 { ... }
    pub fn apply(&self, state: &mut CombatState, rng: &mut dyn Rng) { ... }
    pub fn name(&self) -> &str { ... }
    pub fn description(&self) -> &str { ... }
}
```

### Damage Pipeline (Phase 4 forward)

Never apply damage directly. Always route through the pipeline:

```rust
pub fn resolve_damage(base: i32, attacker: &Player, defender: &Enemy) -> i32 {
    let dmg = base;
    let dmg = apply_strength(dmg, &attacker.statuses);
    let dmg = apply_weak(dmg, &attacker.statuses);
    let dmg = apply_vulnerable(dmg, &defender.statuses);
    dmg.max(0)
}

pub fn deal_damage(damage: i32, target: &mut Enemy) -> i32 {
    let absorbed = damage.min(target.block);
    target.block -= absorbed;
    let remainder = damage - absorbed;
    target.hp -= remainder;
    remainder  // actual HP damage dealt (for events)
}
```

---

## Testing Strategy

### slay-core: Unit Tests

All game logic lives here and is tested with `#[cfg(test)]` modules directly in each source file. No terminal, no I/O. Tests are fast, deterministic (using `FixedRng`), and exhaustive.

```rust
// combat.rs
#[cfg(test)]
mod tests {
    #[test]
    fn player_attack_reduces_enemy_hp() {
        let state = CombatState::new_test();
        let (state, _) = apply_command(state, Command::Attack, &mut FixedRng::default()).unwrap();
        assert_eq!(state.enemy.hp, state.enemy.max_hp - 6);
    }
}
```

### slay-tui: Integration Tests

`slay-tui/tests/integration.rs` issues text command sequences and asserts on resulting `GameState`. This tests the full pipeline: text parsing → `apply_command` → state. These are the tests an LLM harness would also exercise.

```rust
// slay-tui/tests/integration.rs
#[test]
fn player_can_kill_enemy_with_attacks() {
    let mut game = TestHarness::new(); // starts a fresh combat
    while !game.is_over() {
        game.send("attack");
        if !game.is_player_turn() { game.send("end"); }
    }
    assert!(game.state().is_victory());
}

#[test]
fn block_reduces_incoming_damage() {
    let mut game = TestHarness::new();
    game.send("block");
    game.send("end");  // enemy attacks on its turn
    assert!(game.player_hp() > game.player_max_hp() - 8);
}
```

`TestHarness` wraps `slay-tui`'s command parser + `apply_command`, with a fixed/seeded `Rng`.

---

## Phase Breakdown

### Phase 1: The Raw Combat Loop

**Goal:** A complete, playable (if tiny) game. Win and lose conditions. No cards. Validates the full architecture end-to-end.

**slay-core mechanics:**
- `Player`: 80 HP, 0 block.
- `Enemy`: `Louse`, 20 HP, attacks for 8 each turn.
- Commands: `Attack` (deal 6 damage to enemy), `Block` (gain 5 block), `EndTurn`.
- Block resets to 0 at start of player's turn.
- Combat ends when either HP reaches zero.

**slay-tui:**
- Parses: `"attack"`, `"block"`, `"end"` (and aliases `"end turn"`, `"pass"`).
- Renders: player HP/block, enemy HP/intent, available commands, event log.
- No `crossterm` raw mode yet — line-buffered stdin (simpler, correct for command orientation).

**slay-core tests (RED first):**
- `attack_deals_6_damage_to_enemy`
- `block_grants_5_block_to_player`
- `end_turn_triggers_enemy_attack`
- `enemy_attack_reduced_by_block`
- `enemy_attack_excess_damages_hp`
- `block_resets_at_start_of_player_turn`
- `player_death_yields_defeat_state`
- `enemy_death_yields_victory_state`
- `commands_rejected_when_not_player_turn`
- `commands_rejected_after_combat_ends`

**slay-tui integration tests (RED first):**
- `attack_command_parses_and_reduces_enemy_hp`
- `unknown_command_returns_error_not_crash`
- `player_wins_by_attacking_until_enemy_dead`
- `block_then_end_turn_reduces_damage_taken`

**Crates added:** `rand` (in slay-core, for future use), `crossterm` (in slay-tui, for later)

---

### Phase 2: The Deck

**Goal:** Real card mechanics. The game now feels like a card game.

**Additions:**
- `Card` enum: `Strike` (1 energy, 6 damage), `Defend` (1 energy, 5 block).
- Starting deck: 5× Strike, 4× Defend, 1× Bash (preview; Bash implemented in Phase 4).
- Piles: draw pile (shuffled from deck), hand (max 10), discard pile.
- 3 energy per turn. Draw 5 at turn start. Discard hand at turn end.
- Draw pile empty → shuffle discard into draw pile.
- `Command::PlayCard(usize)` replaces `Attack`/`Block`.
- `Command::EndTurn` still exists.
- TUI parses: `"play 1"`, `"play 2"`, etc.

---

### Phase 3: Enemy Intents

**Goal:** Telegraphed enemies.

**Additions:**
- `Intent` on `Enemy`. Computed at start of player turn, displayed, executed on enemy turn.
- Simple `Louse` AI: alternates `Attack(8)` and `Defend(5)`.
- Enemy block works symmetrically to player block.

---

### Phase 4: Status Effects & Damage Pipeline

**Goal:** The modifier system. Architecture stress test.

**Additions:**
- `StatusEffect`: `Strength`, `Vulnerable`, `Weak`, `Poison`.
- `HashMap<StatusEffect, i32>` on Player and Enemy.
- All damage via pipeline. No raw HP subtraction anywhere.
- Status durations tick at end of turn.
- New card: `Bash` (2 energy, 8 damage + 2× Vulnerable).

---

### Phase 5: Map & Run Progression

**Goal:** A full run with persistence between encounters.

**Additions:**
- `MapNode`: `Combat(EnemyId)`, `RestSite`, `Boss(EnemyId)`.
- Linear map: 3× combat → rest site → boss.
- Post-combat card reward: choose 1 of 3 (`Command::ChooseCardReward(usize)`).
- Rest site: `Command::Rest` (heal 30% max HP) or `Command::Smith(usize)` (upgrade a card).
- Second enemy type.
- `GameState` becomes an enum wrapping `CombatState`, `MapState`, etc.

---

### Phase 6: Relic System

**Goal:** Rule-breaking relics via event hooks.

**Additions:**
- `GameEvent` enum emitted alongside every state transition.
- `RelicEffect` trait: `fn on_event(&self, event: &GameEvent, state: &mut CombatState)`.
- `BurningBlood` (heal 6 after combat), `Vajra` (+1 Strength on combat start).
- Relics as run rewards.

---

### Phase 7: TUI Polish

**Goal:** `ratatui` layout replacing raw `println!`. The engine is unchanged.

- Top panel: enemy name, HP bar, intent, statuses.
- Centre: scrollable combat log.
- Bottom: hand (card name, cost, description boxes).
- Left panel: player HP, block, energy, statuses, relics.
- Command input line at bottom.

**Crates added:** `ratatui`

---

### Phase 8: Content Expansion

- Full Ironclad card set
- Elite encounters
- Curse cards, Merchant node
- Branching map

---

## Crate Plan

| Crate | Belongs to | Purpose | Added in |
|---|---|---|---|
| `rand` | slay-core | Shuffling, card rewards, AI variation | Phase 1 |
| `crossterm` | slay-tui | Terminal input, cursor | Phase 1 |
| `ratatui` | slay-tui | Full TUI layout | Phase 7 |

No async. Minimal dependencies until Phase 7.

---

## TDD Notes

- All `slay-core` logic: `#[cfg(test)]` modules, tested directly, deterministic via `FixedRng`.
- `slay-tui` integration tests: issue text commands, assert on `GameState`.
- Mutation testing (`cargo-mutants`) after each phase.
- One failing test before every production code change.
- Commit after each RED→GREEN→REFACTOR cycle.

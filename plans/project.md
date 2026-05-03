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
        combat.rs      ← CombatState, apply_combat_command, Command, Event
        run.rs         ← GameState, apply_command, map/rest/reward logic
        cards/         ← Card enum, CardDef, per-card effect modules
        enemies/       ← EnemyKind, Intent, per-enemy AI modules
        status.rs      ← StatusEffect, StatusMap, damage pipeline
        types.rs       ← Hp, Block, Energy newtypes
        rng.rs         ← Rng trait, ThreadRng, NoOpRng
    slay-tui/          ← terminal frontend
      Cargo.toml
      src/
        main.rs        ← entry point, event loop, rendering
        command.rs     ← text + GameState context → Command
      tests/
        integration.rs ← command sequence → GameState assertions
```

**Hard constraint on `slay-core`:** No terminal, I/O, or display concepts. No `println!`. No `crossterm`. A hypothetical `slay-gui` could depend on `slay-core` and build a graphical game using the same engine. The core speaks only in game types.

**Hard constraint on `slay-tui`:** No game logic. It only translates: text → `Command`, `Command` → `slay-core`, `GameState` → terminal output.

---

### The Command Interface

```rust
// slay-core/run.rs — top-level entry point
pub fn apply_command(state: GameState, command: Command, rng: &mut impl Rng)
    -> Result<(GameState, Vec<Event>), CommandError>
```

`GameState` is the complete, owned snapshot. `apply_command` returns a new state — fully replayable, trivially testable.

```rust
pub enum Command {
    PlayCard(usize),
    EndTurn,
    EndEnemyTurn,          // internal: TUI auto-drains EnemyTurn phase
    ChooseNode(usize),
    Rest,
    ChooseCardReward(usize),
    SkipReward,
}
```

---

### Randomness Abstraction

```rust
pub trait Rng {
    fn shuffle<T>(&mut self, slice: &mut [T]);
}

pub struct ThreadRng(rand::rngs::ThreadRng); // production
pub struct NoOpRng;                           // identity shuffle, for tests
```

All randomness is injected. Tests use `NoOpRng`. Production uses `ThreadRng`.

---

### Core Data Model

```rust
pub struct Player {
    pub hp: Hp,
    pub max_hp: Hp,
    pub block: Block,
    pub energy: Energy,
    pub max_energy: Energy,
    pub hand: Vec<Card>,
    pub draw_pile: Vec<Card>,
    pub discard_pile: Vec<Card>,
    pub deck: Vec<Card>,        // master deck, persists between combats
    pub statuses: StatusMap,    // IndexMap<StatusEffect, i32>, insertion-ordered
    pub gold: i32,
}

pub struct Enemy {
    pub kind: EnemyKind,
    pub hp: Hp,  pub max_hp: Hp,  pub block: Block,
    pub intent: Intent,
    pub statuses: StatusMap,
}

pub enum Intent { Attack(i32), Defend(i32) }

pub struct CombatState {
    pub player: Player,
    pub enemy: Enemy,
    pub turn: u32,
    pub phase: CombatPhase,
}

pub enum CombatPhase { PlayerTurn, EnemyTurn, Victory, Defeat }

pub enum GameState {
    Map(MapState),
    Combat { state: CombatState, floor: usize },
    RestSite(RestSiteState),
    CardReward(CardRewardState),
    GameOver { victory: bool },
}
```

### Integer Types and Newtypes

Use `i32` for all numeric game values. Three newtypes guard the damage pipeline, where transposing HP and Block is a real bug risk:

```rust
pub struct Hp(pub i32);
pub struct Block(pub i32);
pub struct Energy(pub i32);
```

Everything else stays plain `i32`: damage amounts, gold, status stacks, turn count.

### Card Representation

Closed enum with per-card effect modules in `cards/`:

```rust
pub enum Card { Strike, Defend, Bash, Clothesline, Inflame, DeadlyPoison }
```

`CardDef` holds static data (name, description template, energy cost, base damage). `effective_description()` substitutes live values with `*N*` emphasis when modified by statuses.

### Damage Pipeline

```rust
pub fn resolve_damage(base: i32, attacker: &StatusMap, defender: &StatusMap) -> i32 {
    let dmg = base + strength(attacker);
    let dmg = if weak(attacker)       { dmg * 3 / 4 } else { dmg };
    let dmg = if vulnerable(defender) { dmg * 3 / 2 } else { dmg };
    dmg.max(0)
}
```

All damage routes through this. No raw HP subtraction anywhere.

---

## Testing Strategy

### slay-core: Unit Tests

In-file `#[cfg(test)]` modules. Fast, deterministic via `NoOpRng`.

```rust
#[test]
fn strike_deals_6_damage_to_enemy() {
    let state = combat_with_hand(vec![Card::Strike]);
    let (state, _) = apply_command(state, Command::PlayCard(0), &mut NoOpRng).unwrap();
    assert_eq!(state.enemy.hp, Hp(14));
}
```

### slay-tui: Integration Tests

`slay-tui/tests/integration.rs` — `TestHarness` wraps `GameState`, issues text commands, asserts on resulting state.

```rust
#[test]
fn play_strike_reduces_enemy_hp() {
    let mut game = TestHarness::with_hand(vec![Card::Strike]);
    game.send("play 1").unwrap();
    assert_eq!(game.enemy_hp(), 14);
}
```

---

## Phase Breakdown

### Phase 1 ✅ — Raw Combat Loop

**Goal:** Playable game with win/lose. No cards. Validates the full architecture end-to-end.

- 80 HP player vs 20 HP Louse (attacks 8/turn)
- Hardcoded `Attack` (6 dmg) and `Block` (5 block); `EndTurn`
- Block resets at start of player's turn
- Line-buffered stdin; no raw mode

---

### Phase 2 ✅ — The Deck

**Goal:** Real card mechanics.

- `Card` enum: `Strike` (1 energy, 6 dmg), `Defend` (1 energy, 5 block)
- Starting deck: 5× Strike, 3× Defend, 1× Bash, 1× Inflame, 1× Deadly Poison
- 3 energy/turn; draw 5 at turn start; discard hand at turn end
- Draw pile empty → shuffle discard into draw pile
- `Command::PlayCard(usize)` replaces hardcoded attack/block

---

### Phase 3 ✅ — Enemy Intents

**Goal:** Telegraphed enemies.

- `Intent` on `Enemy`; computed at start of player turn, executed on enemy turn
- Louse alternates `Attack(8)` and `Defend(5)`
- Enemy block works symmetrically to player block

---

### Phase 4 ✅ — Status Effects & Damage Pipeline

**Goal:** The modifier system.

- `StatusEffect`: `Vulnerable`, `Weak`, `Poison`, `Strength`
- `StatusMap = IndexMap<StatusEffect, i32>` (insertion-ordered)
- Damage formula: `(base + strength) × weak × vulnerable`
- `Vulnerable`/`Weak` tick down at end of enemy turn; `Poison` drains before enemy acts; `Strength` is permanent
- Cards: `Bash` (2 energy, 8 dmg + 2 Vulnerable), `Clothesline` (2 energy, 12 dmg + 2 Weak), `Inflame` (1 energy, +2 Strength), `Deadly Poison` (1 energy, +5 Poison)

---

### Phase 5 ✅ — Map & Run Progression

**Goal:** Full run loop with persistence between encounters.

- `GameState` enum: `Map`, `Combat`, `RestSite`, `CardReward`, `GameOver`
- Linear map: Combat → Combat → Combat → Rest Site → Boss (all Louse)
- `Player` gains `deck` (master, persists across combats) and `gold`
- 50 gold per combat victory; displayed on map, not in combat
- Rest site: `Rest` heals 30% max HP
- Post-combat card reward: choose 1 of 3 shuffled options, or skip
- Top-level `apply_command(GameState, ...)` in `run.rs`; combat engine is `pub(crate)`

---

### Phase 6 — Second Enemy

**Goal:** Variety in encounters.

- `Fungibeast` enemy: 22 HP, alternates `Attack(6)` and `Attack(10)`
- Floor-based enemy selection: floor 0 = Louse, floor 1 = Fungibeast, floor 2 = Louse, floor 4 = Louse (boss)

---

### Phase 7 — Card Upgrades & Debug Mode

**Goal:** Deeper card strategy; developer tooling.

- Upgraded cards: Strike 6→9, Defend 5→8, Bash 8→10 (+ 2→3 Vuln), Clothesline 12→14 (+ 2→3 Weak), Deadly Poison unchanged, Inflame unchanged
- Rest site gains an `upgrade` option alongside `rest`
- Debug flag (`--debug`): `skip` advances to next floor; `win` sets enemy HP to 0
- Tests verify debug commands are rejected without the flag

---

### Phase 8 — Multiple Enemies & Targeting

**Goal:** Boss fights with real stakes; pile inspection.

- Boss battle: two Lice instead of one
- `Command::PlayCard(card_idx, target_idx)` — player specifies which enemy to target
- View draw pile (`z`) or discard pile (`x`) at any time during combat

---

### Phase 9 — More Cards

- `Cleave` (1 energy): 8 damage to ALL enemies. Upgraded: 11 damage.
- `Pommel Strike` (1 energy): 9 damage + draw 1 card. Upgraded: 10 damage + draw 2.

---

### Phase 10 — Relic System

**Goal:** Rule-breaking relics via event hooks.

- `RelicEffect` trait: `fn on_event(&self, event: &Event, state: &mut CombatState)`
- `BurningBlood` (heal 6 after combat), `Vajra` (+1 Strength on combat start)
- Relics offered as post-combat rewards alongside card rewards

---

### Phase 11 — TUI Polish

**Goal:** `ratatui` layout replacing raw `println!`.

- Top panel: enemy name, HP bar, intent, statuses
- Left panel: player HP, block, energy, statuses, relics, gold
- Centre: scrollable combat log
- Bottom: hand with card boxes and cost
- Input line at bottom

**Crates added:** `ratatui`, `crossterm`

---

### Phase 12 — Content Expansion

- Full Ironclad card set
- Elite encounters
- Curse cards
- Merchant node
- Branching map

---

## Crate Plan

| Crate       | Belongs to | Purpose                             | Added in  |
| ----------- | ---------- | ----------------------------------- | --------- |
| `rand`      | slay-core  | Shuffling, card rewards             | Phase 1   |
| `indexmap`  | slay-core  | Insertion-ordered status tracking   | Phase 4   |
| `crossterm` | slay-tui   | Terminal input, cursor              | Phase 11  |
| `ratatui`   | slay-tui   | Full TUI layout                     | Phase 11  |

No async. Minimal dependencies until Phase 11.

---

## TDD Notes

- All `slay-core` logic: `#[cfg(test)]` modules, deterministic via `NoOpRng`
- `slay-tui` integration tests: issue text commands, assert on `GameState`
- Mutation testing (`cargo-mutants`) after each phase
- One failing test before every production code change
- Commit after each RED→GREEN→REFACTOR cycle

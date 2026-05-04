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
        run.rs                ← GameState, Command, CommandError, apply_command, Scenario, map types
        combat.rs             ← CombatState, apply_combat_command, Player, Enemy, Event
        cards/
          mod.rs              ← Card enum, Grade, CardDef, CardType, dispatch, reward_pool, starter_deck
          tests.rs            ← all card tests (kept separate to keep mod.rs a thin router)
          anger.rs
          bash.rs
          blood_wall.rs
          bloodletting.rs
          bludgeon.rs
          body_slam.rs
          breakthrough.rs
          cleave.rs
          clothesline.rs
          dazed.rs
          deadly_poison.rs
          defend.rs
          disarm.rs
          hemokinesis.rs
          impervious.rs
          inflame.rs
          iron_wave.rs
          mangle.rs
          not_yet.rs
          pommel_strike.rs
          shrug_it_off.rs
          strike.rs
          taunt.rs
          thunderclap.rs
          tremble.rs
          twin_strike.rs
          uppercut.rs
        enemies/
          mod.rs              ← EnemyKind, EnemyDef, Move, Intent, next_move dispatch; id()/from_id()
          blue_slaver.rs
          cultist.rs
          fungibeast.rs
          green_louse.rs
          jaw_worm.rs
          louse.rs
          red_louse.rs
          red_slaver.rs
          small_acid_slime.rs
          small_spike_slime.rs
        relics.rs             ← Relic enum, grant_relic, relic hook functions
        potions.rs            ← Potion enum, PotionDef, MAX_POTIONS
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
    PlayCard(usize, usize),    // card index, target enemy index (0-indexed)
    EndTurn,
    EndEnemyTurn,              // internal — TUI auto-drains; player never issues this

    // Potions
    UsePotion(usize, usize),   // slot index, target enemy index
    DiscardPotion(usize),      // slot index

    // Map navigation
    ChooseNode(usize),         // column index among available_cols
    Spawn(Vec<EnemyKind>),     // override next combat's enemies (always available on Map)

    // Rest site
    Rest,
    UpgradeCard(usize),        // 0-indexed deck position

    // Card reward
    ChooseCardReward(usize),   // 0-indexed option
    SkipReward,

    // Shop
    BuyCard(usize),            // 0-indexed shop card position
    BuyRelic,
    BuyPotion,
    LeaveShop,

    // Debug
    WinCombat,                 // instantly win the current combat
    SkipFloor,                 // advance past current floor without fighting
    AddCard(Card),             // add a card to hand mid-combat
    AddRelic(Relic),           // grant a relic
    AddPotion(Potion),         // add a potion to inventory
}
```

**`Command` lives in `run.rs`** because the full set of variants is dispatched at the run level. `combat.rs` only handles combat variants and rejects the rest with `InvalidPhase`.

### Errors

```rust
// run.rs
pub enum CommandError {
    CombatOver,       // command issued after Victory or Defeat
    InvalidCard,      // out-of-bounds index, or unavailable map column
    NotEnoughEnergy,  // PlayCard when player can't afford it
    NotEnoughGold,    // Buy* when player doesn't have enough gold
    InvalidPhase,     // command not valid in current GameState
    Entangled,        // played an Attack card while Entangled
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
- `new_run(rng)` → `Scenario::Main`, populated deck, full 10-floor graph
- `new_simple_run()` → `Scenario::Simple`, empty deck, minimal 1-node graph
- `generate_map(rng) -> MapGraph` → standalone graph generation

---

## GameState and Transitions

```rust
// run.rs
pub enum GameState {
    Map(MapState),
    Combat { state: CombatState, floor: usize, is_boss: bool, graph: MapGraph, next_floor_cols: Vec<usize>, scenario: Scenario },
    RestSite(RestSiteState),
    CardReward(CardRewardState),
    Shop(ShopState),
    GameOver { victory: bool },
}
```

### Transition table

| Current state | Trigger | Next state (Main) | Next state (Simple) |
|---|---|---|---|
| `Map` | `ChooseNode(col)` → Combat/Boss node | `Combat { floor, is_boss }` | `Combat { floor, is_boss }` |
| `Map` | `ChooseNode(col)` → RestSite node | `RestSite { floor }` | `RestSite { floor }` |
| `Map` | `ChooseNode(col)` → Merchant node | `Shop { floor }` | `Shop { floor }` |
| `Map` | `Spawn(enemies)` | stays `Map` (queues enemies) | stays `Map` (queues enemies) |
| `Combat` | `Victory` (non-boss, Simple) | — | `Map { same floor }` |
| `Combat` | `Victory` (non-boss, Main) | `CardReward { floor+1 }` | — |
| `Combat` | `Victory` (boss) | `GameOver { victory: true }` | `GameOver { victory: true }` |
| `Combat` | `Defeat` | `GameOver { victory: false }` | `GameOver { victory: false }` |
| `RestSite` | `Rest` | `Map { floor+1 }` | `Map { floor+1 }` |
| `RestSite` | `UpgradeCard(i)` | stays `RestSite` (upgrades card) | stays `RestSite` |
| `CardReward` | `ChooseCardReward(i)` | `Map { same floor }` | — (never reached) |
| `CardReward` | `SkipReward` | `Map { same floor }` | — (never reached) |
| `Shop` | `LeaveShop` | `Map { floor+1 }` | `Map { floor+1 }` |
| `GameOver` | any | `Err(CombatOver)` | `Err(CombatOver)` |

### Map graph

```rust
// run.rs
pub struct MapGraph {
    pub rows: Vec<Vec<MapNode>>,      // rows[floor][col]
    pub edges: Vec<Vec<Vec<usize>>>,  // edges[floor][col] = cols reachable on floor+1
}

pub enum MapNode {
    Combat(Vec<EnemyKind>),   // carries its own enemy list
    RestSite,
    Boss(Vec<EnemyKind>),
    Merchant,
}
```

`generate_map(rng)` produces a 10-floor DAG:

```
Floor  Type          Cols
───────────────────────────
  9    Boss           1    ← convergence; single node
  8    Combat         2
  7    Combat         2
  6    RestSite       1    ← convergence
  5    Combat         2
  4    Combat         2
  3    Merchant       1    ← convergence
  2    Combat         2
  1    Combat         2
  0    Combat         2    ← start; both cols available
```

**`available_cols`** on `MapState` (and all other state structs) tracks which columns the player can reach on the current floor based on their chosen path. `ChooseNode(col)` fails with `InvalidCard` if `col` is not in `available_cols`. After choosing, `next_floor_cols` is derived from `edges[floor][col]` and carried forward into the next state.

Each `MapNode::Combat` / `MapNode::Boss` carries its own `Vec<EnemyKind>` — there is no `enemies_for_floor` lookup. The graph is the single source of truth for what enemies appear where.

### Enemy spawn queue

`MapState.next_enemies: Option<Vec<EnemyKind>>` holds enemies set by `Command::Spawn`. `ChooseNode` consumes it (overriding the node's own enemy list) if `Some`. After use, `next_enemies` is cleared to `None`.

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
    pub relics: Vec<Relic>,
    pub potions: Vec<Potion>,      // max MAX_POTIONS (3) slots
}
```

`deck` is the persistent master copy. At combat start, `from_player` clones `deck` into `draw_pile` (resetting hand/discard/exhaust/block/statuses). `player_after_combat` returns a post-combat `Player` with piles cleared and gold added — `deck`, `hp`, `relics`, and `potions` are preserved.

**`exhaust_pile` is scoped to one combat.** `player_after_combat` extends `deck` with `exhaust_pile` then clears the pile — exhausted cards are back in the master deck for the next fight.

---

## Enemy

```rust
// combat.rs
pub struct Enemy {
    pub kind: EnemyKind,
    pub hp: Hp, pub max_hp: Hp, pub block: Block,
    pub move_: Move,           // current queued move (the enemy's intent)
    pub last_move: Option<Move>,
    pub statuses: StatusMap,
}
```

`move_` holds the enemy's current intent as a concrete `Move` variant. `effective_intent(&player_statuses)` converts it to an `Intent` with status-modified damage. `last_move` prevents enemies from repeating the same move twice in a row where forbidden.

```rust
pub enum Intent {
    Attack(i32),
    Defend(i32),
    AttackDefend(i32, i32),
    Buff,
    Debuff,
}
```

### Adding a new enemy

1. Create `enemies/<name>.rs` with a `DEF: EnemyDef` constant and a `pub fn next_move(last: Option<Move>, rng: &mut impl Rng) -> Move`.
2. Add variants to `EnemyKind` and `Move` in `enemies/mod.rs`.
3. Add arms to `EnemyKind::def()` and `next_move()` dispatch in `enemies/mod.rs`.
4. Wire up enemy appearance in `generate_map()` in `run.rs`.

Nothing else needs to change. The combat engine treats all enemies identically.

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

1. **PlayerTurn** — player plays cards, uses potions, ends turn.
2. **EnemyTurn** — `EndEnemyTurn` fires: enemy block resets, move executes, statuses tick, player block expires, energy resets, cards drawn, turn counter increments, new move computed. Relic turn-end and turn-start hooks fire here. Transitions to `PlayerTurn`.
3. **Victory / Defeat** — terminal; no further commands accepted.

The TUI auto-drains `EnemyTurn` by issuing `EndEnemyTurn` in a loop until the phase is no longer `EnemyTurn`.

### PlayCard handler (fixed logic, not per-card)

The handler in `apply_combat_command` owns: energy deduction, hand removal, `CardPlayed` event, calling `cards::apply`, exhaust/discard routing, `CardExhausted` event, relic card-play hooks, victory check. Per-card modules own only their effect.

**Death check is centralised here.** Card modules do not check `enemy.hp <= 0`. This ensures the kill event always fires exactly once.

**Relic hooks fire after card application.** `apply_card_play_relics` receives the `CardType` played and may grant energy, draw cards, or add stacks — all via the same event channel.

### Discard vs exhaust routing

After `cards::apply`, the handler checks `card.exhausts()`:
- `false` → pushed to `discard_pile`
- `true` → pushed to `exhaust_pile`, `CardExhausted` event emitted

**`exhausts()` is the seam to extend.** Currently only `Disarm` and `Impervious` return `true`.

### CombatState counters

```rust
pub struct CombatState {
    // ...
    pub attacks_this_turn: u32,
    pub skills_this_turn: u32,
    pub attacks_this_combat: u32,
    pub skills_this_combat: u32,
    pub cards_played_this_turn: u32,
    pub extra_draws_next_turn: u32,
}
```

These counters feed relic triggers (e.g. Nunchaku grants energy every 10 attacks, Pocketwatch draws extra cards when ≥3 cards played in a turn). `extra_draws_next_turn` is consumed at turn start.

---

## Cards (`cards/`)

### Card enum and Grade

```rust
pub enum Grade { Base, Plus }

pub enum Card {
    // 25 upgradeable cards
    Strike(Grade), Defend(Grade), Bash(Grade), Clothesline(Grade), Inflame(Grade),
    DeadlyPoison(Grade), Cleave(Grade), IronWave(Grade), Tremble(Grade), TwinStrike(Grade),
    Bludgeon(Grade), Impervious(Grade), NotYet(Grade), Mangle(Grade), Uppercut(Grade),
    Taunt(Grade), Thunderclap(Grade), PommelStrike(Grade), ShrugItOff(Grade),
    Breakthrough(Grade), BloodWall(Grade), Bloodletting(Grade), Hemokinesis(Grade),
    BodySlam(Grade), Anger(Grade),
    // 2 non-upgradeable cards
    Disarm,
    Dazed,   // unplayable status card dealt by enemies
}

pub enum CardType { Attack, Skill, Power }

pub enum CardDescription {
    Static(&'static str),
    WithDamage { template: &'static str, base: i32 },
}

pub struct CardDef {
    pub name: &'static str,
    pub description: CardDescription,
    pub energy_cost: Energy,
    pub card_type: CardType,
}
```

Card methods:
- `def() -> CardDef` — static data (name, description, cost, type)
- `grade() -> Option<Grade>` — `None` for `Disarm`/`Dazed`
- `upgrade() -> Option<Card>` — `None` if already `Plus` or non-upgradeable
- `with_grade(Grade) -> Card` — internal helper used by `upgrade()`
- `id() -> &'static str` — e.g. `"strike"`, `"strike-plus"`
- `from_id(s) -> Option<Card>` — inverse of `id()`
- `name()`, `energy_cost()` — convenience delegates to `def()`
- `description() -> String` — base damage substituted
- `effective_description(attacker, defender) -> String` — live damage with `*N*` emphasis if modified
- `effective_damage(attacker, defender) -> Option<i32>` — `None` for non-damage cards
- `exhausts() -> bool` — `Disarm` and `Impervious` return `true`

### Per-card modules

Each card lives in `cards/<name>.rs` with three functions:

```rust
pub(super) fn def(grade: Grade) -> CardDef    // static card data
pub(super) fn id(grade: Grade) -> &'static str // string identifier
pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, ...)
```

**Each module owns its grade-dependent values.** `apply` receives the `Grade` and branches on it internally — the grade-specific numbers (damage, block, status stacks, draws) live in the same place as `def`, eliminating the need to keep them in sync. `mod.rs::apply` is a thin router:

```rust
// mod.rs — dispatches, nothing more
Card::Strike(g)   => strike::apply(state, events, *g, target),
Card::Bash(g)     => bash::apply(state, events, *g, target),
// ...
```

### Adding a new card

1. Create `cards/<name>.rs` with `def(grade)`, `id(grade)`, and `apply(state, events, grade, ...)`.
2. Add a variant to `Card` in `cards/mod.rs`.
3. Add arms to `Card::def()`, `Card::id()`, `Card::from_id()`.
4. Add `Card::exhausts()` arm if the card exhausts itself.
5. Add arm to `Card::grade()` and `Card::with_grade()` if upgradeable.
6. Add dispatch arm in `cards::apply()` in `mod.rs`.
7. Add `mod <name>;` at the top of `cards/mod.rs`.
8. Add to `reward_pool()` if it should appear as a reward.
9. Add to `starter_deck()` if it should be in the starting deck.
10. Add tests in `cards/tests.rs`.

### Starter deck and reward pool

```rust
pub fn starter_deck() -> Vec<Card>  // 5×Strike(Base), 3×Defend(Base), Bash(Base), ...
pub fn reward_pool() -> Vec<Card>   // all Base-grade cards eligible for post-combat reward
```

`reward_pool` lives in `cards/mod.rs` because it is card knowledge — updated alongside card additions, not in the run orchestrator.

---

## Relic System (`relics.rs`)

```rust
pub enum Relic { Strawberry, Pear, Mango, OldCoin, Whetstone, WarPaint, BurningBlood, BlackBlood,
    Anchor, Vajra, Lantern, BloodVial, BagOfMarbles, RedMask, FestivePopper, Pantograph,
    BagOfPreparation, MercuryHourglass, CaptainsWheel, Chandelier, Candelabra, HornCleat,
    HappyFlower, Pendulum, StoneCalendar, Orichalcum, CloakClasp, RegalPillow,
    Nunchaku, OrnamentalFan, Kunai, Shuriken, Kusarigama, LetterOpener, TuningFork,
    GremlinHorn, Pocketwatch }

pub fn grant_relic(player: &mut Player, relic: Relic, rng: &mut impl Rng) -> Vec<Event>
```

Relics trigger at specific lifecycle points via hook functions called from the combat engine and run orchestrator:

| Hook | Called from | Trigger |
|---|---|---|
| `grant_relic` | `run.rs` | Relic picked up (shop, reward) |
| `apply_combat_start_relics` | `combat.rs` | Combat begins |
| `apply_turn_start_relics` | `combat.rs` | Player turn starts |
| `apply_turn_end_relics` | `combat.rs` | Player turn ends (before enemy turn) |
| `apply_card_play_relics` | `combat.rs` | Card played (receives `CardType`) |
| `apply_enemy_died_relics` | `combat.rs` | Enemy dies |
| `apply_end_of_combat_relics` | `run.rs` | Combat ends (victory) |
| `apply_rest_relics` | `run.rs` | Player rests |

All hooks operate on `&mut Player` (or `&mut CombatState`) and emit events. They check `player.relics.contains(&relic)` for each relevant relic — no separate "active relics" structure.

---

## Potion System (`potions.rs`)

```rust
pub const MAX_POTIONS: usize = 3;

pub enum Potion {
    FirePotion, ExplosivePotion, BlockPotion, StrengthPotion, SwiftPotion,
    FearPotion, WeakPotion, BloodPotion, EnergyPotion,
}
```

`Player.potions` holds up to `MAX_POTIONS` potions. `UsePotion(slot, target)` removes the potion from the slot and applies its effect. `DiscardPotion(slot)` removes it without effect. Targeted potions (e.g. `FirePotion`, `FearPotion`) require a target enemy; non-targeted ones ignore the target parameter.

`Potion::is_targeted()` — `true` for potions that need a target, `false` otherwise.

---

## Status System (`status.rs`)

```rust
pub enum StatusEffect { Vulnerable, Weak, Poison, Strength, Entangled }
pub type StatusMap = IndexMap<StatusEffect, i32>;  // insertion-ordered; value is stack count
```

### Damage formula

```rust
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
| `Entangled` | −1 per turn | end of player turn; blocks playing Attack cards while active |

`tick_statuses` removes the status when stacks reach 0. `drain_poison` returns the damage to deal and decrements; caller applies it to HP (bypassing block).

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

All randomness routes through `shuffle`. Currently used for: shuffling draw pile, shuffling card reward pool, shuffling relic pool. Production code passes `&mut ThreadRng`. Tests pass `&mut NoOpRng` for determinism.

**`AnyRng` is the runtime-selectable wrapper.** The `Rng` trait is not object-safe (`shuffle<T>` is generic), so `Box<dyn Rng>` doesn't work. `AnyRng` uses enum dispatch instead.

**`NoOpRng` is an identity shuffle** — slice order is unchanged. Test setup must account for this: draw order = reverse of deck construction order (drawn via `pop()`).

---

## Events (`combat.rs`)

`Event` is the communication channel from the engine to the TUI.

```rust
pub enum Event {
    CardPlayed { card: Card },
    CardExhausted { card: Card },
    CardAdded { card: Card },
    CardUpgraded { from: Card, to: Card },
    StatusCardAddedToDiscard { card: Card },
    PlayerAttacked { raw: i32, damage: i32 },  // raw = post-formula, damage = post-block
    PlayerBlocked { amount: i32 },
    PlayerBlockExpired { amount: i32 },
    PlayerSelfDamaged { amount: i32 },
    PlayerDied,
    EnemyAttacked { raw: i32, damage: i32 },
    EnemyDefended { amount: i32 },
    EnemyPoisoned { damage: i32 },
    EnemyDied,
    StatusApplied { target: Target, status: StatusEffect, stacks: i32 },
    IntentRevealed { intent: Intent },
    TurnStarted { turn: u32 },
    TurnEnded,
    CardsDrawn { count: usize },
    EnergyGained { amount: i32 },
    GoldEarned { amount: i32 },
    Healed { amount: i32 },
    PotionUsed { potion: Potion },
    PotionAwarded { potion: Potion },
    PotionDiscarded { potion: Potion },
}
```

**`raw` vs `damage`**: `raw` is after the status formula; `damage` is after block absorption. Both are included so the TUI can describe partial blocks.

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

Applies one player command, then auto-drains all `EnemyTurn` ticks (issuing `EndEnemyTurn` until the phase is no longer `EnemyTurn`). Returns the final state and a flat `Vec<Event>`. Both `run_game` and `run_tui` call this — there is no other path that drives the engine forward.

Formatting helpers (shared by both renderers):
- `describe_event(&Event) -> String` — long-form text for the log panel / stdout
- `describe_intent(&Intent) -> String` — what the enemy is about to do
- `status_display(StatusEffect) -> (icon, name)` — emoji + label
- `statuses_inline(&StatusMap) -> String` — compact `[💪3 🪫2]` rendering
- `card_type_icon(CardType) -> &'static str`, `enemy_icon(&Enemy) -> &'static str`

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

`TuiState`:
- `game: GameState` — the current state
- `input_buf: String` — accumulated keystrokes
- `event_log: VecDeque<String>` — last 200 event descriptions for the log panel
- `last_error: Option<String>` — shown in red status line until next successful command
- `show_pile: Option<PileView>` — pile overlay
- `should_quit: bool` — set on game over

**Layout** (every screen):
```
┌─ TOP BAR ─ Length(1) ─ HP bar / energy / block / gold / deck ─────────┐
│                                                                        │
│   MAIN AREA (screen-specific) — Min(0)                                 │
│                                                                        │
├─ STATUS LINE — Length(1) — last error (red) or last event ─────────────┤
├─ INPUT BOX — Length(3) — "> " + input_buf ─────────────────────────────┤
└────────────────────────────────────────────────────────────────────────┘
```

Top bar includes a 20-cell HP bar (`[████░░░░░░░░░░░░░░░░]`) computed by `hp_bar(current, max, width)`. A player with ≥1 HP always shows at least one filled cell.

Combat splits the main area horizontally 55/45: enemies + hand + pile counts on the left; scrollable log on the right. Other screens use a single block.

**`render_frame(f: &mut Frame, tui: &TuiState)`** is a pure function. Tests use ratatui's `TestBackend` to render frames into an in-memory buffer and assert on cell contents — no real terminal needed.

### `main.rs` routing

```
--script <path>            → run_game with file reader
--plain                    → run_game with stdin (always)
stdout/stdin not a TTY     → run_game with stdin (auto-fallback)
otherwise                  → run_tui
```

### `command.rs`

```rust
pub fn parse(input: &str, state: &GameState, debug: bool) -> Option<Command>
```

Context-aware: the same input ("1") means different things in different states.

| State | Input | Command | Debug only |
|---|---|---|---|
| Map | `""`, `"enter"` | `ChooseNode(0)` | |
| Map | `"N"` (N≥1) | `ChooseNode(N-1)` | |
| Map | `"spawn <ids...>"` | `Spawn(Vec<EnemyKind>)` | |
| Map | `"skip"` | `SkipFloor` | ✓ |
| Map | `"relic <id>"` | `AddRelic(relic)` | ✓ |
| Map | `"discard N"` | `DiscardPotion(N-1)` | |
| Combat | `"N"`, `"play N"` | `PlayCard(N-1, 0)` | |
| Combat | `"N T"`, `"play N T"` | `PlayCard(N-1, T-1)` | |
| Combat | `"end"`, `"e"`, etc. | `EndTurn` | |
| Combat | `"use N"` | `UsePotion(N-1, 0)` | |
| Combat | `"use N T"` | `UsePotion(N-1, T-1)` | |
| Combat | `"discard N"` | `DiscardPotion(N-1)` | |
| Combat | `"win"` | `WinCombat` | ✓ |
| Combat | `"add <id>"` | `AddCard(card)` | ✓ |
| Combat | `"relic <id>"` | `AddRelic(relic)` | ✓ |
| Combat | `"potion <id>"` | `AddPotion(potion)` | ✓ |
| RestSite | `"rest"`, `"r"` | `Rest` | |
| RestSite | `"upgrade N"`, `"u N"` | `UpgradeCard(N-1)` | |
| RestSite | `"discard N"` | `DiscardPotion(N-1)` | |
| CardReward | `"N"`, `"pick N"` | `ChooseCardReward(N-1)` | |
| CardReward | `"skip"`, `"s"` | `SkipReward` | |
| CardReward | `"discard N"` | `DiscardPotion(N-1)` | |
| Shop | `"N"` | `BuyCard(N-1)` | |
| Shop | `"r"`, `"buy relic"` | `BuyRelic` | |
| Shop | `"p"`, `"buy potion"` | `BuyPotion` | |
| Shop | `"leave"`, `"l"` | `LeaveShop` | |

Pile inspection (`"z"`, `"x"`, `"c"`) is handled in `game::run_game` and `tui.rs` before `parse` is called — it is a display operation, not a `Command`.

**`spawn` is always available on the Map** (not debug-gated) so that `.slay` scripts work without `--debug`.

### Integration tests (`tests/integration.rs`)

`TestHarness` wraps `GameState` and exposes `send(input: &str)` which parses and applies a command, then auto-drains `EnemyTurn` — mirroring the main loop.

### Snapshot tests (`tests/scripts.rs`)

Discovers every `scripts/simple/*.slay` file at test time and runs each through `run_game` with `NoOpRng` and a `Vec<u8>` writer. The full output is compared against a committed snapshot via `insta`.

**Running snapshot tests:**
```
cargo test -p slay-tui --test scripts
```

**After changing TUI output**, accept new snapshots:
```
INSTA_UPDATE=always cargo test -p slay-tui --test scripts
```

**Adding a new scenario:** write a `scripts/simple/<name>.slay` file and run `INSTA_UPDATE=new cargo test -p slay-tui --test scripts` to generate its snapshot. Commit both.

---

## Testing Strategy

| Layer | Location | What it tests |
|---|---|---|
| Damage formula | `status.rs` tests | `resolve_damage` math in isolation |
| Per-card effects | `cards/tests.rs` | What each card does when played |
| Combat engine | `combat.rs` tests | Drawing, energy, phases, block, turn cycle, status tick-down, relic hooks |
| Run/progression | `run.rs` tests | Map transitions, rest, rewards, gold, floor progression, shop |
| Enemy moves | `enemies/mod.rs` tests | `id()`/`from_id()` round-trip, move intent values, no-repeat constraints |
| TUI integration | `slay-tui/tests/integration.rs` | Text commands → `GameState` assertions through the full stack |
| TUI snapshot | `slay-tui/tests/scripts.rs` | Full TUI output from `.slay` scripts, compared via `insta` |
| TUI layout | `slay-tui/src/tui.rs` tests | `TestBackend` renders; assert expected text appears in frame buffer |

All `slay-core` tests use `NoOpRng` directly. Snapshot tests use `AnyRng::NoOp`. No mocks.

**Run snapshot tests after any TUI output change.** They catch regressions in render layout, event descriptions, or state transition messages.

---

## Key Architecture Decisions

### Fixed constraints

- **`GameState` is owned and cloned, never mutated in-place.** `apply_command` takes ownership and returns a new state. Every state transition is a pure function.
- **Card modules cannot see whether they killed the enemy.** Death detection is centralised in the `PlayCard` handler. Card modules only apply their effect. This prevents double-emitting `EnemyDied`.
- **`exhaust_pile` is scoped to one combat.** `player_after_combat` returns exhausted cards to `deck`. For permanent card removal, a separate mechanism is needed.
- **Each card module is the single source of truth for its grade-dependent values.** `def(grade)` and `apply(..., grade, ...)` both live in the same file — the numbers in `apply` and the numbers shown in `CardDescription` cannot drift independently.
- **`MapNode` carries its own enemies.** Each node in the graph owns its `Vec<EnemyKind>`. There is no floor-index-to-enemy lookup function.

### Flexible / designed to change

- **`generate_map`'s 10-floor layout** is fixed in code. To change the map shape, modify the floor loop and convergence points inside `generate_map`.
- **`Card::exhausts()` covers only self-exhaust.** Random or player-choice exhaust requires a different mechanism — likely a return value from `cards::apply`.
- **`Rng` trait only exposes `shuffle`.** Extend with `next_u32` or similar when proc chances or random damage ranges are needed.
- **`Intent` may grow.** Future intents (multi-hit, special) require new variants. Likely to grow as more enemies are added.
- **`resolve_damage` has no relic hooks.** A relic modifying the damage formula would require a context parameter.
- **Shop stock is generated once** at Merchant entry. There's no mechanism to refresh or remove purchased items from display beyond the `bool` sold flag on each item.

# Plan: Split cards/mod.rs

**Branch**: main
**Status**: Active

## Goal

Reduce `cards/mod.rs` from 1484 lines to ~200 lines by:
1. Introducing a `Grade` enum so upgradeable cards have one variant instead of two
2. Pushing per-card data (`def`, `id`) into individual card files
3. Extracting the test module to `cards/tests.rs`

## Grade enum

```rust
pub enum Grade { Base, Plus }
```

Upgradeable cards become `Card::Bash(Grade)` instead of `Card::Bash` + `Card::BashPlus`. Cards without an upgrade stay plain: `Card::Disarm`, `Card::Dazed`.

**Effect on mod.rs dispatch** — one arm per card instead of two:
```rust
// Before
Card::Bash     => bash::def(),
Card::BashPlus => bash::def_plus(),

// After
Card::Bash(g) => bash::def(g),
```

**Effect on call sites** — more explicit:
```rust
// Before
Card::Strike, Card::StrikePlus
// After
Card::Strike(Grade::Base), Card::Strike(Grade::Plus)
```

**Effect on predicates:**
```rust
// exhausts() — cleaner
matches!(self, Card::Disarm | Card::Impervious(_) | Card::Dazed)

// upgrade() — same length but uniform pattern
Card::Strike(Grade::Base) => Some(Card::Strike(Grade::Plus)),
Card::Strike(Grade::Plus) => None,
// or: Card::X(Grade::Plus) | Card::Disarm | Card::Dazed => None (catchall)
```

## Target structure

```
cards/mod.rs      ~200 lines  (Card enum, Grade, types, thin dispatch, upgrade/predicates/pools)
cards/tests.rs    ~820 lines  (all tests, moved verbatim)
cards/bash.rs      ~30 lines  (def(Grade), id(Grade), apply — self-contained)
cards/strike.rs    ~25 lines  (def(Grade), id(Grade), apply)
... (each card file is self-contained)
cards/bludgeon.rs  ~15 lines  (def(Grade), id(Grade) — new file; apply delegates to strike::apply)
cards/impervious.rs ~15 lines (def(Grade), id(Grade) — new file; apply delegates to defend::apply)
cards/dazed.rs     ~10 lines  (def(), id() — new file, no Grade, no apply)
```

## Interface convention

Each card file exposes:

```rust
// bash.rs
pub(super) fn def(grade: Grade) -> CardDef {
    let (name, damage, vuln) = match grade {
        Grade::Base => ("Bash",  8, 2),
        Grade::Plus => ("Bash+", 10, 3),
    };
    CardDef {
        name,
        description: CardDescription::WithDamage {
            template: "Deal {damage} damage. Apply {vuln} Vulnerable.",
            base: damage,
        },
        energy_cost: Energy(2),
        card_type: CardType::Attack,
    }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "bash", Grade::Plus => "bash-plus" }
}

pub fn apply(state, events, damage, vuln, target) { ... }  // unchanged
```

Cards without a + variant (Disarm, Dazed) expose plain `def() -> CardDef` and `id() -> &'static str`.

## Steps

### Step 1: Commit pending work

Commit the map grid branching display + README changes before touching cards/.

### Step 2: Add def/id to all card files

For each of the 24 existing card files, add `def(grade: Grade)` and `id(grade: Grade)`.
Create `bludgeon.rs`, `impervious.rs`, `dazed.rs` with their def/id.

No changes to mod.rs yet — everything still compiles and tests pass.

### Step 3: Introduce Grade + update Card enum and all call sites

- Add `pub enum Grade { Base, Plus }` to mod.rs
- Change `Card::Bash`, `Card::BashPlus` → `Card::Bash(Grade)` for all upgradeable cards
- Update mod.rs dispatch (`def`, `id`, `apply`, `upgrade`, `exhausts`, `from_id`, `starter_deck`, `reward_pool`)
- Update all other call sites: `command.rs`, tests, TUI/game rendering

All tests must pass at the end of this step.

### Step 4: Move tests to tests.rs

Move the `mod tests { ... }` block verbatim to `cards/tests.rs`.
Replace with `#[cfg(test)] mod tests;` in mod.rs.
Run `cargo test` — all tests must still pass.

### Step 5: Verify and commit

`cargo test && cargo clippy` — clean.
Commit: `refactor: introduce Grade enum, push per-card data into card files`

## Done when

- `mod.rs` is ≤ 250 lines
- `Card` enum has ~27 variants (one per card) instead of ~54
- All existing tests pass unchanged
- `cargo clippy` is clean
- Each card file owns its own def, id, and apply

# Card Architecture: Enum vs Trait

## Why not `trait Card` (the Java/Kotlin analogy)?

In Java, `interface Card` works naturally — all objects are heap-allocated anyway, and `equals()`/`clone()` have default mechanisms. In Rust, using `dyn Trait` instead of an enum forces you to opt in to costs that the enum avoids:

- **`Box<dyn Card>` everywhere** — every card in hand/draw/discard is a heap allocation
- **`PartialEq` is lost** — `dyn Card` doesn't support `==`; test assertions like `assert_eq!(discard_pile, vec![Card::Strike])` require `Any` downcasting hacks
- **`Clone` is lost** — must add `fn clone_box(&self) -> Box<dyn Card>` to every impl; can't derive it
- **No exhaustive matching** — the compiler no longer tells you when a new card isn't handled somewhere; silent omissions become runtime bugs
- **Coupling** — `card.apply(&mut CombatState, ...)` requires every card struct to depend on the full combat state type

## Why the closed enum is the right call

The exhaustive match is a **feature**, not a limitation. When you add card 51, the compiler tells you every place that needs updating. That's the guarantee you want.

The enum also gives `PartialEq`, `Clone`, and `Debug` for free, keeps cards as plain values in `Vec<Card>` with no heap indirection, and makes test setup trivial (`vec![Card::Strike, Card::Defend]`).

## How we scale to 50+ cards

The enum stays closed. The two scaling problems are addressed structurally:

1. **Static data** (`name`, `description`, `energy_cost`) lives in `CardDef`, returned from `Card::def()`. One match, all data in one place, easy to scan.

2. **Effect logic** lives in per-card modules (`cards/strike.rs`, `cards/defend.rs`, …), dispatched from `cards::apply()`. Adding a card means adding one file and four lines — enum variant, `CardDef` entry, dispatch arm, the file itself.

The compiler enforces all four touch points on every new card.

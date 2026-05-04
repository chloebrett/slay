# Plan: Shop / Merchant

**Branch**: feat/shop
**Status**: Active

## Goal

Add a Merchant node to the map where the player can spend gold on cards, a relic, and a potion.

## Design

- **Map**: `[Combat, Combat, Combat, Merchant, RestSite, Boss]` — shop after 3 fights, before rest
- **Inventory**: 2 random cards (75g each), 1 random relic (150g), 1 random potion (50g)
- **Prices**: fixed constants — `CARD_PRICE`, `RELIC_PRICE`, `POTION_PRICE`
- **Purchased slots**: marked on `ShopState`; bought items disappear
- **No card removal** (deferred)

## State shape

```rust
// slay-core/src/run.rs
pub struct ShopState {
    pub player: Player,
    pub floor: usize,
    pub cards: Vec<(Card, bool)>,        // (card, purchased)
    pub relic: Option<(Relic, bool)>,
    pub potion: Option<(Potion, bool)>,
}
```

New commands: `BuyCard(usize)`, `BuyRelic`, `BuyPotion`, `LeaveShop`
New error: `CommandError::NotEnoughGold`

## Acceptance Criteria

- [ ] `MAP_NODES` is `[Combat, Combat, Combat, Merchant, RestSite, Boss]`
- [ ] Player starts a new run with 99 gold
- [ ] Entering floor 3 node opens `GameState::Shop` with 2 cards, 1 relic, 1 potion generated from their respective pools
- [ ] `LeaveShop` transitions to `Map { floor: floor + 1 }`
- [ ] `BuyCard(i)` deducts 75g and adds the card to the player's deck; returns `NotEnoughGold` if insufficient; returns `InvalidCard` if index is out of bounds or already purchased
- [ ] `BuyRelic` deducts 150g and grants the relic; returns `NotEnoughGold` if insufficient; returns `InvalidCard` if already purchased or no relic in shop
- [ ] `BuyPotion` deducts 50g and adds the potion; returns `NotEnoughGold` if insufficient; returns `InvalidPhase` if potion slots are full; returns `InvalidCard` if already purchased or no potion in shop
- [ ] Gold persists after leaving the shop
- [ ] TUI renders shop inventory with prices, marks purchased items
- [ ] Plain-text renderer renders shop inventory
- [ ] Shop commands parse correctly: `"1"`/`"2"` → `BuyCard`, `"r"` → `BuyRelic`, `"p"` → `BuyPotion`, `"leave"` → `LeaveShop`

## Steps

Every step follows RED-GREEN-MUTATE-KILL MUTANTS-REFACTOR.

---

### Step 1: State machine — types, transition, generation

**Acceptance criteria**:
- `MapNode::Merchant` exists; `MAP_NODES[2]` is `Merchant`
- `ChooseNode(0)` on floor 2 produces `GameState::Shop` with 2 cards from `reward_pool`, 1 relic from `Relic::all()`, 1 potion from `Potion::all()`
- `LeaveShop` on `GameState::Shop` produces `GameState::Map { floor: floor + 1 }`
- `Command::LeaveShop` exists; `CommandError::NotEnoughGold` exists

**RED**: Tests for:
- floor 2 node is `Merchant`
- `ChooseNode(0)` from map floor 2 → `GameState::Shop`
- `ShopState.cards.len() == 2`, `.relic.is_some()`, `.potion.is_some()`
- `LeaveShop` → `Map { floor: 3 }`

**GREEN**: Add `MapNode::Merchant`, `ShopState`, `GameState::Shop`, `Command::LeaveShop`, `CommandError::NotEnoughGold`. Wire `apply_command`. Add `generate_shop` fn. Update `MAP_NODES`. Add `Relic::all()` if not present, `Potion::all()`.

**MUTATE / KILL / REFACTOR**: Standard cycle.

**Done when**: All criteria met, mutation report reviewed, human approves commit.

---

### Step 2: Buy commands

**Acceptance criteria**:
- `BuyCard(i)` deducts 75g, adds card to deck, marks slot purchased
- `BuyCard(i)` returns `NotEnoughGold` when gold < 75
- `BuyCard(i)` returns `InvalidCard` when index OOB or slot already purchased
- `BuyRelic` deducts 150g, calls `grant_relic`, marks purchased; same error cases
- `BuyPotion` deducts 50g, adds to `player.potions`, marks purchased
- `BuyPotion` returns `InvalidPhase` when `player.potions.len() == MAX_POTIONS`
- Gold persists on `player` after `LeaveShop`

**RED**: Tests for each case above.

**GREEN**: Handle `BuyCard`, `BuyRelic`, `BuyPotion` in `apply_command` for `GameState::Shop`.

**MUTATE / KILL / REFACTOR**: Standard cycle.

**Done when**: All criteria met, mutation report reviewed, human approves commit.

---

### Step 3: TUI — render + parse

**Acceptance criteria**:
- Plain-text renderer prints shop inventory (card names + price, relic name + price, potion name + price); purchased slots show as `[sold]`
- TUI renders the same information in a block widget
- `"1"` / `"2"` → `BuyCard(0)` / `BuyCard(1)` in Shop state
- `"r"` → `BuyRelic`; `"p"` → `BuyPotion`; `"leave"` / `"l"` → `LeaveShop`
- Snapshot test covers a buy-and-leave sequence

**RED**: TUI unit test (TestBackend) asserting card names and prices appear; command parsing tests in `command.rs`.

**GREEN**: Add `render_shop` to `engine.rs` / `tui.rs`; add Shop arms to `command::parse`; add shop script + snapshot.

**MUTATE / KILL / REFACTOR**: Standard cycle.

**Done when**: All criteria met, snapshot committed, mutation report reviewed, human approves commit.

---

## Pre-PR Quality Gate

1. Mutation testing — run `mutation-testing` skill
2. Refactoring assessment — run `refactoring` skill
3. `cargo clippy` clean, `cargo test` all green
4. Snapshot files committed alongside `.slay` scripts

---
*Delete this file when the plan is complete.*

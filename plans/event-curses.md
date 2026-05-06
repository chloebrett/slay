# Plan: Act 1 Curse Events

**Status**: Active

## Goal

Wire up the four Act 1 events that add curse cards to the player's deck: Ssssserpent (Doubt), Big Fish (Regret), Mushrooms (Parasite), and Golden Idol (Injury).

## Acceptance Criteria

- [x] `Command::ChooseEventOption(usize)` is a valid command
- [x] `GameState::EventRoom` exists and events can be entered from `MapNode::Event`
- [x] Ssssserpent: Agree gives 150 gold and adds Doubt to deck; Disagree/Leave do nothing
- [x] Big Fish: Banana heals 30% max HP; Donut gives +3 max HP; Box gives a random relic and adds Regret to deck; Leave does nothing
- [x] Mushrooms: Eat heals 12 HP and adds Parasite to deck; Leave does nothing (Stomp deferred — needs mid-event combat)
- [x] Golden Idol: Outrun adds Injury to deck; Smash deals 25 damage; Hide costs 6 max HP; Leave does nothing. (No relic reward yet — Golden Idol relic deferred.)
- [x] All options return the player to `GameState::Map`
- [x] `Event::CardAdded { card }` emitted whenever a curse is added

## Notes

- **Mushrooms Stomp** (triggers combat mid-event) is deferred — needs a different state machine. Only Eat and Leave are implemented.
- **Golden Idol relic** not yet implemented. All non-Leave options award 250 gold as a temporary stand-in. Swap for the actual relic when it's added.
- Golden Idol is simplified to a single-step choice (escape route presented directly, no intermediate "Take or Leave" step) to avoid multi-step sub-state.

## Steps

Every step follows RED-GREEN-MUTATE-KILL MUTANTS-REFACTOR.

---

### Step 1: Event infrastructure + Ssssserpent

**Acceptance criteria**: `GameState::EventRoom(EventRoomState)` exists. Applying `ChooseEventOption(0)` on a Ssssserpent event adds 150 gold and Doubt to the deck. Options 1 and 2 do nothing. All options return to `GameState::Map`.

**RED**: Tests that a Ssssserpent event room transitions correctly on each option.  
**GREEN**: Add `MapNode::Event`, `EventKind`, `EventRoomState`, `GameState::EventRoom`, `Command::ChooseEventOption`, and Ssssserpent handling.  
**MUTATE**: Run `cargo mutants` on the new event code.  
**KILL MUTANTS**: Address survivors.  
**REFACTOR**: Assess.  
**Done when**: All criteria met, mutation report reviewed.

---

### Step 2: Big Fish

**Acceptance criteria**: Banana heals 30% max HP (capped at max, at least 1). Donut increases both max HP and current HP by 3. Box gives a random relic and adds Regret. Leave does nothing. All return to Map.

**RED**: Tests for each option (Banana heal, Donut max HP, Box relic + Regret, Leave).  
**GREEN**: Add `BigFish` to `EventKind`, handle each option.  
**MUTATE/KILL MUTANTS/REFACTOR**: Standard.  
**Done when**: All criteria met, mutation report reviewed.

---

### Step 3: Mushrooms

**Acceptance criteria**: Eat heals 12 HP (capped at max) and adds Parasite to deck. Leave does nothing. Both return to Map.

**RED**: Tests for Eat (heal + Parasite) and Leave.  
**GREEN**: Add `Mushrooms` to `EventKind`.  
**MUTATE/KILL MUTANTS/REFACTOR**: Standard.  
**Done when**: All criteria met, mutation report reviewed.

---

### Step 4: Golden Idol

**Acceptance criteria**: Outrun adds Injury to deck and gives 250 gold. Smash deals 25 damage and gives 250 gold. Hide costs 6 max HP and gives 250 gold. Leave does nothing. All return to Map.

**RED**: Tests for each escape option and Leave.  
**GREEN**: Add `GoldenIdol` to `EventKind`.  
**MUTATE/KILL MUTANTS/REFACTOR**: Standard.  
**Done when**: All criteria met, mutation report reviewed.

---

## Pre-PR Quality Gate

1. `cargo test` passes
2. `cargo clippy` clean
3. Mutation testing run on event handler code

---
*Delete this file when the plan is complete.*

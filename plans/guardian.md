# Plan: The Guardian (Act 1 Boss)

**Status**: Active

## Reference

https://slay-the-spire.fandom.com/wiki/The_Guardian

## Stats

- **HP**: 240
- **Name**: "The Guardian"

## Moves

| Move | Effects |
|------|---------|
| Charging Up | Gains 9 Block |
| Fierce Bash | Deal 32 damage |
| Vent Steam | Apply 2 Weak + 2 Vulnerable to player |
| Whirlwind | Deal 5 damage × 4 |
| Roll Attack | Deal 9 damage |
| Twin Slam | Deal 8 damage × 2, then clear all Sharp Hide stacks |

## Mode Mechanic

**Offensive Mode** (default cycle, repeats):
`None → ChargingUp → FierceBash → VentSteam → Whirlwind → ChargingUp → …`

**Mode Shift** (triggered mid-cycle when cumulative damage ≥ threshold):
- Threshold: 30 on first shift, increases by 10 each subsequent shift (30 → 40 → 50 → …)
- On trigger: immediately gain 20 Block + 3 Sharp Hide stacks; `move_` forced to RollAttack; mode flips to Defensive
- ModeShiftProgress resets to 0; ModeShiftCount increments

**Defensive Mode** (runs once, then returns to Offensive):
`RollAttack → TwinSlam → Whirlwind → ChargingUp`

(Both paths end with Whirlwind → ChargingUp, so `next_move` transitions are the same regardless of mode.)

## Mode State

Stored as `StatusEffect::GuardianMode` on the enemy's `StatusMap`:
- Value `0` = Offensive
- Value `1` = Defensive

This is the canonical mode enum. Mode shifts only trigger when in Offensive Mode (`GuardianMode == 0`).

## Sharp Hide

- Stored as `StatusEffect::SharpHide` on the enemy
- While the Guardian has Sharp Hide stacks, each **Attack card** the player plays deals `stacks × 5` damage **to the player, once per card** (Twin Strike = one instance, not two)
- The damage is **blockable** by the player's block
- Twin Slam removes all stacks via `Effect::ClearSelfStatus(StatusEffect::SharpHide)`

## Mode Shift Tracking

Stored on the enemy's `statuses` map:
- `StatusEffect::ModeShiftProgress` — accumulated HP loss since last shift. **Visible to player** (displayed in TUI as e.g. "Mode Shift: 18/30"). The display threshold is derived as `30 + ModeShiftCount × 10`.
- `StatusEffect::ModeShiftCount` — how many shifts have occurred (invisible). Threshold = `30 + count × 10`.

## Effect Extension

- Add `Effect::ClearSelfStatus(StatusEffect)` — removes all stacks of a status from self when the move is executed.

---

## Acceptance Criteria

- [ ] `EnemyKind::TheGuardian` exists with HP 240
- [ ] Offensive move sequence: ChargingUp → FierceBash → VentSteam → Whirlwind → ChargingUp (repeats)
- [ ] Sharp Hide: player takes `stacks × 5` damage (once per Attack card, blockable) while Guardian has SharpHide stacks
- [ ] Mode shift triggers at 30 cumulative HP loss (in Offensive Mode only); Guardian gains 20 Block + 3 Sharp Hide; move_ becomes RollAttack; GuardianMode → Defensive
- [ ] Defensive move sequence: RollAttack → TwinSlam → Whirlwind → ChargingUp; GuardianMode → Offensive after Whirlwind
- [ ] TwinSlam clears all Sharp Hide stacks via ClearSelfStatus
- [ ] Second mode shift threshold is 40 damage
- [ ] ModeShiftProgress resets to 0 on mode shift
- [ ] Boss floor uses TheGuardian
- [ ] `Effect::ClearSelfStatus(StatusEffect)` variant exists

---

## Steps

### Step 1: Offensive Guardian

**Acceptance criteria**: `EnemyKind::TheGuardian` has HP 240. Offensive move sequence is correct: `None → ChargingUp → FierceBash → VentSteam → Whirlwind → ChargingUp`. All move effects correct (see table). Boss floor uses TheGuardian.

**RED**: Tests for HP, name, each move def (name + effects), and the full offensive cycle.
**GREEN**: Add `TheGuardian` to `EnemyKind`, six `Move` variants + `MoveDef`s, `next_move` for offensive cycle, `EnemyDef`. Add `Effect::ClearSelfStatus`. Update boss floor node.
**MUTATE / KILL / REFACTOR**: Standard.

---

### Step 2: Sharp Hide + mode shift

**Acceptance criteria**: (see full acceptance criteria above — everything not in step 1)

**RED**: Tests for sharp hide player damage (once per card, blockable), mode shift trigger at 30 damage, mode shift effects, defensive sequence, threshold escalation to 40, TwinSlam clears sharp hide.
**GREEN**: Add `StatusEffect::SharpHide`, `GuardianMode`, `ModeShiftProgress`, `ModeShiftCount`. Handle SharpHide in PlayCard (blockable, once per card). Handle mode shift in damage-to-enemy code. Handle `ClearSelfStatus` in move execution. Handle Whirlwind → ChargingUp + mode reset in `next_move`.
**MUTATE / KILL / REFACTOR**: Standard.

---

*Delete when complete.*

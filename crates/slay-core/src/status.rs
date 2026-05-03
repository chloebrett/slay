use indexmap::IndexMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusEffect {
    Vulnerable,
    Weak,
    Poison,
    Strength,
}

impl StatusEffect {
    fn ticks_at_end_of_turn(self) -> bool {
        matches!(self, StatusEffect::Vulnerable | StatusEffect::Weak)
    }
}

pub type StatusMap = IndexMap<StatusEffect, i32>;

pub fn resolve_damage(base: i32, attacker: &StatusMap, defender: &StatusMap) -> i32 {
    let dmg = base + attacker.get(&StatusEffect::Strength).copied().unwrap_or(0);
    let dmg = if attacker.contains_key(&StatusEffect::Weak) { dmg * 3 / 4 } else { dmg };
    let dmg = if defender.contains_key(&StatusEffect::Vulnerable) { dmg * 3 / 2 } else { dmg };
    dmg.max(0)
}

pub fn tick_statuses(statuses: &mut StatusMap) {
    statuses.retain(|&status, stacks| {
        if !status.ticks_at_end_of_turn() {
            return true;
        }
        *stacks -= 1;
        *stacks > 0
    });
}

/// Drains poison: returns damage dealt and decrements the stack.
/// Returns 0 if no poison. Caller applies the damage to HP.
pub fn drain_poison(statuses: &mut StatusMap) -> i32 {
    let damage = statuses.get(&StatusEffect::Poison).copied().unwrap_or(0);
    if damage == 0 {
        return 0;
    }
    if damage == 1 {
        statuses.remove(&StatusEffect::Poison);
    } else {
        *statuses.get_mut(&StatusEffect::Poison).unwrap() -= 1;
    }
    damage
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty() -> StatusMap { StatusMap::new() }

    fn map_with(effect: StatusEffect, stacks: i32) -> StatusMap {
        let mut m = StatusMap::new();
        m.insert(effect, stacks);
        m
    }

    // --- resolve_damage ---

    #[test]
    fn vulnerable_multiplies_damage_by_3_over_2() {
        let defender = map_with(StatusEffect::Vulnerable, 2);
        assert_eq!(resolve_damage(6, &empty(), &defender), 9); // 6 * 3/2
    }

    #[test]
    fn weak_multiplies_damage_by_3_over_4() {
        let attacker = map_with(StatusEffect::Weak, 2);
        assert_eq!(resolve_damage(8, &attacker, &empty()), 6); // 8 * 3/4
    }

    #[test]
    fn strength_adds_flat_bonus_to_damage() {
        let attacker = map_with(StatusEffect::Strength, 2);
        assert_eq!(resolve_damage(6, &attacker, &empty()), 8); // 6 + 2
    }

    #[test]
    fn strength_applies_before_vulnerable_multiplier() {
        let attacker = map_with(StatusEffect::Strength, 2);
        let defender = map_with(StatusEffect::Vulnerable, 2);
        assert_eq!(resolve_damage(6, &attacker, &defender), 12); // (6 + 2) * 3/2
    }
}

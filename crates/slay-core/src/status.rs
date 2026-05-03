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

/// Triggers poison on an entity: returns damage dealt and decrements the stack.
/// Returns 0 if no poison. Caller is responsible for applying damage to HP.
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

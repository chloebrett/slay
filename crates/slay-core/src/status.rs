use indexmap::IndexMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusEffect {
    Vulnerable,
    Weak,
}

pub type StatusMap = IndexMap<StatusEffect, i32>;

pub fn resolve_damage(base: i32, attacker: &StatusMap, defender: &StatusMap) -> i32 {
    let dmg = base;
    let dmg = if attacker.contains_key(&StatusEffect::Weak) { dmg * 3 / 4 } else { dmg };
    let dmg = if defender.contains_key(&StatusEffect::Vulnerable) { dmg * 3 / 2 } else { dmg };
    dmg.max(0)
}

pub fn tick_statuses(statuses: &mut StatusMap) {
    statuses.retain(|_, stacks| {
        *stacks -= 1;
        *stacks > 0
    });
}

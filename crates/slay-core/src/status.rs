use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusEffect {
    Vulnerable,
    Weak,
}

pub fn resolve_damage(
    base: i32,
    attacker_statuses: &HashMap<StatusEffect, i32>,
    defender_statuses: &HashMap<StatusEffect, i32>,
) -> i32 {
    let dmg = base;
    let dmg = if attacker_statuses.contains_key(&StatusEffect::Weak) { dmg * 3 / 4 } else { dmg };
    let dmg = if defender_statuses.contains_key(&StatusEffect::Vulnerable) { dmg * 3 / 2 } else { dmg };
    dmg.max(0)
}

pub fn tick_statuses(statuses: &mut HashMap<StatusEffect, i32>) {
    statuses.retain(|_, stacks| {
        *stacks -= 1;
        *stacks > 0
    });
}

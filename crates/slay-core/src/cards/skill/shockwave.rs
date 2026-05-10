use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, apply_enemy_debuff};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, _target: usize) {
    let stacks = GradeValues { base: 3, plus: 5 }.get(grade);
    for i in 0..state.enemies.len() {
        apply_enemy_debuff(state, i, StatusEffect::Weak, stacks, events);
        apply_enemy_debuff(state, i, StatusEffect::Vulnerable, stacks, events);
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Shockwave",  "Apply 3 Weak and Vulnerable to ALL enemies. Exhaust."),
        Grade::Plus => ("Shockwave+", "Apply 5 Weak and Vulnerable to ALL enemies. Exhaust."),
    };
    CardDef { name, description: CardDescription::Static(desc), energy_cost: Energy(2), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "shockwave", Grade::Plus => "shockwave-plus" }
}

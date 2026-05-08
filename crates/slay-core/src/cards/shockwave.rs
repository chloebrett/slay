use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, apply_status, Target};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, _target: usize) {
    let stacks = match grade { Grade::Base => 3, Grade::Plus => 5 };
    for i in 0..state.enemies.len() {
        apply_status(&mut state.enemies[i].statuses, Target::Enemy, StatusEffect::Weak, stacks, events);
        apply_status(&mut state.enemies[i].statuses, Target::Enemy, StatusEffect::Vulnerable, stacks, events);
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

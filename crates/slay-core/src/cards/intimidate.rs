use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, apply_enemy_debuff};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, _target: usize) {
    let stacks = match grade { Grade::Base => 1, Grade::Plus => 2 };
    for i in 0..state.enemies.len() {
        apply_enemy_debuff(state, i, StatusEffect::Weak, stacks, events);
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Intimidate",  "Apply 1 Weak to ALL enemies. Exhaust."),
        Grade::Plus => ("Intimidate+", "Apply 2 Weak to ALL enemies. Exhaust."),
    };
    CardDef { name, description: CardDescription::Static(desc), energy_cost: Energy(0), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "intimidate", Grade::Plus => "intimidate-plus" }
}

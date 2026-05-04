use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, Target, apply_status};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize) {
    let amount = match grade { Grade::Base => 3, Grade::Plus => 4 };
    apply_status(&mut state.enemies[target].statuses, Target::Enemy, StatusEffect::Vulnerable, amount, events);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Tremble",  CardDescription::Static("Apply 3 Vulnerable.")),
        Grade::Plus => ("Tremble+", CardDescription::Static("Apply 4 Vulnerable.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(1), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "tremble", Grade::Plus => "tremble-plus" }
}

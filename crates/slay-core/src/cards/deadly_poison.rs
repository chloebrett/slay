use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, Target, apply_status};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, poison: i32, target: usize) {
    apply_status(&mut state.enemies[target].statuses, Target::Enemy, StatusEffect::Poison, poison, events);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Deadly Poison",  CardDescription::Static("Apply 5 Poison.")),
        Grade::Plus => ("Deadly Poison+", CardDescription::Static("Apply 7 Poison.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(1), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "deadly-poison", Grade::Plus => "deadly-poison-plus" }
}

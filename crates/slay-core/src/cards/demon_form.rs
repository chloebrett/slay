use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, Target, apply_status};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, _target: usize) {
    let strength = GradeValues { base: 2, plus: 3 }.get(grade);
    apply_status(&mut state.player.statuses, Target::Player, StatusEffect::DemonForm, strength, events);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Demon Form",  CardDescription::Static("At the start of your turn, gain 2 Strength.")),
        Grade::Plus => ("Demon Form+", CardDescription::Static("At the start of your turn, gain 3 Strength.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(3), card_type: CardType::Power }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "demon-form", Grade::Plus => "demon-form-plus" }
}

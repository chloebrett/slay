use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event};
use crate::types::Energy;

pub fn apply(state: &mut CombatState, _events: &mut Vec<Event>, grade: Grade) {
    state.hand_cost_max = Some(Energy(1));
    state.hand_cost_max_expires = matches!(grade, Grade::Base);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Enlightenment",  CardDescription::Static("Reduce the cost of all cards in your hand to 1 this turn.")),
        Grade::Plus => ("Enlightenment+", CardDescription::Static("Reduce the cost of all cards in your hand to 1 this combat.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(0), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "enlightenment", Grade::Plus => "enlightenment-plus" }
}

use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event};
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, _grade: Grade) {
    state.player.energy.0 += 2;
    events.push(Event::EnergyGained { amount: 2 });
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, cost) = match grade {
        Grade::Base => ("Seeing Red",  Energy(1)),
        Grade::Plus => ("Seeing Red+", Energy(0)),
    };
    CardDef { name, description: CardDescription::Static("Gain [R][R]. Exhaust."), energy_cost: cost, card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "seeing-red", Grade::Plus => "seeing-red-plus" }
}

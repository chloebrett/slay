use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, Target, apply_status};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, _grade: Grade, _target: usize) {
    apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Barricade, 1, events);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, cost) = match grade {
        Grade::Base => ("Barricade",  Energy(3)),
        Grade::Plus => ("Barricade+", Energy(2)),
    };
    CardDef { name, description: CardDescription::Static("Block is not removed at the start of your turn."), energy_cost: cost, card_type: CardType::Power }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "barricade", Grade::Plus => "barricade-plus" }
}

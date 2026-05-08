use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, Target, apply_status};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, _grade: Grade, _target: usize) {
    apply_status(&mut state.player.statuses, Target::Player, StatusEffect::DarkEmbrace, 1, events);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, cost) = match grade {
        Grade::Base => ("Dark Embrace",  Energy(2)),
        Grade::Plus => ("Dark Embrace+", Energy(1)),
    };
    CardDef { name, description: CardDescription::Static("Whenever a card is Exhausted, draw 1 card."), energy_cost: cost, card_type: CardType::Power }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "dark-embrace", Grade::Plus => "dark-embrace-plus" }
}

use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, apply_status, Target};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, _grade: Grade, _target: usize) {
    apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Brutality, 1, events);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let name = match grade { Grade::Base => "Brutality", Grade::Plus => "Brutality+" };
    CardDef {
        name,
        description: CardDescription::Static("At the start of your turn, lose 1 HP and draw 1 card."),
        energy_cost: Energy(0),
        card_type: CardType::Power,
    }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "brutality", Grade::Plus => "brutality-plus" }
}

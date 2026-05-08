use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, apply_status, Target};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, _target: usize) {
    let amount = GradeValues { base: 1, plus: 2 }.get(grade);
    apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Evolve, amount, events);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Evolve",  "Whenever you draw a Status card, draw 1 card."),
        Grade::Plus => ("Evolve+", "Whenever you draw a Status card, draw 2 cards."),
    };
    CardDef { name, description: CardDescription::Static(desc), energy_cost: Energy(1), card_type: CardType::Power }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "evolve", Grade::Plus => "evolve-plus" }
}

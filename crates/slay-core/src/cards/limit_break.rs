use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, apply_status, Target};
use crate::status::{StatusEffect, get_stacks};
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, _grade: Grade, _target: usize) {
    let strength = get_stacks(&state.player.statuses, StatusEffect::Strength);
    if strength > 0 {
        apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Strength, strength, events);
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Limit Break",  "Double your Strength. Exhaust."),
        Grade::Plus => ("Limit Break+", "Double your Strength."),
    };
    CardDef { name, description: CardDescription::Static(desc), energy_cost: Energy(1), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "limit-break", Grade::Plus => "limit-break-plus" }
}

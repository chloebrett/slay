use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, Target, apply_status};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, strength: i32, _target: usize) {
    apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Strength, strength, events);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Inflame",  CardDescription::Static("Gain 2 Strength.")),
        Grade::Plus => ("Inflame+", CardDescription::Static("Gain 3 Strength.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(1), card_type: CardType::Power }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "inflame", Grade::Plus => "inflame-plus" }
}

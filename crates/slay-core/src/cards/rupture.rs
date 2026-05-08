use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, Target, apply_status};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, _target: usize) {
    let strength = match grade { Grade::Base => 1, Grade::Plus => 2 };
    apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Rupture, strength, events);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Rupture",  CardDescription::Static("Whenever you lose HP on your turn, gain 1 Strength.")),
        Grade::Plus => ("Rupture+", CardDescription::Static("Whenever you lose HP on your turn, gain 2 Strength.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(1), card_type: CardType::Power }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "rupture", Grade::Plus => "rupture-plus" }
}

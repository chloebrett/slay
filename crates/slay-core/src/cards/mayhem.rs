use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, apply_status};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, _grade: Grade) {
    apply_status(&mut state.player.statuses, crate::combat::Target::Player, StatusEffect::Mayhem, 1, events);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Mayhem",  CardDescription::Static("At the start of your turn, play the top card of your draw pile.")),
        Grade::Plus => ("Mayhem+", CardDescription::Static("At the start of your turn, play the top card of your draw pile.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(2), card_type: CardType::Power }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "mayhem", Grade::Plus => "mayhem-plus" }
}

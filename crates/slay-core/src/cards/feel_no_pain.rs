use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, Target, apply_status};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, _target: usize) {
    let block = match grade { Grade::Base => 3, Grade::Plus => 4 };
    apply_status(&mut state.player.statuses, Target::Player, StatusEffect::FeelNoPain, block, events);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Feel No Pain",  CardDescription::Static("Whenever a card is Exhausted, gain 3 Block.")),
        Grade::Plus => ("Feel No Pain+", CardDescription::Static("Whenever a card is Exhausted, gain 4 Block.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(1), card_type: CardType::Power }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "feel-no-pain", Grade::Plus => "feel-no-pain-plus" }
}

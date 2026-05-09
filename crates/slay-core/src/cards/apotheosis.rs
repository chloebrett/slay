use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event};
use crate::types::Energy;

pub fn apply(state: &mut CombatState, _events: &mut Vec<Event>, _grade: Grade) {
    for card in state.player.hand.iter_mut() {
        if let Some(upgraded) = card.upgrade() {
            *card = upgraded;
        }
    }
    for card in state.player.discard_pile.iter_mut() {
        if let Some(upgraded) = card.upgrade() {
            *card = upgraded;
        }
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let name = match grade { Grade::Base => "Apotheosis", Grade::Plus => "Apotheosis+" };
    CardDef { name, description: CardDescription::Static("Upgrade ALL your cards for the rest of combat. Exhaust."), energy_cost: Energy(2), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "apotheosis", Grade::Plus => "apotheosis-plus" }
}

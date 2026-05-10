use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event};
use crate::types::Energy;

pub fn apply(state: &mut CombatState, _events: &mut Vec<Event>, _grade: Grade) {
    if let Some(idx) = state.player.draw_pile.iter().rposition(|c| c.card_type() == CardType::Skill) {
        let card = state.player.draw_pile.remove(idx);
        state.player.hand.push(card);
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Secret Technique",  CardDescription::Static("Put a Skill from your draw pile into your hand. Exhaust.")),
        Grade::Plus => ("Secret Technique+", CardDescription::Static("Put a Skill from your draw pile into your hand.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(0), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "secret-technique", Grade::Plus => "secret-technique-plus" }
}

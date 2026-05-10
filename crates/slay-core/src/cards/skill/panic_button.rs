use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, gain_player_block};
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, _grade: Grade, rng: &mut impl crate::rng::Rng) {
    gain_player_block(state, events, 30, rng);
    state.block_locked_turns = 2;
}

pub(super) fn def(grade: Grade) -> CardDef {
    let name = match grade { Grade::Base => "Panic Button", Grade::Plus => "Panic Button+" };
    CardDef { name, description: CardDescription::Static("Gain 30 Block. You cannot gain Block for 2 turns. Exhaust."), energy_cost: Energy(0), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "panic-button", Grade::Plus => "panic-button-plus" }
}

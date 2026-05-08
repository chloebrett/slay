use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, exhaust_card, gain_player_block};
use crate::rng::Rng;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, rng: &mut impl Rng) {
    let block = match grade { Grade::Base => 7, Grade::Plus => 9 };
    gain_player_block(state, events, block, rng);
    if !state.player.hand.is_empty() {
        let mut indices: Vec<usize> = (0..state.player.hand.len()).collect();
        rng.shuffle(&mut indices);
        let card = state.player.hand.remove(indices[0]);
        exhaust_card(card, state, events, rng);
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("True Grit",  CardDescription::Static("Gain 7 Block. Exhaust 1 card at random.")),
        Grade::Plus => ("True Grit+", CardDescription::Static("Gain 9 Block. Exhaust 1 card at random.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(1), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "true-grit", Grade::Plus => "true-grit-plus" }
}

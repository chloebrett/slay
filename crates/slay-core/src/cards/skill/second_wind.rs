use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, exhaust_card, gain_player_block};
use crate::rng::Rng;
use crate::status::resolve_block;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, rng: &mut impl Rng) {
    let block_per_card = GradeValues { base: 5, plus: 7 }.get(grade);
    let non_attacks: Vec<_> = state.player.hand.iter()
        .filter(|c| c.card_type() != CardType::Attack)
        .cloned()
        .collect();
    state.player.hand.retain(|c| c.card_type() == CardType::Attack);
    let count = non_attacks.len() as i32;
    for card in non_attacks {
        exhaust_card(card, state, events, rng);
    }
    if count > 0 {
        let block = resolve_block(block_per_card * count, &state.player.statuses);
        gain_player_block(state, events, block, rng);
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, description) = match grade {
        Grade::Base => ("Second Wind",  CardDescription::Static("Exhaust all non-Attack cards in your hand. Gain 5 Block for each card Exhausted.")),
        Grade::Plus => ("Second Wind+", CardDescription::Static("Exhaust all non-Attack cards in your hand. Gain 7 Block for each card Exhausted.")),
    };
    CardDef { name, description, energy_cost: Energy(1), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "second-wind", Grade::Plus => "second-wind-plus" }
}

use super::{Card, CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, gain_player_block};
use crate::rng::Rng;
use crate::status::resolve_block;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, _target: usize, rng: &mut impl Rng) {
    for _ in 0..2 {
        state.player.hand.push(Card::Wound);
        events.push(Event::StatusCardAddedToHand { card: Card::Wound });
    }
    let block = GradeValues { base: 15, plus: 20 }.get(grade);
    let actual = resolve_block(block, &state.player.statuses);
    gain_player_block(state, events, actual, rng);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, description) = match grade {
        Grade::Base => ("Power Through",  CardDescription::Static("Add 2 Wounds into your hand. Gain 15 Block.")),
        Grade::Plus => ("Power Through+", CardDescription::Static("Add 2 Wounds into your hand. Gain 20 Block.")),
    };
    CardDef { name, description, energy_cost: Energy(1), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "power-through", Grade::Plus => "power-through-plus" }
}

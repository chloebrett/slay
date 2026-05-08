use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, draw_cards, gain_player_block};
use crate::rng::Rng;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, rng: &mut impl Rng) {
    let block = match grade { Grade::Base => 8, Grade::Plus => 11 };
    gain_player_block(state, events, block, rng);
    draw_cards(&mut state.player, 1, rng);
    events.push(Event::CardsDrawn { count: 1 });
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Shrug It Off",  CardDescription::Static("Gain 8 Block. Draw 1 card.")),
        Grade::Plus => ("Shrug It Off+", CardDescription::Static("Gain 11 Block. Draw 1 card.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(1), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "shrug-it-off", Grade::Plus => "shrug-it-off-plus" }
}

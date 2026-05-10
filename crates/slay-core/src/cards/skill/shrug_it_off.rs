use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, draw_cards, gain_player_block};
use crate::rng::Rng;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, rng: &mut impl Rng) {
    let block = GradeValues { base: 8, plus: 11 }.get(grade);
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

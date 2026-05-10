use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, gain_player_block};
use crate::rng::Rng;
use crate::status::resolve_block;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, rng: &mut impl Rng) {
    let block = GradeValues { base: 5, plus: 8 }.get(grade);
    let actual = resolve_block(block, &state.player.statuses);
    gain_player_block(state, events, actual, rng);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, description) = match grade {
        Grade::Base => ("Sentinel",  CardDescription::Static("Gain 5 Block. If this card is Exhausted, gain 2 Energy.")),
        Grade::Plus => ("Sentinel+", CardDescription::Static("Gain 8 Block. If this card is Exhausted, gain 3 Energy.")),
    };
    CardDef { name, description, energy_cost: Energy(1), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "sentinel", Grade::Plus => "sentinel-plus" }
}

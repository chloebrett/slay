use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, gain_player_block};
use crate::rng::Rng;
use crate::types::Energy;

pub(super) fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, _target: usize, rng: &mut impl Rng) {
    let block_amount = match grade { Grade::Base => 30, Grade::Plus => 40 };
    let actual = crate::status::resolve_block(block_amount, &state.player.statuses);
    gain_player_block(state, events, actual, rng);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Impervious",  CardDescription::Static("Gain 30 Block. Exhaust.")),
        Grade::Plus => ("Impervious+", CardDescription::Static("Gain 40 Block. Exhaust.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(2), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "impervious", Grade::Plus => "impervious-plus" }
}

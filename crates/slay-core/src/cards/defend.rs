use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, gain_player_block};
use crate::rng::Rng;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, _target: usize, rng: &mut impl Rng) {
    let block_amount = match grade { Grade::Base => 5, Grade::Plus => 8 };
    let actual = crate::status::resolve_block(block_amount, &state.player.statuses);
    gain_player_block(state, events, actual, rng);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Defend",  CardDescription::Static("Gain 5 block.")),
        Grade::Plus => ("Defend+", CardDescription::Static("Gain 8 block.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(1), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "defend", Grade::Plus => "defend-plus" }
}

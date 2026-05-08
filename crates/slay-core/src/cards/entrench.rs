use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, gain_player_block};
use crate::rng::Rng;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, _grade: Grade, rng: &mut impl Rng) {
    let gained = state.player.block.0;
    gain_player_block(state, events, gained, rng);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, cost) = match grade {
        Grade::Base => ("Entrench",  Energy(2)),
        Grade::Plus => ("Entrench+", Energy(1)),
    };
    CardDef { name, description: CardDescription::Static("Double your Block."), energy_cost: cost, card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "entrench", Grade::Plus => "entrench-plus" }
}

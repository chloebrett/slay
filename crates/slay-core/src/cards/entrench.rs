use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event};
use crate::types::{Block, Energy};

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, _grade: Grade) {
    let new_block = state.player.block.0 * 2;
    let gained = new_block - state.player.block.0;
    state.player.block = Block(new_block);
    events.push(Event::PlayerBlocked { amount: gained });
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

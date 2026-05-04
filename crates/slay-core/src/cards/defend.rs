use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event};
use crate::types::{Block, Energy};

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, block_amount: i32, _target: usize) {
    let actual = crate::status::resolve_block(block_amount, &state.player.statuses);
    state.player.block = Block(state.player.block.0 + actual);
    events.push(Event::PlayerBlocked { amount: actual });
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

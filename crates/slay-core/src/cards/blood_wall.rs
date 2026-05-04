use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, damage_player};
use crate::types::{Block, Energy};

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade) {
    let block = match grade { Grade::Base => 16, Grade::Plus => 20 };
    damage_player(state, events, 2);
    state.player.block = Block(state.player.block.0 + block);
    events.push(Event::PlayerBlocked { amount: block });
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Blood Wall",  CardDescription::Static("Lose 2 HP. Gain 16 Block.")),
        Grade::Plus => ("Blood Wall+", CardDescription::Static("Lose 2 HP. Gain 20 Block.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(2), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "blood-wall", Grade::Plus => "blood-wall-plus" }
}

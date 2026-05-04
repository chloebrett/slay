use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, Target, apply_status};
use crate::status::StatusEffect;
use crate::types::{Block, Energy};

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, block: i32, vuln: i32, target: usize) {
    state.player.block = Block(state.player.block.0 + block);
    events.push(Event::PlayerBlocked { amount: block });
    apply_status(&mut state.enemies[target].statuses, Target::Enemy, StatusEffect::Vulnerable, vuln, events);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Taunt",  CardDescription::Static("Gain 7 Block. Apply 1 Vulnerable.")),
        Grade::Plus => ("Taunt+", CardDescription::Static("Gain 8 Block. Apply 2 Vulnerable.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(1), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "taunt", Grade::Plus => "taunt-plus" }
}

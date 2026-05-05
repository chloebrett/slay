use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, Target, apply_status};
use crate::enemies::Intent;
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize) {
    let amount = match grade { Grade::Base => 3, Grade::Plus => 4 };
    let intent = state.enemies[target].effective_intent(&state.player.statuses);
    if matches!(intent, Intent::Attack(_) | Intent::AttackDefend(_, _)) {
        apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Strength, amount, events);
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Spot Weakness",  CardDescription::Static("If the enemy intends to attack, gain 3 Strength.")),
        Grade::Plus => ("Spot Weakness+", CardDescription::Static("If the enemy intends to attack, gain 4 Strength.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(1), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "spot-weakness", Grade::Plus => "spot-weakness-plus" }
}

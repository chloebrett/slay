use super::{CardDef, CardDescription, CardType};
use crate::combat::{CombatState, Event, Target, apply_status};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, target: usize) {
    apply_status(&mut state.enemies[target].statuses, Target::Enemy, StatusEffect::Strength, -2, events);
}

pub(super) fn def() -> CardDef {
    CardDef { name: "Disarm", description: CardDescription::Static("Enemy loses 2 Strength. Exhaust."), energy_cost: Energy(1), card_type: CardType::Skill }
}

pub(super) fn id() -> &'static str { "disarm" }

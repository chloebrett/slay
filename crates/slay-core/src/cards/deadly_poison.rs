use crate::combat::{CombatState, Event, Target, apply_status};
use crate::status::StatusEffect;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, poison: i32) {
    apply_status(&mut state.enemy.statuses, Target::Enemy, StatusEffect::Poison, poison, events);
}

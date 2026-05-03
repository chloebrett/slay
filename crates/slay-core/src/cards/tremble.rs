use crate::combat::{CombatState, Event, Target, apply_status};
use crate::status::StatusEffect;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, amount: i32, target: usize) {
    apply_status(&mut state.enemies[target].statuses, Target::Enemy, StatusEffect::Vulnerable, amount, events);
}

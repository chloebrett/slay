use crate::combat::{CombatState, Event, Target};
use crate::status::StatusEffect;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>) {
    *state.enemy.statuses.entry(StatusEffect::Poison).or_insert(0) += 5;
    events.push(Event::StatusApplied { target: Target::Enemy, status: StatusEffect::Poison, stacks: 5 });
}

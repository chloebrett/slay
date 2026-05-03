use crate::combat::{CombatState, Event, Target};
use crate::status::StatusEffect;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>) {
    *state.player.statuses.entry(StatusEffect::Strength).or_insert(0) += 2;
    events.push(Event::StatusApplied { target: Target::Player, status: StatusEffect::Strength, stacks: 2 });
}

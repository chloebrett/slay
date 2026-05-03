use crate::combat::{CombatState, Event, Target, apply_status};
use crate::status::StatusEffect;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>) {
    apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Strength, 2, events);
}

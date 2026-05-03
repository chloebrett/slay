use crate::combat::{CombatState, Event, Target, apply_status};
use crate::status::StatusEffect;
use crate::types::Block;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, block: i32, vuln: i32, target: usize) {
    state.player.block = Block(state.player.block.0 + block);
    events.push(Event::PlayerBlocked { amount: block });
    apply_status(&mut state.enemies[target].statuses, Target::Enemy, StatusEffect::Vulnerable, vuln, events);
}

use crate::combat::{CombatState, Event};
use crate::types::Block;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, block_amount: i32) {
    state.player.block = Block(state.player.block.0 + block_amount);
    events.push(Event::PlayerBlocked { amount: block_amount });
}

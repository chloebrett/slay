use crate::combat::{CombatState, Event};
use crate::types::Block;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, block_amount: i32, _target: usize) {
    let actual = crate::status::resolve_block(block_amount, &state.player.statuses);
    state.player.block = Block(state.player.block.0 + actual);
    events.push(Event::PlayerBlocked { amount: actual });
}

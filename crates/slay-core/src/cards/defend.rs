use crate::combat::{CombatState, Event};
use crate::types::Block;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>) {
    let amount = 5;
    state.player.block = Block(state.player.block.0 + amount);
    events.push(Event::PlayerBlocked { amount });
}

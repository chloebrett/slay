use crate::combat::{CombatState, Event, damage_player};
use crate::types::Block;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, hp_loss: i32, block: i32) {
    damage_player(state, events, hp_loss);
    state.player.block = Block(state.player.block.0 + block);
    events.push(Event::PlayerBlocked { amount: block });
}

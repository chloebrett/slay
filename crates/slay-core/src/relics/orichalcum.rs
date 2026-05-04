use crate::combat::{CombatState, Event};
use crate::types::Block;

pub(super) fn id() -> &'static str { "orichalcum" }

pub(super) fn on_turn_end(state: &mut CombatState, events: &mut Vec<Event>, _hand_size_before_discard: usize) {
    if state.player.block == Block(0) {
        state.player.block.0 += 6;
        events.push(Event::PlayerBlocked { amount: 6 });
    }
}

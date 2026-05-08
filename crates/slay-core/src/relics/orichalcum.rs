use crate::combat::{CombatState, Event};
use crate::types::Block;
use super::RelicDef;

pub(super) fn id() -> &'static str { "orichalcum" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Orichalcum", description: "If you end your turn without Block, gain 6 Block." } }

pub(super) fn on_turn_end(state: &mut CombatState, events: &mut Vec<Event>, _hand_size_before_discard: usize) {
    if state.player.block == Block(0) {
        state.player.block.0 += 6;
        events.push(Event::PlayerBlocked { amount: 6 });
    }
}

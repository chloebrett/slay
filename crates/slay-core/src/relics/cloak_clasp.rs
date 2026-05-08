use crate::combat::{CombatState, Event};
use super::RelicDef;

pub(super) fn id() -> &'static str { "cloak-clasp" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Cloak Clasp", description: "At the end of your turn, gain 1 Block for each card in your hand." } }

pub(super) fn on_turn_end(state: &mut CombatState, events: &mut Vec<Event>, hand_size_before_discard: usize) {
    let gain = hand_size_before_discard as i32;
    if gain > 0 {
        state.player.block.0 += gain;
        events.push(Event::PlayerBlocked { amount: gain });
    }
}

use crate::combat::{CombatState, Event};
use super::RelicDef;

pub(super) fn id() -> &'static str { "pocketwatch" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Pocketwatch" } }

pub(super) fn on_turn_end(state: &mut CombatState, _events: &mut Vec<Event>, _hand_size_before_discard: usize) {
    if state.cards_played_this_turn <= 3 {
        state.extra_draws_next_turn += 3;
    }
}

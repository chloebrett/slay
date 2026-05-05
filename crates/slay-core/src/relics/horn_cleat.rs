use crate::combat::{CombatState, Event};
use crate::rng::Rng;
use super::RelicDef;

pub(super) fn id() -> &'static str { "horn-cleat" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Horn Cleat" } }

pub(super) fn on_turn_start(state: &mut CombatState, events: &mut Vec<Event>, _rng: &mut impl Rng) {
    if state.turn == 2 {
        state.player.block.0 += 14;
        events.push(Event::PlayerBlocked { amount: 14 });
    }
}

use crate::combat::{CombatState, Event};
use crate::rng::Rng;
use super::RelicDef;

pub(super) fn id() -> &'static str { "captains-wheel" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Captain's Wheel" } }

pub(super) fn on_turn_start(state: &mut CombatState, events: &mut Vec<Event>, _rng: &mut impl Rng) {
    if state.turn == 3 {
        state.player.block.0 += 18;
        events.push(Event::PlayerBlocked { amount: 18 });
    }
}

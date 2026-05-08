use crate::combat::{CombatState, Event};
use crate::rng::Rng;
use super::RelicDef;

pub(super) fn id() -> &'static str { "happy-flower" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Happy Flower", description: "Every 3 turns, gain 1 energy." } }

pub(super) fn on_turn_start(state: &mut CombatState, events: &mut Vec<Event>, _rng: &mut impl Rng) {
    if state.turn.is_multiple_of(3) {
        state.player.energy.0 += 1;
        events.push(Event::EnergyGained { amount: 1 });
    }
}

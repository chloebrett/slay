use crate::combat::{CombatState, Event};
use crate::rng::Rng;
use super::RelicDef;

pub(super) fn id() -> &'static str { "candelabra" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Candelabra" } }

pub(super) fn on_turn_start(state: &mut CombatState, events: &mut Vec<Event>, _rng: &mut impl Rng) {
    if state.turn == 2 {
        state.player.energy.0 += 2;
        events.push(Event::EnergyGained { amount: 2 });
    }
}

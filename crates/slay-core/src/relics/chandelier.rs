use crate::combat::{CombatState, Event};
use crate::rng::Rng;

pub(super) fn id() -> &'static str { "chandelier" }

pub(super) fn on_turn_start(state: &mut CombatState, events: &mut Vec<Event>, _rng: &mut impl Rng) {
    if state.turn == 3 {
        state.player.energy.0 += 3;
        events.push(Event::EnergyGained { amount: 3 });
    }
}

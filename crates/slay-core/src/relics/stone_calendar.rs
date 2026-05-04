use crate::combat::{CombatState, Event};
use crate::rng::Rng;

pub(super) fn id() -> &'static str { "stone-calendar" }

pub(super) fn on_turn_start(state: &mut CombatState, events: &mut Vec<Event>, _rng: &mut impl Rng) {
    if state.turn == 7 {
        super::damage_all_enemies(state, events, 52);
    }
}

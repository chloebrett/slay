use crate::combat::{CombatState, Event};
use crate::rng::Rng;
use super::RelicDef;

pub(super) fn id() -> &'static str { "stone-calendar" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Stone Calendar" } }

pub(super) fn on_turn_start(state: &mut CombatState, events: &mut Vec<Event>, _rng: &mut impl Rng) {
    if state.turn == 7 {
        super::damage_all_enemies(state, events, 52);
    }
}

use crate::combat::{CombatState, Event};
use crate::rng::Rng;
use super::RelicDef;

pub(super) fn id() -> &'static str { "mercury-hourglass" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Mercury Hourglass" } }

pub(super) fn on_turn_start(state: &mut CombatState, events: &mut Vec<Event>, _rng: &mut impl Rng) {
    super::damage_all_enemies(state, events, 3);
}

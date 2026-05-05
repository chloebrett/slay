use crate::combat::{CombatState, Event};
use crate::rng::Rng;
use super::RelicDef;

pub(super) fn id() -> &'static str { "festive-popper" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Festive Popper" } }

pub(super) fn on_combat_start(state: &mut CombatState, events: &mut Vec<Event>, _rng: &mut impl Rng, _is_boss: bool) {
    super::damage_all_enemies(state, events, 9);
}

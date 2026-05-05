use crate::combat::{CombatState, Event};
use crate::rng::Rng;
use super::RelicDef;

pub(super) fn id() -> &'static str { "blood-vial" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Blood Vial" } }

pub(super) fn on_combat_start(state: &mut CombatState, events: &mut Vec<Event>, _rng: &mut impl Rng, _is_boss: bool) {
    super::heal_player(&mut state.player, 2, events);
}

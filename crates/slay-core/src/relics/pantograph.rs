use crate::combat::{CombatState, Event};
use crate::rng::Rng;

pub(super) fn id() -> &'static str { "pantograph" }

pub(super) fn on_combat_start(state: &mut CombatState, events: &mut Vec<Event>, _rng: &mut impl Rng, is_boss: bool) {
    if is_boss {
        super::heal_player(&mut state.player, 25, events);
    }
}

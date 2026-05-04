use crate::combat::{apply_status, CombatState, Event, Target};
use crate::rng::Rng;
use crate::status::StatusEffect;

pub(super) fn id() -> &'static str { "vajra" }

pub(super) fn on_combat_start(state: &mut CombatState, events: &mut Vec<Event>, _rng: &mut impl Rng, _is_boss: bool) {
    apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Strength, 1, events);
}

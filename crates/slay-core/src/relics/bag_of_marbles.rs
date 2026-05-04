use crate::combat::{apply_status, CombatState, Event, Target};
use crate::rng::Rng;
use crate::status::StatusEffect;
use crate::types::Hp;

pub(super) fn id() -> &'static str { "bag-of-marbles" }

pub(super) fn on_combat_start(state: &mut CombatState, events: &mut Vec<Event>, _rng: &mut impl Rng, _is_boss: bool) {
    for i in 0..state.enemies.len() {
        if state.enemies[i].hp > Hp(0) {
            apply_status(&mut state.enemies[i].statuses, Target::Enemy, StatusEffect::Vulnerable, 1, events);
        }
    }
}

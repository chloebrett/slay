use crate::combat::{apply_status, CombatState, Event, Target};
use crate::rng::Rng;
use crate::status::StatusEffect;
use crate::types::Hp;
use super::RelicDef;

pub(super) fn id() -> &'static str { "red-mask" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Red Mask" } }

pub(super) fn on_combat_start(state: &mut CombatState, events: &mut Vec<Event>, _rng: &mut impl Rng, _is_boss: bool) {
    for i in 0..state.enemies.len() {
        if state.enemies[i].hp > Hp(0) {
            apply_status(&mut state.enemies[i].statuses, Target::Enemy, StatusEffect::Weak, 1, events);
        }
    }
}

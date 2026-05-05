use crate::combat::{CombatState, Event};
use crate::rng::Rng;
use super::RelicDef;

pub(super) fn id() -> &'static str { "lantern" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Lantern" } }

pub(super) fn on_combat_start(state: &mut CombatState, events: &mut Vec<Event>, _rng: &mut impl Rng, _is_boss: bool) {
    state.player.max_energy.0 += 1;
    state.player.energy.0 += 1;
    events.push(Event::EnergyGained { amount: 1 });
}

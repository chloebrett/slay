use crate::combat::{CombatState, Event};
use crate::rng::Rng;

pub(super) fn id() -> &'static str { "anchor" }

pub(super) fn on_combat_start(state: &mut CombatState, events: &mut Vec<Event>, _rng: &mut impl Rng, _is_boss: bool) {
    state.player.block.0 += 10;
    events.push(Event::PlayerBlocked { amount: 10 });
}

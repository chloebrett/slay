use crate::combat::{draw_cards, CombatState, Event};
use crate::rng::Rng;
use super::RelicDef;

pub(super) fn id() -> &'static str { "gremlin-horn" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Gremlin Horn" } }

pub(super) fn on_enemy_died(state: &mut CombatState, events: &mut Vec<Event>, rng: &mut impl Rng) {
    state.player.energy.0 += 1;
    events.push(Event::EnergyGained { amount: 1 });
    draw_cards(&mut state.player, 1, rng);
    events.push(Event::CardsDrawn { count: 1 });
}

use crate::combat::{draw_cards, CombatState, Event};
use crate::rng::Rng;
use super::RelicDef;

pub(super) fn id() -> &'static str { "pendulum" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Pendulum", description: "Every 3 turns, draw 1 card." } }

pub(super) fn on_turn_start(state: &mut CombatState, events: &mut Vec<Event>, rng: &mut impl Rng) {
    if state.turn.is_multiple_of(3) {
        draw_cards(&mut state.player, 1, rng);
        events.push(Event::CardsDrawn { count: 1 });
    }
}

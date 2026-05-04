use crate::combat::{draw_cards, CombatState, Event};
use crate::rng::Rng;

pub(super) fn id() -> &'static str { "bag-of-preparation" }

pub(super) fn on_combat_start(state: &mut CombatState, events: &mut Vec<Event>, rng: &mut impl Rng, _is_boss: bool) {
    draw_cards(&mut state.player, 2, rng);
    events.push(Event::CardsDrawn { count: 2 });
}

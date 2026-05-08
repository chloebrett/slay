use crate::combat::{draw_cards, CombatState, Event};
use crate::rng::Rng;
use super::RelicDef;

pub(super) fn id() -> &'static str { "bag-of-preparation" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Bag of Preparation", description: "At the start of each combat, draw 2 additional cards." } }

pub(super) fn on_combat_start(state: &mut CombatState, events: &mut Vec<Event>, rng: &mut impl Rng, _is_boss: bool) {
    draw_cards(&mut state.player, 2, rng);
    events.push(Event::CardsDrawn { count: 2 });
}

use crate::cards::CardType;
use crate::combat::{CombatState, Event};
use crate::rng::Rng;

pub(super) fn id() -> &'static str { "nunchaku" }

pub(super) fn on_card_play(state: &mut CombatState, events: &mut Vec<Event>, card_type: CardType, _rng: &mut impl Rng) {
    if card_type == CardType::Attack && state.attacks_this_combat.is_multiple_of(10) {
        state.player.energy.0 += 1;
        events.push(Event::EnergyGained { amount: 1 });
    }
}

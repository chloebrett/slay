use crate::cards::CardType;
use crate::combat::{CombatState, Event};
use crate::rng::Rng;
use super::RelicDef;

pub(super) fn id() -> &'static str { "ornamental-fan" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Ornamental Fan", description: "Every 3 Attacks played in a single turn, gain 4 Block." } }

pub(super) fn on_card_play(state: &mut CombatState, events: &mut Vec<Event>, card_type: CardType, _rng: &mut impl Rng) {
    if card_type == CardType::Attack && state.attacks_this_turn.is_multiple_of(3) {
        state.player.block.0 += 4;
        events.push(Event::PlayerBlocked { amount: 4 });
    }
}

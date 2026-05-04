use crate::cards::CardType;
use crate::combat::{apply_status, CombatState, Event, Target};
use crate::rng::Rng;
use crate::status::StatusEffect;

pub(super) fn id() -> &'static str { "kunai" }

pub(super) fn on_card_play(state: &mut CombatState, events: &mut Vec<Event>, card_type: CardType, _rng: &mut impl Rng) {
    if card_type == CardType::Attack && state.attacks_this_turn.is_multiple_of(3) {
        apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Dexterity, 1, events);
    }
}

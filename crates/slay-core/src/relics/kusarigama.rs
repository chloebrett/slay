use crate::cards::CardType;
use crate::combat::{CombatState, Event};
use crate::rng::Rng;
use super::RelicDef;

pub(super) fn id() -> &'static str { "kusarigama" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Kusarigama", description: "Every 3 Attacks played in a single turn, deal 6 damage to a random enemy." } }

pub(super) fn on_card_play(state: &mut CombatState, events: &mut Vec<Event>, card_type: CardType, rng: &mut impl Rng) {
    if card_type == CardType::Attack && state.attacks_this_turn.is_multiple_of(3) {
        super::damage_random_living_enemy(state, events, 6, rng);
    }
}

use crate::cards::CardType;
use crate::combat::{CombatState, Event};
use crate::rng::Rng;
use super::RelicDef;

pub(super) fn id() -> &'static str { "letter-opener" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Letter Opener", description: "Every 3 Skills played in a single turn, deal 5 damage to ALL enemies." } }

pub(super) fn on_card_play(state: &mut CombatState, events: &mut Vec<Event>, card_type: CardType, _rng: &mut impl Rng) {
    if card_type == CardType::Skill && state.skills_this_turn.is_multiple_of(3) {
        super::damage_all_enemies(state, events, 5);
    }
}

use crate::cards::CardType;
use crate::combat::{CombatState, Event};
use crate::rng::Rng;
use super::RelicDef;

pub(super) fn id() -> &'static str { "tuning-fork" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Tuning Fork" } }

pub(super) fn on_card_play(state: &mut CombatState, events: &mut Vec<Event>, card_type: CardType, _rng: &mut impl Rng) {
    if card_type == CardType::Skill && state.skills_this_combat.is_multiple_of(10) {
        state.player.block.0 += 7;
        events.push(Event::PlayerBlocked { amount: 7 });
    }
}

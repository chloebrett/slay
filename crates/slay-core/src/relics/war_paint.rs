use crate::cards::CardType;
use crate::combat::{Event, Player};
use crate::rng::Rng;

pub(super) fn id() -> &'static str { "war-paint" }

pub(super) fn on_grant(player: &mut Player, events: &mut Vec<Event>, rng: &mut impl Rng) {
    super::upgrade_random_of_type(player, CardType::Skill, 2, rng, events);
}

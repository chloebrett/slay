use crate::cards::CardType;
use crate::combat::{Event, Player};
use crate::rng::Rng;
use super::RelicDef;

pub(super) fn id() -> &'static str { "war-paint" }
pub(super) fn def() -> RelicDef { RelicDef { name: "War Paint", description: "Upon pickup, upgrade 2 random Skills." } }

pub(super) fn on_grant(player: &mut Player, events: &mut Vec<Event>, rng: &mut impl Rng) {
    super::upgrade_random_of_type(player, CardType::Skill, 2, rng, events);
}

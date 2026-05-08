use crate::cards::CardType;
use crate::combat::{Event, Player};
use crate::rng::Rng;
use super::RelicDef;

pub(super) fn id() -> &'static str { "whetstone" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Whetstone", description: "Upon pickup, upgrade 2 random Attacks." } }

pub(super) fn on_grant(player: &mut Player, events: &mut Vec<Event>, rng: &mut impl Rng) {
    super::upgrade_random_of_type(player, CardType::Attack, 2, rng, events);
}

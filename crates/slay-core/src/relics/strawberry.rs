use crate::combat::{Event, Player};
use crate::rng::Rng;
use super::RelicDef;

pub(super) fn id() -> &'static str { "strawberry" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Strawberry", description: "Upon pickup, raise your Max HP by 7." } }

pub(super) fn on_grant(player: &mut Player, _events: &mut Vec<Event>, _rng: &mut impl Rng) {
    super::raise_max_hp(player, 7);
}

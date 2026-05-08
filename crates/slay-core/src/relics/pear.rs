use crate::combat::{Event, Player};
use crate::rng::Rng;
use super::RelicDef;

pub(super) fn id() -> &'static str { "pear" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Pear", description: "Upon pickup, raise your Max HP by 10." } }

pub(super) fn on_grant(player: &mut Player, _events: &mut Vec<Event>, _rng: &mut impl Rng) {
    super::raise_max_hp(player, 10);
}

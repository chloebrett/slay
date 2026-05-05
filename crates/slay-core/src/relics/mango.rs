use crate::combat::{Event, Player};
use crate::rng::Rng;
use super::RelicDef;

pub(super) fn id() -> &'static str { "mango" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Mango" } }

pub(super) fn on_grant(player: &mut Player, _events: &mut Vec<Event>, _rng: &mut impl Rng) {
    super::raise_max_hp(player, 14);
}

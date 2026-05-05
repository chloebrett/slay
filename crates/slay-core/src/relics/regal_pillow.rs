use crate::combat::{Event, Player};
use super::RelicDef;

pub(super) fn id() -> &'static str { "regal-pillow" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Regal Pillow" } }

pub(super) fn on_rest(player: &mut Player, events: &mut Vec<Event>) {
    super::heal_player(player, 15, events);
}

use crate::combat::{Event, Player};
use super::RelicDef;

pub(super) fn id() -> &'static str { "burning-blood" }
pub(super) fn def() -> RelicDef { RelicDef { name: "Burning Blood" } }

pub(super) fn on_combat_end(player: &mut Player, events: &mut Vec<Event>) {
    super::heal_player(player, 6, events);
}

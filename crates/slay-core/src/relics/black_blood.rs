use crate::combat::{Event, Player};

pub(super) fn id() -> &'static str { "black-blood" }

pub(super) fn on_combat_end(player: &mut Player, events: &mut Vec<Event>) {
    super::heal_player(player, 12, events);
}

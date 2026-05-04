use crate::combat::{Event, Player};
use crate::rng::Rng;

pub(super) fn id() -> &'static str { "old-coin" }

pub(super) fn on_grant(player: &mut Player, _events: &mut Vec<Event>, _rng: &mut impl Rng) {
    player.gold += 300;
}

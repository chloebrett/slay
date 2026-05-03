mod cards;
mod combat;
mod enemies;
mod rng;
pub(crate) mod status;
mod types;

pub use cards::Card;
pub use combat::{
    apply_command, Command, CommandError, CombatPhase, CombatState, Enemy, Event, Player, Target,
};
pub use enemies::{EnemyKind, Intent};
pub use rng::{NoOpRng, Rng, ThreadRng};
pub use status::{StatusEffect, StatusMap};
pub use types::{Block, Energy, Hp};

pub fn welcome() -> &'static str {
    "Slay the Spire"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn welcome_message_identifies_the_game() {
        assert_eq!(welcome(), "Slay the Spire");
    }
}

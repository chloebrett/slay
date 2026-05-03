mod combat;
mod types;

pub use combat::{
    apply_command, Command, CommandError, CombatPhase, CombatState, Enemy, Event, Player,
};
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

mod cards;
mod combat;
pub mod relics;
mod enemies;
mod rng;
pub mod run;
pub(crate) mod status;
mod types;

pub use cards::{Card, CardDef, CardType, reward_pool, starter_deck};
pub use combat::{CombatPhase, CombatState, Enemy, Event, Player, Target};
pub use relics::{Relic, grant_relic};
pub use enemies::{EnemyKind, Intent};
pub use rng::{NoOpRng, Rng, ThreadRng};
pub use run::{
    apply_command, Command, CommandError, CardRewardState, GameState, MapNode, MapState,
    RestSiteState, new_run,
};
pub use status::{StatusEffect, StatusMap};
pub use types::{Block, Energy, Hp};

pub fn welcome() -> &'static str {
    "Slay the Spire"
}

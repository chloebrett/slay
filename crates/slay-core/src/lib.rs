mod cards;
mod combat;
pub mod relics;
mod enemies;
mod potions;
mod rng;
pub mod run;
pub(crate) mod status;
mod types;

pub use cards::{Card, CardCost, CardDef, CardType, Grade, reward_pool, starter_deck};
pub use potions::{Potion, MAX_POTIONS};
pub use combat::{CombatPhase, CombatState, Enemy, Event, Player, Target};
pub use relics::{Relic, grant_relic};
pub use enemies::{EnemyKind, Intent, Move};
pub use rng::{AnyRng, NoOpRng, Rng, ThreadRng};
pub use run::{
    apply_command, Command, CommandError, CardRewardState, EventKind, EventRoomState, GameState,
    MapGraph, MapNode, MapState, RestSiteState, Scenario, ShopState, TreasureRoomState,
    CARD_PRICE, RELIC_PRICE, POTION_PRICE, new_run, new_simple_run, generate_map,
};
pub use status::{StatusEffect, StatusMap};
pub use types::{Block, Energy, Hp};

pub fn welcome() -> &'static str {
    "Slay the Spire is a registered trademark by Mega Crit, LLC. Please support the developers of this amazing game on Steam: https://store.steampowered.com/app/646570/Slay_the_Spire/"
}

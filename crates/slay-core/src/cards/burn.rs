use super::{CardDef, CardDescription, CardType, EndOfTurnHook};
use crate::types::Energy;

pub(super) fn def() -> CardDef {
    CardDef { name: "Burn", description: CardDescription::Static("Unplayable. At the end of your turn, take 2 damage."), energy_cost: Energy(0), card_type: CardType::Status }
}

pub(super) fn id() -> &'static str { "burn" }

pub(super) fn end_of_turn_hook() -> EndOfTurnHook {
    EndOfTurnHook::BlockableDamage(2)
}

pub(super) fn plus_def() -> CardDef {
    CardDef { name: "Burn+", description: CardDescription::Static("Unplayable. At the end of your turn, take 4 damage."), energy_cost: Energy(0), card_type: CardType::Status }
}

pub(super) fn plus_id() -> &'static str { "burn+" }

pub(super) fn plus_end_of_turn_hook() -> EndOfTurnHook {
    EndOfTurnHook::BlockableDamage(4)
}

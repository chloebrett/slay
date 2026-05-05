use super::{CardDef, CardDescription, CardType};
use crate::types::Energy;

pub(super) fn def() -> CardDef {
    CardDef { name: "Decay", description: CardDescription::Static("Unplayable. At the end of your turn, take 2 damage."), energy_cost: Energy(0), card_type: CardType::Curse }
}

pub(super) fn id() -> &'static str { "decay" }

pub(super) fn end_of_turn_hook() -> super::EndOfTurnHook {
    super::EndOfTurnHook::BlockableDamage(2)
}

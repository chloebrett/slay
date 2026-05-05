use super::{CardDef, CardDescription, CardType};
use crate::types::Energy;

pub(super) fn def() -> CardDef {
    CardDef { name: "Regret", description: CardDescription::Static("Unplayable. At the end of your turn, lose 1 HP."), energy_cost: Energy(0), card_type: CardType::Curse }
}

pub(super) fn id() -> &'static str { "regret" }

pub(super) fn end_of_turn_hook() -> super::EndOfTurnHook {
    super::EndOfTurnHook::DirectHpLoss(1)
}

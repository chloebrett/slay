use super::{CardDef, CardDescription, CardType, EndOfTurnHook};
use crate::types::Energy;

pub(super) fn def() -> CardDef {
    CardDef { name: "Regret", description: CardDescription::Static("Unplayable. At the end of your turn, lose HP equal to the number of cards in your hand."), energy_cost: Energy(0), card_type: CardType::Curse }
}

pub(super) fn id() -> &'static str { "regret" }

pub(super) fn end_of_turn_hook(hand_size: i32) -> EndOfTurnHook {
    EndOfTurnHook::DirectHpLoss(hand_size)
}

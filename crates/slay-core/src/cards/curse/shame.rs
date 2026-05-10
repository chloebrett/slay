use super::{CardDef, CardDescription, CardType, EndOfTurnHook};
use crate::status::StatusEffect;
use crate::types::Energy;

pub(super) fn def() -> CardDef {
    CardDef { name: "Shame", description: CardDescription::Static("Unplayable. At the end of your turn, gain 1 Frail."), energy_cost: Energy(0), card_type: CardType::Curse }
}

pub(super) fn id() -> &'static str { "shame" }

pub(super) fn end_of_turn_hook() -> EndOfTurnHook {
    EndOfTurnHook::ApplyPlayerStatus { effect: StatusEffect::Frail, amount: 1 }
}

use super::{CardDef, CardDescription, CardType};
use crate::types::Energy;

pub(super) fn def() -> CardDef {
    CardDef { name: "Pain", description: CardDescription::Static("Unplayable. When you play a card, lose 1 HP."), energy_cost: Energy(0), card_type: CardType::Curse }
}

pub(super) fn id() -> &'static str { "pain" }

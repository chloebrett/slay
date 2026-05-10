use super::{CardDef, CardDescription, CardType};
use crate::types::Energy;

pub(super) fn def() -> CardDef {
    CardDef { name: "Writhe", description: CardDescription::Static("Innate. Unplayable."), energy_cost: Energy(0), card_type: CardType::Curse }
}

pub(super) fn id() -> &'static str { "writhe" }

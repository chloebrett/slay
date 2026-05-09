use super::{CardDef, CardDescription, CardType};
use crate::types::Energy;

pub(super) fn def() -> CardDef {
    CardDef { name: "Void", description: CardDescription::Static("Ethereal. Unplayable. When drawn, lose 1 Energy."), energy_cost: Energy(0), card_type: CardType::Status }
}

pub(super) fn id() -> &'static str { "void" }

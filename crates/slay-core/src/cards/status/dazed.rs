use super::{CardDef, CardDescription, CardType};
use crate::types::Energy;

pub(super) fn def() -> CardDef {
    CardDef { name: "Dazed", description: CardDescription::Static("Unplayable. Ethereal."), energy_cost: Energy(0), card_type: CardType::Status }
}

pub(super) fn id() -> &'static str { "dazed" }

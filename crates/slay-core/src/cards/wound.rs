use super::{CardDef, CardDescription, CardType};
use crate::types::Energy;

pub(super) fn def() -> CardDef {
    CardDef { name: "Wound", description: CardDescription::Static("Unplayable."), energy_cost: Energy(0), card_type: CardType::Status }
}

pub(super) fn id() -> &'static str { "wound" }

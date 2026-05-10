use super::{CardDef, CardDescription, CardType};
use crate::types::Energy;

pub(super) fn def() -> CardDef {
    CardDef { name: "Slimed", description: CardDescription::Static("Exhaust."), energy_cost: Energy(1), card_type: CardType::Status }
}

pub(super) fn id() -> &'static str { "slimed" }

use super::{CardDef, CardDescription, CardType};
use crate::types::Energy;

pub(super) fn def() -> CardDef {
    CardDef { name: "Parasite", description: CardDescription::Static("Unplayable. If transformed or removed from your deck, lose 3 Max HP."), energy_cost: Energy(0), card_type: CardType::Curse }
}

pub(super) fn id() -> &'static str { "parasite" }

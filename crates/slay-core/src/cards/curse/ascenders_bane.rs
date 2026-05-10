use super::{CardDef, CardDescription, CardType};
use crate::types::Energy;

pub(super) fn def() -> CardDef {
    CardDef { name: "Ascender's Bane", description: CardDescription::Static("Unplayable. Ethereal. Cannot be removed from your deck."), energy_cost: Energy(0), card_type: CardType::Curse }
}

pub(super) fn id() -> &'static str { "ascenders_bane" }

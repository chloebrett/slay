use super::{CardDef, CardDescription, CardType};
use crate::types::Energy;

pub(super) fn def() -> CardDef {
    CardDef { name: "Curse of the Bell", description: CardDescription::Static("Unplayable. Cannot be removed from your deck."), energy_cost: Energy(0), card_type: CardType::Curse }
}

pub(super) fn id() -> &'static str { "curse_of_the_bell" }

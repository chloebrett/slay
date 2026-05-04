use super::{CardDef, CardDescription, CardType, Grade};
use crate::types::Energy;

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, base) = match grade { Grade::Base => ("Bludgeon", 32), Grade::Plus => ("Bludgeon+", 42) };
    CardDef { name, description: CardDescription::WithDamage { template: "Deal {damage} damage.", base }, energy_cost: Energy(3), card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "bludgeon", Grade::Plus => "bludgeon-plus" }
}

use super::{CardDef, CardDescription, CardType, Grade};
use crate::types::Energy;

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Impervious",  CardDescription::Static("Gain 30 Block. Exhaust.")),
        Grade::Plus => ("Impervious+", CardDescription::Static("Gain 40 Block. Exhaust.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(2), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "impervious", Grade::Plus => "impervious-plus" }
}

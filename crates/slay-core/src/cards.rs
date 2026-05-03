use crate::types::Energy;

#[derive(Debug, Clone, PartialEq)]
pub enum Card {
    Strike,
    Defend,
}

impl Card {
    pub fn energy_cost(&self) -> Energy {
        Energy(1)
    }

    pub fn name(&self) -> &'static str {
        match self {
            Card::Strike => "Strike",
            Card::Defend => "Defend",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Card::Strike => "Deal 6 damage.",
            Card::Defend => "Gain 5 block.",
        }
    }
}

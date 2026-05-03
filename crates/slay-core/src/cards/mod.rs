mod bash;
mod clothesline;
mod defend;
mod strike;

use crate::types::Energy;

#[derive(Debug, Clone, PartialEq)]
pub enum Card {
    Strike,
    Defend,
    Bash,
    Clothesline,
}

#[derive(Debug, Clone, Copy)]
pub struct CardDef {
    pub name: &'static str,
    pub description: &'static str,
    pub energy_cost: Energy,
}

impl Card {
    pub fn def(&self) -> CardDef {
        match self {
            Card::Strike => CardDef {
                name: "Strike",
                description: "Deal 6 damage.",
                energy_cost: Energy(1),
            },
            Card::Defend => CardDef {
                name: "Defend",
                description: "Gain 5 block.",
                energy_cost: Energy(1),
            },
            Card::Bash => CardDef {
                name: "Bash",
                description: "Deal 8 damage. Apply 2 Vulnerable.",
                energy_cost: Energy(2),
            },
            Card::Clothesline => CardDef {
                name: "Clothesline",
                description: "Deal 12 damage. Apply 2 Weak.",
                energy_cost: Energy(2),
            },
        }
    }

    pub fn name(&self) -> &'static str { self.def().name }
    pub fn description(&self) -> &'static str { self.def().description }
    pub fn energy_cost(&self) -> Energy { self.def().energy_cost }
}

pub fn apply(card: &Card, state: &mut crate::combat::CombatState, events: &mut Vec<crate::combat::Event>) {
    match card {
        Card::Strike => strike::apply(state, events),
        Card::Defend => defend::apply(state, events),
        Card::Bash => bash::apply(state, events),
        Card::Clothesline => clothesline::apply(state, events),
    }
}

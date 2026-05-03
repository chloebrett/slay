mod bash;
mod clothesline;
mod deadly_poison;
mod defend;
mod inflame;
mod strike;

use crate::status::{StatusMap, resolve_damage};
use crate::types::Energy;

#[derive(Debug, Clone, PartialEq)]
pub enum Card {
    Strike,
    Defend,
    Bash,
    Clothesline,
    Inflame,
    DeadlyPoison,
}

#[derive(Debug, Clone, Copy)]
pub struct CardDef {
    pub name: &'static str,
    /// Description template: use `{damage}` as a placeholder where the damage number goes.
    pub description: &'static str,
    pub energy_cost: Energy,
    pub base_damage: Option<i32>,
}

impl Card {
    pub fn def(&self) -> CardDef {
        match self {
            Card::Strike => CardDef {
                name: "Strike",
                description: "Deal {damage} damage.",
                energy_cost: Energy(1),
                base_damage: Some(6),
            },
            Card::Defend => CardDef {
                name: "Defend",
                description: "Gain 5 block.",
                energy_cost: Energy(1),
                base_damage: None,
            },
            Card::Bash => CardDef {
                name: "Bash",
                description: "Deal {damage} damage. Apply 2 Vulnerable.",
                energy_cost: Energy(2),
                base_damage: Some(8),
            },
            Card::Clothesline => CardDef {
                name: "Clothesline",
                description: "Deal {damage} damage. Apply 2 Weak.",
                energy_cost: Energy(2),
                base_damage: Some(12),
            },
            Card::Inflame => CardDef {
                name: "Inflame",
                description: "Gain 2 Strength.",
                energy_cost: Energy(1),
                base_damage: None,
            },
            Card::DeadlyPoison => CardDef {
                name: "Deadly Poison",
                description: "Apply 5 Poison.",
                energy_cost: Energy(1),
                base_damage: None,
            },
        }
    }

    pub fn name(&self) -> &'static str { self.def().name }
    pub fn energy_cost(&self) -> Energy { self.def().energy_cost }

    /// Description with base damage values substituted (no emphasis).
    pub fn description(&self) -> String {
        let def = self.def();
        match def.base_damage {
            None => def.description.to_string(),
            Some(base) => def.description.replace("{damage}", &base.to_string()),
        }
    }

    /// Description with effective damage substituted; uses `*N*` emphasis when modified by statuses.
    pub fn effective_description(&self, attacker: &StatusMap, defender: &StatusMap) -> String {
        let def = self.def();
        let Some(base) = def.base_damage else {
            return def.description.to_string();
        };
        let eff = resolve_damage(base, attacker, defender);
        let num = if eff != base { format!("*{eff}*") } else { eff.to_string() };
        def.description.replace("{damage}", &num)
    }

    pub fn effective_damage(&self, attacker: &StatusMap, defender: &StatusMap) -> Option<i32> {
        self.def().base_damage.map(|base| resolve_damage(base, attacker, defender))
    }
}

pub fn starter_deck() -> Vec<Card> {
    let mut deck = Vec::new();
    for _ in 0..5 {
        deck.push(Card::Strike);
    }
    for _ in 0..3 {
        deck.push(Card::Defend);
    }
    deck.push(Card::Bash);
    deck.push(Card::Inflame);
    deck.push(Card::DeadlyPoison);
    deck
}

pub fn apply(card: &Card, state: &mut crate::combat::CombatState, events: &mut Vec<crate::combat::Event>) {
    match card {
        Card::Strike => strike::apply(state, events),
        Card::Defend => defend::apply(state, events),
        Card::Bash => bash::apply(state, events),
        Card::Clothesline => clothesline::apply(state, events),
        Card::Inflame => inflame::apply(state, events),
        Card::DeadlyPoison => deadly_poison::apply(state, events),
    }
}

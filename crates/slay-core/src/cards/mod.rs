mod bash;
mod clothesline;
mod deadly_poison;
mod defend;
mod disarm;
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
    Disarm,
}

#[derive(Debug, Clone, Copy)]
pub enum CardDescription {
    Static(&'static str),
    WithDamage { template: &'static str, base: i32 },
}

#[derive(Debug, Clone, Copy)]
pub struct CardDef {
    pub name: &'static str,
    pub description: CardDescription,
    pub energy_cost: Energy,
}

impl Card {
    pub fn def(&self) -> CardDef {
        match self {
            Card::Strike => CardDef {
                name: "Strike",
                description: CardDescription::WithDamage { template: "Deal {damage} damage.", base: 6 },
                energy_cost: Energy(1),
            },
            Card::Defend => CardDef {
                name: "Defend",
                description: CardDescription::Static("Gain 5 block."),
                energy_cost: Energy(1),
            },
            Card::Bash => CardDef {
                name: "Bash",
                description: CardDescription::WithDamage { template: "Deal {damage} damage. Apply 2 Vulnerable.", base: 8 },
                energy_cost: Energy(2),
            },
            Card::Clothesline => CardDef {
                name: "Clothesline",
                description: CardDescription::WithDamage { template: "Deal {damage} damage. Apply 2 Weak.", base: 12 },
                energy_cost: Energy(2),
            },
            Card::Inflame => CardDef {
                name: "Inflame",
                description: CardDescription::Static("Gain 2 Strength."),
                energy_cost: Energy(1),
            },
            Card::DeadlyPoison => CardDef {
                name: "Deadly Poison",
                description: CardDescription::Static("Apply 5 Poison."),
                energy_cost: Energy(1),
            },
            Card::Disarm => CardDef {
                name: "Disarm",
                description: CardDescription::Static("Enemy loses 2 Strength. Exhaust."),
                energy_cost: Energy(1),
            },
        }
    }

    pub fn exhausts(&self) -> bool {
        matches!(self, Card::Disarm)
    }

    pub fn name(&self) -> &'static str { self.def().name }
    pub fn energy_cost(&self) -> Energy { self.def().energy_cost }

    pub fn description(&self) -> String {
        match self.def().description {
            CardDescription::Static(s) => s.to_string(),
            CardDescription::WithDamage { template, base } => {
                template.replace("{damage}", &base.to_string())
            }
        }
    }

    pub fn effective_description(&self, attacker: &StatusMap, defender: &StatusMap) -> String {
        match self.def().description {
            CardDescription::Static(s) => s.to_string(),
            CardDescription::WithDamage { template, base } => {
                let eff = resolve_damage(base, attacker, defender);
                let num = if eff != base { format!("*{eff}*") } else { eff.to_string() };
                template.replace("{damage}", &num)
            }
        }
    }

    pub fn effective_damage(&self, attacker: &StatusMap, defender: &StatusMap) -> Option<i32> {
        match self.def().description {
            CardDescription::WithDamage { base, .. } => Some(resolve_damage(base, attacker, defender)),
            CardDescription::Static(_) => None,
        }
    }
}

pub fn reward_pool() -> Vec<Card> {
    vec![Card::Bash, Card::Clothesline, Card::Inflame, Card::DeadlyPoison, Card::Strike, Card::Defend]
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
    deck.push(Card::Disarm);
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
        Card::Disarm => disarm::apply(state, events),
    }
}

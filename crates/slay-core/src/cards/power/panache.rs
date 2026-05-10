use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, _events: &mut Vec<Event>, grade: Grade) {
    let damage = GradeValues { base: 10, plus: 14 }.get(grade);
    *state.player.statuses.entry(StatusEffect::Panache).or_insert(0) = damage;
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Panache",  CardDescription::Static("Every time you play 5 cards in a turn, deal 10 damage to ALL enemies.")),
        Grade::Plus => ("Panache+", CardDescription::Static("Every time you play 5 cards in a turn, deal 14 damage to ALL enemies.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(0), card_type: CardType::Power }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "panache", Grade::Plus => "panache-plus" }
}

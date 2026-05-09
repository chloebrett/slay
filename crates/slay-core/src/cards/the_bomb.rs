use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event};
use crate::types::Energy;

pub fn apply(state: &mut CombatState, _events: &mut Vec<Event>, grade: Grade) {
    let damage = GradeValues { base: 40, plus: 50 }.get(grade);
    state.pending_bombs.push((damage, 3));
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("The Bomb",  CardDescription::Static("At the end of 3 turns, deal 40 damage to ALL enemies.")),
        Grade::Plus => ("The Bomb+", CardDescription::Static("At the end of 3 turns, deal 50 damage to ALL enemies.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(2), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "the-bomb", Grade::Plus => "the-bomb-plus" }
}

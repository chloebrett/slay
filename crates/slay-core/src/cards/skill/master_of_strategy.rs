use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, draw_with_triggers};
use crate::rng::Rng;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, rng: &mut impl Rng) {
    let count = GradeValues { base: 3, plus: 4 }.get(grade) as usize;
    draw_with_triggers(state, count, events, rng);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let name = match grade { Grade::Base => "Master of Strategy", Grade::Plus => "Master of Strategy+" };
    CardDef { name, description: CardDescription::Static("Draw 3 cards. Exhaust."), energy_cost: Energy(0), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "master-of-strategy", Grade::Plus => "master-of-strategy-plus" }
}

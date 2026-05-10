use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, gain_player_block};
use crate::rng::Rng;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, rng: &mut impl Rng) {
    let block = GradeValues { base: 6, plus: 9 }.get(grade);
    gain_player_block(state, events, block, rng);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Good Instincts",  CardDescription::Static("Gain 6 Block.")),
        Grade::Plus => ("Good Instincts+", CardDescription::Static("Gain 9 Block.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(0), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "good-instincts", Grade::Plus => "good-instincts-plus" }
}

use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{ChooseCardContext, CombatPhase, CombatState};
use crate::types::Energy;

pub fn apply(state: &mut CombatState, grade: Grade) {
    let draws = GradeValues { base: 2usize, plus: 3usize }.get(grade);
    state.phase = CombatPhase::ChooseCard(ChooseCardContext::BurningPact { draws });
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, description) = match grade {
        Grade::Base => ("Burning Pact",  CardDescription::Static("Exhaust 1 card. Draw 2 cards.")),
        Grade::Plus => ("Burning Pact+", CardDescription::Static("Exhaust 1 card. Draw 3 cards.")),
    };
    CardDef { name, description, energy_cost: Energy(1), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "burning-pact", Grade::Plus => "burning-pact-plus" }
}

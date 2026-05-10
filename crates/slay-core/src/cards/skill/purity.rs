use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{ChooseCardContext, CombatPhase, CombatState, Event};
use crate::types::Energy;

pub fn apply(state: &mut CombatState, _events: &mut Vec<Event>, grade: Grade) {
    if !state.player.hand.is_empty() {
        let remaining = GradeValues { base: 3, plus: 5 }.get(grade) as usize;
        state.phase = CombatPhase::ChooseCard(ChooseCardContext::Purity { remaining });
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Purity",  CardDescription::Static("Exhaust up to 3 cards in your hand. Exhaust.")),
        Grade::Plus => ("Purity+", CardDescription::Static("Exhaust up to 5 cards in your hand. Exhaust.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(0), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "purity", Grade::Plus => "purity-plus" }
}

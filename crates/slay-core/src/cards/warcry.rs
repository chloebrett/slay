use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{ChooseCardContext, CombatPhase, CombatState, Event, draw_with_triggers};
use crate::rng::Rng;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, rng: &mut impl Rng) {
    let draws = GradeValues { base: 1usize, plus: 2usize }.get(grade);
    draw_with_triggers(state, draws, events, rng);
    state.phase = CombatPhase::ChooseCard(ChooseCardContext::Warcry);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, description) = match grade {
        Grade::Base => ("Warcry",  CardDescription::Static("Draw 1 card. Put a card from your hand onto the top of your draw pile. Exhaust.")),
        Grade::Plus => ("Warcry+", CardDescription::Static("Draw 2 cards. Put a card from your hand onto the top of your draw pile. Exhaust.")),
    };
    CardDef { name, description, energy_cost: Energy(0), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "warcry", Grade::Plus => "warcry-plus" }
}

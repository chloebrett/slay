use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{ChooseCardContext, CombatPhase, CombatState, Event, draw_with_triggers};
use crate::rng::Rng;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, _grade: Grade, rng: &mut impl Rng) {
    draw_with_triggers(state, 2, events, rng);
    if !state.player.hand.is_empty() {
        state.phase = CombatPhase::ChooseCard(ChooseCardContext::ThinkingAhead);
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Thinking Ahead",  CardDescription::Static("Draw 2. Put a card from your hand on top of your draw pile. Exhaust.")),
        Grade::Plus => ("Thinking Ahead+", CardDescription::Static("Draw 2. Put a card from your hand on top of your draw pile.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(0), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "thinking-ahead", Grade::Plus => "thinking-ahead-plus" }
}

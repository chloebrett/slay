use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{ChooseCardContext, CombatPhase, CombatState, Event};
use crate::types::Energy;

pub fn apply(state: &mut CombatState, _events: &mut Vec<Event>, _grade: Grade) {
    if !state.player.hand.is_empty() {
        state.phase = CombatPhase::ChooseCard(ChooseCardContext::Forethought);
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let name = match grade { Grade::Base => "Forethought", Grade::Plus => "Forethought+" };
    CardDef { name, description: CardDescription::Static("Place a card from your hand to the bottom of your draw pile."), energy_cost: Energy(0), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "forethought", Grade::Plus => "forethought-plus" }
}

use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event};
use crate::rng::Rng;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, _events: &mut Vec<Event>, _grade: Grade, rng: &mut impl Rng) {
    if state.player.hand.is_empty() {
        return;
    }
    let mut indices: Vec<usize> = (0..state.player.hand.len()).collect();
    rng.shuffle(&mut indices);
    let card = state.player.hand[indices[0]].clone();
    state.zero_cost_cards.push(card);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let name = match grade { Grade::Base => "Madness", Grade::Plus => "Madness+" };
    CardDef { name, description: CardDescription::Static("A random card in your hand costs 0 this combat. Exhaust."), energy_cost: Energy(1), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "madness", Grade::Plus => "madness-plus" }
}

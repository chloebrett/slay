use super::{CardDef, CardDescription, CardType, Grade, attack_pool};
use crate::combat::{CombatState, Event};
use crate::rng::Rng;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, _events: &mut Vec<Event>, _grade: Grade, rng: &mut impl Rng) {
    let mut pool = attack_pool();
    rng.shuffle(&mut pool);
    let cards: Vec<_> = pool.into_iter().take(3).collect();
    for card in &cards {
        state.zero_cost_cards.push(card.clone());
    }
    state.player.hand.extend(cards);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let name = match grade { Grade::Base => "Metamorphosis", Grade::Plus => "Metamorphosis+" };
    CardDef { name, description: CardDescription::Static("Add 3 random Attacks that cost 0 to your hand. Exhaust."), energy_cost: Energy(2), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "metamorphosis", Grade::Plus => "metamorphosis-plus" }
}

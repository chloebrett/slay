use super::{CardDef, CardDescription, CardType, Grade, colorless_reward_pool};
use crate::combat::{CombatState, Event};
use crate::rng::Rng;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, _events: &mut Vec<Event>, grade: Grade, rng: &mut impl Rng, x_value: i32) {
    if x_value <= 0 {
        return;
    }
    let mut pool = colorless_reward_pool();
    rng.shuffle(&mut pool);
    let count = (x_value as usize).min(pool.len());
    let cards: Vec<_> = pool.into_iter().take(count).collect();
    if grade == Grade::Plus {
        for card in &cards {
            state.zero_cost_cards.push(card.clone());
        }
    }
    state.player.hand.extend(cards);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Transmutation",  CardDescription::Static("Create X random Colorless cards in your hand. Exhaust.")),
        Grade::Plus => ("Transmutation+", CardDescription::Static("Create X random Colorless cards that cost 0 in your hand. Exhaust.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(0), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "transmutation", Grade::Plus => "transmutation-plus" }
}

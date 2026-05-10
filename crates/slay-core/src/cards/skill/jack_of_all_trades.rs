use super::{CardDef, CardDescription, CardType, Grade, colorless_reward_pool};
use crate::combat::{CombatState, Event};
use crate::rng::Rng;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, _events: &mut Vec<Event>, _grade: Grade, rng: &mut impl Rng) {
    let mut pool = colorless_reward_pool();
    rng.shuffle(&mut pool);
    state.player.hand.push(pool.remove(0));
}

pub(super) fn def(grade: Grade) -> CardDef {
    let name = match grade { Grade::Base => "Jack of All Trades", Grade::Plus => "Jack of All Trades+" };
    CardDef { name, description: CardDescription::Static("Add 1 random Colorless card to your hand. Exhaust."), energy_cost: Energy(0), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "jack-of-all-trades", Grade::Plus => "jack-of-all-trades-plus" }
}

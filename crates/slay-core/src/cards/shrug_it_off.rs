use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, draw_cards};
use crate::rng::Rng;
use crate::types::{Block, Energy};

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, block: i32, draws: usize, rng: &mut impl Rng) {
    state.player.block = Block(state.player.block.0 + block);
    events.push(Event::PlayerBlocked { amount: block });
    draw_cards(&mut state.player, draws, rng);
    events.push(Event::CardsDrawn { count: draws });
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Shrug It Off",  CardDescription::Static("Gain 8 Block. Draw 1 card.")),
        Grade::Plus => ("Shrug It Off+", CardDescription::Static("Gain 11 Block. Draw 1 card.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(1), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "shrug-it-off", Grade::Plus => "shrug-it-off-plus" }
}

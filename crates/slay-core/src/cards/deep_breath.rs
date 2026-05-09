use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, draw_with_triggers};
use crate::rng::Rng;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, rng: &mut impl Rng) {
    let discard = std::mem::take(&mut state.player.discard_pile);
    state.player.draw_pile.extend(discard);
    rng.shuffle(&mut state.player.draw_pile);
    let draws = GradeValues { base: 1, plus: 2 }.get(grade) as usize;
    draw_with_triggers(state, draws, events, rng);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Deep Breath",  CardDescription::Static("Shuffle your discard pile into your draw pile. Draw 1.")),
        Grade::Plus => ("Deep Breath+", CardDescription::Static("Shuffle your discard pile into your draw pile. Draw 2.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(0), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "deep-breath", Grade::Plus => "deep-breath-plus" }
}

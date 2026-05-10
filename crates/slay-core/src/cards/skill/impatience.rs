use super::{CardDef, CardDescription, CardType, GradeValues, Grade};
use crate::combat::{CombatState, Event, draw_with_triggers};
use crate::rng::Rng;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, rng: &mut impl Rng) {
    let has_attack = state.player.hand.iter().any(|c| c.card_type() == CardType::Attack);
    if !has_attack {
        let draws = GradeValues { base: 2, plus: 3 }.get(grade) as usize;
        draw_with_triggers(state, draws, events, rng);
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, draws) = match grade {
        Grade::Base => ("Impatience",  2),
        Grade::Plus => ("Impatience+", 3),
    };
    CardDef {
        name,
        description: CardDescription::WithDamage { template: "If you have no Attacks in hand, draw {}.", base: draws },
        energy_cost: Energy(0),
        card_type: CardType::Skill,
    }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "impatience", Grade::Plus => "impatience-plus" }
}

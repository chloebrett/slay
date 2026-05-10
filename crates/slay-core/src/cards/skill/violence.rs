use super::{CardDef, CardDescription, CardType, GradeValues, Grade};
use crate::combat::{CombatState, Event};
use crate::rng::Rng;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, _events: &mut Vec<Event>, grade: Grade, rng: &mut impl Rng) {
    let count = GradeValues { base: 3, plus: 5 }.get(grade) as usize;

    let mut attack_indices: Vec<usize> = state.player.draw_pile.iter()
        .enumerate()
        .filter(|(_, c)| c.card_type() == CardType::Attack)
        .map(|(i, _)| i)
        .collect();

    rng.shuffle(&mut attack_indices);

    let mut to_remove: Vec<usize> = attack_indices.into_iter().take(count).collect();
    to_remove.sort_unstable_by(|a, b| b.cmp(a));

    for idx in to_remove {
        let card = state.player.draw_pile.remove(idx);
        state.player.hand.push(card);
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let n = GradeValues { base: 3, plus: 5 }.get(grade);
    let (name, desc) = match grade {
        Grade::Base => ("Violence",  CardDescription::WithDamage { template: "Put {} random Attacks from your draw pile into your hand. Exhaust.", base: n }),
        Grade::Plus => ("Violence+", CardDescription::WithDamage { template: "Put {} random Attacks from your draw pile into your hand. Exhaust.", base: n }),
    };
    CardDef { name, description: desc, energy_cost: Energy(0), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "violence", Grade::Plus => "violence-plus" }
}

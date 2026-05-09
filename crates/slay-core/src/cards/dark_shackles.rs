use super::{CardDef, CardDescription, CardType, GradeValues, Grade};
use crate::combat::{CombatState, Event, Target, apply_status};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, target: usize, grade: Grade) {
    let amount = GradeValues { base: 9, plus: 15 }.get(grade);
    apply_status(&mut state.enemies[target].statuses, Target::Enemy, StatusEffect::Strength, -amount, events);
    apply_status(&mut state.enemies[target].statuses, Target::Enemy, StatusEffect::Shackled, amount, events);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let amount = GradeValues { base: 9, plus: 15 }.get(grade);
    CardDef {
        name: "Dark Shackles",
        description: CardDescription::WithDamage { template: "Enemy loses {} Strength this turn. Exhaust.", base: amount },
        energy_cost: Energy(0),
        card_type: CardType::Skill,
    }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "dark-shackles", Grade::Plus => "dark-shackles-plus" }
}

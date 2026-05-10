use super::{CardDef, CardDescription, CardType, GradeValues, Grade};
use crate::combat::{CombatState, Event};
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade) {
    let amount = GradeValues { base: 4, plus: 6 }.get(grade);
    state.player.hp.0 = (state.player.hp.0 + amount).min(state.player.max_hp.0);
    events.push(Event::Healed { amount });
}

pub(super) fn def(grade: Grade) -> CardDef {
    let amount = GradeValues { base: 4, plus: 6 }.get(grade);
    CardDef {
        name: "Bandage Up",
        description: CardDescription::WithDamage { template: "Heal {} HP. Exhaust.", base: amount },
        energy_cost: Energy(0),
        card_type: CardType::Skill,
    }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "bandage-up", Grade::Plus => "bandage-up-plus" }
}

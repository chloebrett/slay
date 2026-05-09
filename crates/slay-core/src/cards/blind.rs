use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, Target, apply_status};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize) {
    let weak = GradeValues { base: 2, plus: 2 }.get(grade);
    match grade {
        Grade::Base => {
            apply_status(&mut state.enemies[target].statuses, Target::Enemy, StatusEffect::Weak, weak, events);
        }
        Grade::Plus => {
            for i in 0..state.enemies.len() {
                apply_status(&mut state.enemies[i].statuses, Target::Enemy, StatusEffect::Weak, weak, events);
            }
        }
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Blind",  CardDescription::Static("Apply 2 Weak.")),
        Grade::Plus => ("Blind+", CardDescription::Static("Apply 2 Weak to ALL enemies.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(0), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "blind", Grade::Plus => "blind-plus" }
}

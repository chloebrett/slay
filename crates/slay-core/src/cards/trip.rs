use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, apply_enemy_debuff};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade) {
    let vuln = GradeValues { base: 2, plus: 2 }.get(grade);
    for i in 0..state.enemies.len() {
        apply_enemy_debuff(state, i, StatusEffect::Vulnerable, vuln, events);
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Trip",  CardDescription::Static("Apply 2 Vulnerable to ALL enemies.")),
        Grade::Plus => ("Trip+", CardDescription::Static("Apply 2 Vulnerable to ALL enemies.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(0), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "trip", Grade::Plus => "trip-plus" }
}

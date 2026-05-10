use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, apply_enemy_debuff};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize) {
    let poison = GradeValues { base: 5, plus: 7 }.get(grade);
    apply_enemy_debuff(state, target, StatusEffect::Poison, poison, events);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Deadly Poison",  CardDescription::Static("Apply 5 Poison.")),
        Grade::Plus => ("Deadly Poison+", CardDescription::Static("Apply 7 Poison.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(1), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "deadly-poison", Grade::Plus => "deadly-poison-plus" }
}

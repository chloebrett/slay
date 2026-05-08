use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, apply_status, Target};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, _target: usize) {
    let amount = GradeValues { base: 2, plus: 4 }.get(grade);
    apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Strength, amount, events);
    apply_status(&mut state.player.statuses, Target::Player, StatusEffect::StrengthDown, amount, events);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Flex",  "Gain 2 Strength. At the end of this turn, lose 2 Strength."),
        Grade::Plus => ("Flex+", "Gain 4 Strength. At the end of this turn, lose 4 Strength."),
    };
    CardDef { name, description: CardDescription::Static(desc), energy_cost: Energy(0), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "flex", Grade::Plus => "flex-plus" }
}

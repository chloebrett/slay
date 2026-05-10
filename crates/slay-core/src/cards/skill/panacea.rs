use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, apply_status, Target};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade) {
    let stacks = GradeValues { base: 2, plus: 3 }.get(grade);
    apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Artifact, stacks, events);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Panacea",  CardDescription::Static("Gain 2 Artifact. Exhaust.")),
        Grade::Plus => ("Panacea+", CardDescription::Static("Gain 3 Artifact. Exhaust.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(0), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "panacea", Grade::Plus => "panacea-plus" }
}

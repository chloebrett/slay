use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, apply_status, Target};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, _target: usize) {
    let damage = GradeValues { base: 5, plus: 7 }.get(grade);
    apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Combust, damage, events);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Combust",  "At the end of your turn, lose 1 HP and deal 5 damage to ALL enemies."),
        Grade::Plus => ("Combust+", "At the end of your turn, lose 1 HP and deal 7 damage to ALL enemies."),
    };
    CardDef { name, description: CardDescription::Static(desc), energy_cost: Energy(1), card_type: CardType::Power }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "combust", Grade::Plus => "combust-plus" }
}

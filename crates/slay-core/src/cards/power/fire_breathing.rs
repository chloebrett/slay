use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, apply_status, Target};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, _target: usize) {
    let damage = GradeValues { base: 6, plus: 10 }.get(grade);
    apply_status(&mut state.player.statuses, Target::Player, StatusEffect::FireBreathing, damage, events);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Fire Breathing",  "Whenever you draw a Status or Curse card, deal 6 damage to ALL enemies."),
        Grade::Plus => ("Fire Breathing+", "Whenever you draw a Status or Curse card, deal 10 damage to ALL enemies."),
    };
    CardDef { name, description: CardDescription::Static(desc), energy_cost: Energy(1), card_type: CardType::Power }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "fire-breathing", Grade::Plus => "fire-breathing-plus" }
}

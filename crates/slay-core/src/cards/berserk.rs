use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, apply_status, Target};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, _target: usize) {
    let vuln = match grade { Grade::Base => 2, Grade::Plus => 1 };
    apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Vulnerable, vuln, events);
    apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Berserk, 1, events);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Berserk",  "Gain 2 Vulnerable. NL At the start of your turn, gain 1 Energy."),
        Grade::Plus => ("Berserk+", "Gain 1 Vulnerable. NL At the start of your turn, gain 1 Energy."),
    };
    CardDef { name, description: CardDescription::Static(desc), energy_cost: Energy(0), card_type: CardType::Power }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "berserk", Grade::Plus => "berserk-plus" }
}

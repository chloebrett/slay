use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, Target, apply_status};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, _target: usize) {
    let damage = match grade { Grade::Base => 5, Grade::Plus => 7 };
    apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Juggernaut, damage, events);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Juggernaut",  CardDescription::Static("Whenever you gain Block, deal 5 damage to a random enemy.")),
        Grade::Plus => ("Juggernaut+", CardDescription::Static("Whenever you gain Block, deal 7 damage to a random enemy.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(2), card_type: CardType::Power }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "juggernaut", Grade::Plus => "juggernaut-plus" }
}

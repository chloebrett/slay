use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, damage_all_enemies};
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade) {
    let base_damage = GradeValues { base: 8, plus: 12 }.get(grade);
    damage_all_enemies(&mut state.enemies, events, base_damage);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, base) = match grade {
        Grade::Base => ("Dramatic Entrance",  8),
        Grade::Plus => ("Dramatic Entrance+", 12),
    };
    CardDef {
        name,
        description: CardDescription::WithDamage { template: "Innate. Deal {damage} damage to ALL enemies. Exhaust.", base },
        energy_cost: Energy(0),
        card_type: CardType::Attack,
    }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "dramatic-entrance", Grade::Plus => "dramatic-entrance-plus" }
}

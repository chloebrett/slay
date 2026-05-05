use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, deal_damage};
use crate::status::resolve_damage;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize) {
    let hits = match grade { Grade::Base => 4, Grade::Plus => 5 };
    let raw = resolve_damage(2, &state.player.statuses, &state.enemies[target].statuses);
    for _ in 0..hits {
        let e = &mut state.enemies[target];
        let damage = deal_damage(raw, &mut e.hp, &mut e.block);
        events.push(Event::PlayerAttacked { raw, damage });
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Pummel",  CardDescription::Static("Deal 2 damage 4 times. Exhaust.")),
        Grade::Plus => ("Pummel+", CardDescription::Static("Deal 2 damage 5 times. Exhaust.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(1), card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "pummel", Grade::Plus => "pummel-plus" }
}

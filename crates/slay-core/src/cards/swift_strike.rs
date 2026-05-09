use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, deal_damage};
use crate::status::resolve_damage;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize) {
    let base_damage = GradeValues { base: 7, plus: 10 }.get(grade);
    let raw = resolve_damage(base_damage, &state.player.statuses, &state.enemies[target].statuses);
    let enemy = &mut state.enemies[target];
    let damage = deal_damage(raw, &mut enemy.hp, &mut enemy.block);
    events.push(Event::PlayerAttacked { raw, damage });
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, base) = match grade {
        Grade::Base => ("Swift Strike",  7),
        Grade::Plus => ("Swift Strike+", 10),
    };
    CardDef {
        name,
        description: CardDescription::WithDamage { template: "Deal {damage} damage.", base },
        energy_cost: Energy(0),
        card_type: CardType::Attack,
    }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "swift-strike", Grade::Plus => "swift-strike-plus" }
}

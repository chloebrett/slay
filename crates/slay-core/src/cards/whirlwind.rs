use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, deal_damage};
use crate::status::resolve_damage;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, x_value: i32) {
    let damage_per_hit = GradeValues { base: 5, plus: 8 }.get(grade);
    for _ in 0..x_value {
        for i in 0..state.enemies.len() {
            let raw = resolve_damage(damage_per_hit, &state.player.statuses, &state.enemies[i].statuses);
            let e = &mut state.enemies[i];
            let damage = deal_damage(raw, &mut e.hp, &mut e.block);
            events.push(Event::PlayerAttacked { raw, damage });
        }
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, base) = match grade {
        Grade::Base => ("Whirlwind",  5),
        Grade::Plus => ("Whirlwind+", 8),
    };
    CardDef {
        name,
        description: CardDescription::WithDamage { template: "Deal {damage} damage to ALL enemies X times.", base },
        energy_cost: Energy(0),
        card_type: CardType::Attack,
    }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "whirlwind", Grade::Plus => "whirlwind-plus" }
}

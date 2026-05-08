use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, deal_damage};
use crate::status::resolve_damage;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize) {
    let damage = GradeValues { base: 5, plus: 7 }.get(grade);
    for _ in 0..2 {
        let raw = resolve_damage(damage, &state.player.statuses, &state.enemies[target].statuses);
        let enemy = &mut state.enemies[target];
        let dealt = deal_damage(raw, &mut enemy.hp, &mut enemy.block);
        events.push(Event::PlayerAttacked { raw, damage: dealt });
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, base) = match grade { Grade::Base => ("Twin Strike", 5), Grade::Plus => ("Twin Strike+", 7) };
    CardDef { name, description: CardDescription::WithDamage { template: "Deal {damage} damage twice.", base }, energy_cost: Energy(1), card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "twin-strike", Grade::Plus => "twin-strike-plus" }
}

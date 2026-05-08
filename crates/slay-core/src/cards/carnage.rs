use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, deal_damage};
use crate::status::resolve_damage;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize) {
    let base = GradeValues { base: 20, plus: 28 }.get(grade);
    let raw = resolve_damage(base, &state.player.statuses, &state.enemies[target].statuses);
    let enemy = &mut state.enemies[target];
    let damage = deal_damage(raw, &mut enemy.hp, &mut enemy.block);
    events.push(Event::PlayerAttacked { raw, damage });
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, base) = match grade {
        Grade::Base => ("Carnage", 20),
        Grade::Plus => ("Carnage+", 28),
    };
    CardDef { name, description: CardDescription::WithDamage { template: "Deal {damage} damage. Ethereal. Exhaust.", base }, energy_cost: Energy(2), card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "carnage", Grade::Plus => "carnage-plus" }
}

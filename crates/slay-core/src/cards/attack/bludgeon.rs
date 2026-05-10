use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, deal_damage};
use crate::types::Energy;

pub(super) fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize) {
    let base_damage = GradeValues { base: 32, plus: 42 }.get(grade);
    let raw = crate::status::resolve_damage(base_damage, &state.player.statuses, &state.enemies[target].statuses);
    let damage = { let e = &mut state.enemies[target]; deal_damage(raw, &mut e.hp, &mut e.block) };
    events.push(Event::PlayerAttacked { raw, damage });
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, base) = match grade { Grade::Base => ("Bludgeon", 32), Grade::Plus => ("Bludgeon+", 42) };
    CardDef { name, description: CardDescription::WithDamage { template: "Deal {damage} damage.", base }, energy_cost: Energy(3), card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "bludgeon", Grade::Plus => "bludgeon-plus" }
}

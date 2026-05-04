use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, deal_damage};
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize) {
    let base_damage = match grade { Grade::Base => 6, Grade::Plus => 9 };
    let raw = crate::status::resolve_damage(base_damage, &state.player.statuses, &state.enemies[target].statuses);
    let enemy = &mut state.enemies[target];
    let damage = deal_damage(raw, &mut enemy.hp, &mut enemy.block);
    events.push(Event::PlayerAttacked { raw, damage });
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, base) = match grade {
        Grade::Base => ("Strike", 6),
        Grade::Plus => ("Strike+", 9),
    };
    CardDef { name, description: CardDescription::WithDamage { template: "Deal {damage} damage.", base }, energy_cost: Energy(1), card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "strike", Grade::Plus => "strike-plus" }
}

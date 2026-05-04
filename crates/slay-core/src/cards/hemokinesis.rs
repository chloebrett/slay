use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, damage_player, deal_damage};
use crate::status::resolve_damage;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize) {
    let base_damage = match grade { Grade::Base => 15, Grade::Plus => 20 };
    damage_player(state, events, 2);
    let raw = resolve_damage(base_damage, &state.player.statuses, &state.enemies[target].statuses);
    let dealt = { let e = &mut state.enemies[target]; deal_damage(raw, &mut e.hp, &mut e.block) };
    events.push(Event::PlayerAttacked { raw, damage: dealt });
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, base) = match grade { Grade::Base => ("Hemokinesis", 15), Grade::Plus => ("Hemokinesis+", 20) };
    CardDef { name, description: CardDescription::WithDamage { template: "Lose 2 HP. Deal {damage} damage.", base }, energy_cost: Energy(1), card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "hemokinesis", Grade::Plus => "hemokinesis-plus" }
}

use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, damage_player, deal_damage};
use crate::status::resolve_damage;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, hp_loss: i32, base_damage: i32) {
    damage_player(state, events, hp_loss);
    for i in 0..state.enemies.len() {
        let raw = resolve_damage(base_damage, &state.player.statuses, &state.enemies[i].statuses);
        let enemy = &mut state.enemies[i];
        let damage = deal_damage(raw, &mut enemy.hp, &mut enemy.block);
        events.push(Event::PlayerAttacked { raw, damage });
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, base) = match grade { Grade::Base => ("Breakthrough", 9), Grade::Plus => ("Breakthrough+", 13) };
    CardDef { name, description: CardDescription::WithDamage { template: "Lose 1 HP. Deal {damage} damage to ALL enemies.", base }, energy_cost: Energy(1), card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "breakthrough", Grade::Plus => "breakthrough-plus" }
}

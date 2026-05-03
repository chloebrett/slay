use crate::combat::{CombatState, Event, deal_damage};
use crate::status::resolve_damage;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, base_damage: i32) {
    for i in 0..state.enemies.len() {
        let raw = resolve_damage(base_damage, &state.player.statuses, &state.enemies[i].statuses);
        let enemy = &mut state.enemies[i];
        let damage = deal_damage(raw, &mut enemy.hp, &mut enemy.block);
        events.push(Event::PlayerAttacked { raw, damage });
    }
}

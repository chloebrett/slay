use crate::combat::{CombatState, Event, deal_damage};
use crate::status::resolve_damage;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, damage: i32, target: usize) {
    for _ in 0..2 {
        let raw = resolve_damage(damage, &state.player.statuses, &state.enemies[target].statuses);
        let enemy = &mut state.enemies[target];
        let dealt = deal_damage(raw, &mut enemy.hp, &mut enemy.block);
        events.push(Event::PlayerAttacked { raw, damage: dealt });
    }
}

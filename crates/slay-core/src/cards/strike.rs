use crate::combat::{CombatState, Event, deal_damage};

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, base_damage: i32, target: usize) {
    let raw = crate::status::resolve_damage(base_damage, &state.player.statuses, &state.enemies[target].statuses);
    let enemy = &mut state.enemies[target];
    let damage = deal_damage(raw, &mut enemy.hp, &mut enemy.block);
    events.push(Event::PlayerAttacked { raw, damage });
}

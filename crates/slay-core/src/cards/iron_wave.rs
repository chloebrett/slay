use crate::combat::{CombatState, Event, deal_damage};
use crate::status::resolve_damage;
use crate::types::Block;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, damage: i32, block: i32, target: usize) {
    state.player.block = Block(state.player.block.0 + block);
    events.push(Event::PlayerBlocked { amount: block });
    let raw = resolve_damage(damage, &state.player.statuses, &state.enemies[target].statuses);
    let enemy = &mut state.enemies[target];
    let dealt = deal_damage(raw, &mut enemy.hp, &mut enemy.block);
    events.push(Event::PlayerAttacked { raw, damage: dealt });
}

use crate::cards::Card;
use crate::combat::{CombatState, Event, deal_damage};
use crate::status::resolve_damage;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, damage: i32, copy: Card, target: usize) {
    let raw = resolve_damage(damage, &state.player.statuses, &state.enemies[target].statuses);
    let dealt = { let e = &mut state.enemies[target]; deal_damage(raw, &mut e.hp, &mut e.block) };
    events.push(Event::PlayerAttacked { raw, damage: dealt });
    state.player.discard_pile.push(copy);
}

use crate::combat::{CombatState, Event, deal_damage};

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>) {
    let raw = crate::status::resolve_damage(6, &state.player.statuses, &state.enemy.statuses);
    let damage = deal_damage(raw, &mut state.enemy.hp, &mut state.enemy.block);
    events.push(Event::PlayerAttacked { raw, damage });
}

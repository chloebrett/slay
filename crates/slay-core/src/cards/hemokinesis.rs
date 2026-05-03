use crate::combat::{CombatState, Event, damage_player, deal_damage};
use crate::status::resolve_damage;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, hp_loss: i32, base_damage: i32, target: usize) {
    damage_player(state, events, hp_loss);
    let raw = resolve_damage(base_damage, &state.player.statuses, &state.enemies[target].statuses);
    let dealt = { let e = &mut state.enemies[target]; deal_damage(raw, &mut e.hp, &mut e.block) };
    events.push(Event::PlayerAttacked { raw, damage: dealt });
}

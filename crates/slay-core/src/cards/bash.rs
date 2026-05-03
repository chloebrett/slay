use crate::combat::{CombatState, Event, Target, apply_status, deal_damage};
use crate::status::StatusEffect;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, damage: i32, vuln: i32) {
    let raw = crate::status::resolve_damage(damage, &state.player.statuses, &state.enemy.statuses);
    let dealt = deal_damage(raw, &mut state.enemy.hp, &mut state.enemy.block);
    events.push(Event::PlayerAttacked { raw, damage: dealt });
    apply_status(&mut state.enemy.statuses, Target::Enemy, StatusEffect::Vulnerable, vuln, events);
}

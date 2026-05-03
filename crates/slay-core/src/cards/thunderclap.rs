use crate::combat::{CombatState, Event, Target, apply_status, deal_damage};
use crate::status::{StatusEffect, resolve_damage};

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, damage: i32, vuln: i32) {
    for i in 0..state.enemies.len() {
        let raw = resolve_damage(damage, &state.player.statuses, &state.enemies[i].statuses);
        let dealt = { let e = &mut state.enemies[i]; deal_damage(raw, &mut e.hp, &mut e.block) };
        events.push(Event::PlayerAttacked { raw, damage: dealt });
    }
    for i in 0..state.enemies.len() {
        apply_status(&mut state.enemies[i].statuses, Target::Enemy, StatusEffect::Vulnerable, vuln, events);
    }
}

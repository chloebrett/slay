use crate::combat::{CombatState, Event, Target, apply_status, deal_damage};
use crate::status::StatusEffect;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, damage: i32, weak: i32, target: usize) {
    let raw = crate::status::resolve_damage(damage, &state.player.statuses, &state.enemies[target].statuses);
    let dealt = {
        let enemy = &mut state.enemies[target];
        deal_damage(raw, &mut enemy.hp, &mut enemy.block)
    };
    events.push(Event::PlayerAttacked { raw, damage: dealt });
    apply_status(&mut state.enemies[target].statuses, Target::Enemy, StatusEffect::Weak, weak, events);
}

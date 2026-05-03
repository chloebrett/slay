use crate::combat::{CombatState, Event, Target, apply_status, deal_damage};
use crate::status::StatusEffect;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>) {
    let raw = crate::status::resolve_damage(12, &state.player.statuses, &state.enemy.statuses);
    let damage = deal_damage(raw, &mut state.enemy.hp, &mut state.enemy.block);
    events.push(Event::PlayerAttacked { raw, damage });
    apply_status(&mut state.enemy.statuses, Target::Enemy, StatusEffect::Weak, 2, events);
}

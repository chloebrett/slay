use crate::combat::{CombatState, Event, Target, deal_damage};
use crate::status::StatusEffect;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>) {
    let raw = crate::status::resolve_damage(12, &state.player.statuses, &state.enemy.statuses);
    let damage = deal_damage(raw, &mut state.enemy.hp, &mut state.enemy.block);
    events.push(Event::PlayerAttacked { raw, damage });
    *state.enemy.statuses.entry(StatusEffect::Weak).or_insert(0) += 2;
    events.push(Event::StatusApplied { target: Target::Enemy, status: StatusEffect::Weak, stacks: 2 });
}

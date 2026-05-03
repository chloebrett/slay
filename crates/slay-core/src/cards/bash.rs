use crate::combat::{CombatPhase, CombatState, Event, Target, deal_damage};
use crate::status::StatusEffect;
use crate::types::Hp;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>) {
    let raw = crate::status::resolve_damage(8, &state.player.statuses, &state.enemy.statuses);
    let damage = deal_damage(raw, &mut state.enemy.hp, &mut state.enemy.block);
    events.push(Event::PlayerAttacked { raw, damage });
    *state.enemy.statuses.entry(StatusEffect::Vulnerable).or_insert(0) += 2;
    events.push(Event::StatusApplied { target: Target::Enemy, status: StatusEffect::Vulnerable, stacks: 2 });
    if state.enemy.hp <= Hp(0) {
        state.phase = CombatPhase::Victory;
        events.push(Event::EnemyDied);
    }
}

use crate::combat::{CombatPhase, CombatState, Event, deal_damage};
use crate::types::Hp;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>) {
    let damage = deal_damage(6, &mut state.enemy.hp, &mut state.enemy.block);
    events.push(Event::PlayerAttacked { damage });
    if state.enemy.hp <= Hp(0) {
        state.phase = CombatPhase::Victory;
        events.push(Event::EnemyDied);
    }
}

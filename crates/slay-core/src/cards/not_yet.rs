use crate::combat::{CombatState, Event};
use crate::types::Hp;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, amount: i32, _target: usize) {
    let new_hp = (state.player.hp.0 + amount).min(state.player.max_hp.0);
    state.player.hp = Hp(new_hp);
    events.push(Event::Healed { amount });
}

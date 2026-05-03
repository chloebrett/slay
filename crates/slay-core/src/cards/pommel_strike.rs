use crate::combat::{CombatState, Event, deal_damage, draw_cards};
use crate::rng::Rng;
use crate::status::resolve_damage;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, base_damage: i32, draws: usize, target: usize, rng: &mut impl Rng) {
    let raw = resolve_damage(base_damage, &state.player.statuses, &state.enemies[target].statuses);
    let dealt = { let e = &mut state.enemies[target]; deal_damage(raw, &mut e.hp, &mut e.block) };
    events.push(Event::PlayerAttacked { raw, damage: dealt });
    draw_cards(&mut state.player, draws, rng);
    events.push(Event::CardsDrawn { count: draws });
}

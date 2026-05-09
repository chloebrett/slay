use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, deal_damage};
use crate::status::resolve_damage;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, target: usize) {
    let base = state.player.draw_pile.len() as i32;
    let raw = resolve_damage(base, &state.player.statuses, &state.enemies[target].statuses);
    let dealt = { let e = &mut state.enemies[target]; deal_damage(raw, &mut e.hp, &mut e.block) };
    events.push(Event::PlayerAttacked { raw, damage: dealt });
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, cost) = match grade {
        Grade::Base => ("Mind Blast",  Energy(2)),
        Grade::Plus => ("Mind Blast+", Energy(1)),
    };
    CardDef { name, description: CardDescription::Static("Innate. Deal damage equal to the size of your draw pile."), energy_cost: cost, card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "mind-blast", Grade::Plus => "mind-blast-plus" }
}

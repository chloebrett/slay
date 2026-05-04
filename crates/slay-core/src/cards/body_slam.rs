use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, deal_damage};
use crate::status::resolve_damage;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, target: usize) {
    let damage = state.player.block.0;
    let raw = resolve_damage(damage, &state.player.statuses, &state.enemies[target].statuses);
    let dealt = { let e = &mut state.enemies[target]; deal_damage(raw, &mut e.hp, &mut e.block) };
    events.push(Event::PlayerAttacked { raw, damage: dealt });
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, cost) = match grade { Grade::Base => ("Body Slam", Energy(1)), Grade::Plus => ("Body Slam+", Energy(0)) };
    CardDef { name, description: CardDescription::Static("Deal damage equal to your Block."), energy_cost: cost, card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "body-slam", Grade::Plus => "body-slam-plus" }
}

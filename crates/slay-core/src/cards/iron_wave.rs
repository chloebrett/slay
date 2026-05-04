use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, deal_damage};
use crate::status::resolve_damage;
use crate::types::{Block, Energy};

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize) {
    let n = match grade { Grade::Base => 5, Grade::Plus => 7 };
    state.player.block = Block(state.player.block.0 + n);
    events.push(Event::PlayerBlocked { amount: n });
    let raw = resolve_damage(n, &state.player.statuses, &state.enemies[target].statuses);
    let dealt = { let e = &mut state.enemies[target]; deal_damage(raw, &mut e.hp, &mut e.block) };
    events.push(Event::PlayerAttacked { raw, damage: dealt });
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, base, template) = match grade {
        Grade::Base => ("Iron Wave",  5, "Gain 5 Block. Deal {damage} damage."),
        Grade::Plus => ("Iron Wave+", 7, "Gain 7 Block. Deal {damage} damage."),
    };
    CardDef { name, description: CardDescription::WithDamage { template, base }, energy_cost: Energy(1), card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "iron-wave", Grade::Plus => "iron-wave-plus" }
}

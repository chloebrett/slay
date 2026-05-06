use super::{Card, CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, deal_damage};
use crate::rng::Rng;
use crate::status::resolve_damage;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize, rng: &mut impl Rng) {
    let base = match grade { Grade::Base => 12, Grade::Plus => 17 };
    let raw = resolve_damage(base, &state.player.statuses, &state.enemies[target].statuses);
    let dealt = { let e = &mut state.enemies[target]; deal_damage(raw, &mut e.hp, &mut e.block) };
    events.push(Event::PlayerAttacked { raw, damage: dealt });
    state.player.draw_pile.push(Card::Wound);
    rng.shuffle(&mut state.player.draw_pile);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, base) = match grade {
        Grade::Base => ("Wild Strike", 12),
        Grade::Plus => ("Wild Strike+", 17),
    };
    CardDef { name, description: CardDescription::WithDamage { template: "Deal {damage} damage. Shuffle a Wound into your draw pile.", base }, energy_cost: Energy(1), card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "wild-strike", Grade::Plus => "wild-strike-plus" }
}

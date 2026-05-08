use super::{Card, CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, deal_damage};
use crate::rng::Rng;
use crate::status::resolve_damage;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize, rng: &mut impl Rng) {
    let damage = GradeValues { base: 7, plus: 10 }.get(grade);
    let raw = resolve_damage(damage, &state.player.statuses, &state.enemies[target].statuses);
    let dealt = { let e = &mut state.enemies[target]; deal_damage(raw, &mut e.hp, &mut e.block) };
    events.push(Event::PlayerAttacked { raw, damage: dealt });
    state.player.draw_pile.push(Card::Dazed);
    rng.shuffle(&mut state.player.draw_pile);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, base) = match grade { Grade::Base => ("Reckless Charge", 7), Grade::Plus => ("Reckless Charge+", 10) };
    CardDef { name, description: CardDescription::WithDamage { template: "Deal {damage} damage. Shuffle a Dazed into your draw pile.", base }, energy_cost: Energy(0), card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "reckless-charge", Grade::Plus => "reckless-charge-plus" }
}

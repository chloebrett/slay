use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, deal_damage, exhaust_card};
use crate::rng::Rng;
use crate::status::resolve_damage;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize, rng: &mut impl Rng) {
    let damage_per_card = GradeValues { base: 7, plus: 10 }.get(grade);
    let hand: Vec<_> = state.player.hand.drain(..).collect();
    let count = hand.len();
    for card in hand {
        exhaust_card(card, state, events, rng);
    }
    for _ in 0..count {
        let raw = resolve_damage(damage_per_card, &state.player.statuses, &state.enemies[target].statuses);
        let e = &mut state.enemies[target];
        let damage = deal_damage(raw, &mut e.hp, &mut e.block);
        events.push(Event::PlayerAttacked { raw, damage });
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, base) = match grade {
        Grade::Base => ("Fiend Fire",  7),
        Grade::Plus => ("Fiend Fire+", 10),
    };
    CardDef {
        name,
        description: CardDescription::WithDamage { template: "Exhaust your hand. Deal {damage} damage for each card Exhausted. Exhaust.", base },
        energy_cost: Energy(2),
        card_type: CardType::Attack,
    }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "fiend-fire", Grade::Plus => "fiend-fire-plus" }
}

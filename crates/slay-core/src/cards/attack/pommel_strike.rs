use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, deal_damage, draw_cards};
use crate::rng::Rng;
use crate::status::resolve_damage;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize, rng: &mut impl Rng) {
    let (base_damage, draws) = match grade { Grade::Base => (9, 1usize), Grade::Plus => (10, 2usize) };
    let raw = resolve_damage(base_damage, &state.player.statuses, &state.enemies[target].statuses);
    let dealt = { let e = &mut state.enemies[target]; deal_damage(raw, &mut e.hp, &mut e.block) };
    events.push(Event::PlayerAttacked { raw, damage: dealt });
    draw_cards(&mut state.player, draws, rng);
    events.push(Event::CardsDrawn { count: draws });
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, base, template) = match grade {
        Grade::Base => ("Pommel Strike",  9,  "Deal {damage} damage. Draw 1 card."),
        Grade::Plus => ("Pommel Strike+", 10, "Deal {damage} damage. Draw 2 cards."),
    };
    CardDef { name, description: CardDescription::WithDamage { template, base }, energy_cost: Energy(1), card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "pommel-strike", Grade::Plus => "pommel-strike-plus" }
}

use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, deal_damage, draw_cards};
use crate::rng::Rng;
use crate::status::resolve_damage;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize, rng: &mut impl Rng) {
    let base_damage = GradeValues { base: 3, plus: 6 }.get(grade);
    let raw = resolve_damage(base_damage, &state.player.statuses, &state.enemies[target].statuses);
    let dealt = { let e = &mut state.enemies[target]; deal_damage(raw, &mut e.hp, &mut e.block) };
    events.push(Event::PlayerAttacked { raw, damage: dealt });
    draw_cards(&mut state.player, 1, rng);
    events.push(Event::CardsDrawn { count: 1 });
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, base) = match grade {
        Grade::Base => ("Flash of Steel",  3),
        Grade::Plus => ("Flash of Steel+", 6),
    };
    CardDef {
        name,
        description: CardDescription::WithDamage { template: "Deal {damage} damage. Draw 1 card.", base },
        energy_cost: Energy(0),
        card_type: CardType::Attack,
    }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "flash-of-steel", Grade::Plus => "flash-of-steel-plus" }
}

use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, deal_damage, gain_player_block};
use crate::rng::Rng;
use crate::status::resolve_damage;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize, rng: &mut impl Rng) {
    let n = GradeValues { base: 5, plus: 7 }.get(grade);
    gain_player_block(state, events, n, rng);
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

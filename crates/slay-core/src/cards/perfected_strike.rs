use super::{Card, CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, deal_damage};
use crate::status::resolve_damage;
use crate::types::Energy;

fn count_strike_cards(state: &CombatState, self_card: Card) -> i32 {
    let piles = [&state.player.hand, &state.player.draw_pile, &state.player.discard_pile];
    let pile_count = piles
        .iter()
        .flat_map(|p| p.iter())
        .filter(|c| c.def().name.contains("Strike"))
        .count() as i32;
    let self_count = if self_card.def().name.contains("Strike") { 1 } else { 0 };
    pile_count + self_count
}

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize) {
    let bonus_per_strike = GradeValues { base: 2, plus: 3 }.get(grade);
    let strike_count = count_strike_cards(state, Card::PerfectedStrike(grade));
    let base = 6 + bonus_per_strike * strike_count;
    let raw = resolve_damage(base, &state.player.statuses, &state.enemies[target].statuses);
    let e = &mut state.enemies[target];
    let damage = deal_damage(raw, &mut e.hp, &mut e.block);
    events.push(Event::PlayerAttacked { raw, damage });
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, description) = match grade {
        Grade::Base => ("Perfected Strike",  CardDescription::Static("Deal 6 damage. Deals 2 additional damage for ALL your cards containing \"Strike\".")),
        Grade::Plus => ("Perfected Strike+", CardDescription::Static("Deal 6 damage. Deals 3 additional damage for ALL your cards containing \"Strike\".")),
    };
    CardDef { name, description, energy_cost: Energy(2), card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "perfected-strike", Grade::Plus => "perfected-strike-plus" }
}

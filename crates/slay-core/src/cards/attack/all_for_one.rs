use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, deal_damage};
use crate::status::resolve_damage;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize) {
    let base_damage = GradeValues { base: 10, plus: 14 }.get(grade);
    let raw = resolve_damage(base_damage, &state.player.statuses, &state.enemies[target].statuses);
    let e = &mut state.enemies[target];
    let damage = deal_damage(raw, &mut e.hp, &mut e.block);
    events.push(Event::PlayerAttacked { raw, damage });
    let zero_cost: Vec<_> = state.player.discard_pile.iter()
        .filter(|c| c.energy_cost().0 == 0)
        .cloned()
        .collect();
    state.player.discard_pile.retain(|c| c.energy_cost().0 != 0);
    state.player.hand.extend(zero_cost);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, description) = match grade {
        Grade::Base => ("All for One",  CardDescription::Static("Deal 10 damage. Put all cost-0 cards from your discard pile into your hand.")),
        Grade::Plus => ("All for One+", CardDescription::Static("Deal 14 damage. Put all cost-0 cards from your discard pile into your hand.")),
    };
    CardDef { name, description, energy_cost: Energy(1), card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "all-for-one", Grade::Plus => "all-for-one-plus" }
}

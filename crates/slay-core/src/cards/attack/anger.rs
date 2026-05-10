use super::{Card, CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, deal_damage};
use crate::status::resolve_damage;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize) {
    let damage = GradeValues { base: 6, plus: 8 }.get(grade);
    let raw = resolve_damage(damage, &state.player.statuses, &state.enemies[target].statuses);
    let dealt = { let e = &mut state.enemies[target]; deal_damage(raw, &mut e.hp, &mut e.block) };
    events.push(Event::PlayerAttacked { raw, damage: dealt });
    state.player.discard_pile.push(Card::Anger(grade));
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, base) = match grade { Grade::Base => ("Anger", 6), Grade::Plus => ("Anger+", 8) };
    CardDef { name, description: CardDescription::WithDamage { template: "Deal {damage} damage. Add a copy to your Discard Pile.", base }, energy_cost: Energy(0), card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "anger", Grade::Plus => "anger-plus" }
}

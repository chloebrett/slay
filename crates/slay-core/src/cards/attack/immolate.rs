use super::{Card, CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, deal_damage};
use crate::status::resolve_damage;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, _target: usize) {
    let base_damage = GradeValues { base: 21, plus: 28 }.get(grade);
    for i in 0..state.enemies.len() {
        let raw = resolve_damage(base_damage, &state.player.statuses, &state.enemies[i].statuses);
        let enemy = &mut state.enemies[i];
        let damage = deal_damage(raw, &mut enemy.hp, &mut enemy.block);
        events.push(Event::PlayerAttacked { raw, damage });
    }
    state.player.discard_pile.push(Card::Burn);
    events.push(Event::StatusCardAddedToDiscard { card: Card::Burn });
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, base) = match grade {
        Grade::Base => ("Immolate",  21),
        Grade::Plus => ("Immolate+", 28),
    };
    CardDef {
        name,
        description: CardDescription::WithDamage { template: "Deal {damage} damage to ALL enemies. Add a Burn into your discard pile.", base },
        energy_cost: Energy(2),
        card_type: CardType::Attack,
    }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "immolate", Grade::Plus => "immolate-plus" }
}

use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, deal_damage};
use crate::status::resolve_damage;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, _target: usize) {
    let base_damage = GradeValues { base: 4, plus: 5 }.get(grade);
    let mut total_heal = 0;
    for i in 0..state.enemies.len() {
        let raw = resolve_damage(base_damage, &state.player.statuses, &state.enemies[i].statuses);
        let e = &mut state.enemies[i];
        let damage = deal_damage(raw, &mut e.hp, &mut e.block);
        events.push(Event::PlayerAttacked { raw, damage });
        total_heal += damage;
    }
    if total_heal > 0 {
        state.player.hp.0 = (state.player.hp.0 + total_heal).min(state.player.max_hp.0);
        events.push(Event::Healed { amount: total_heal });
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, base) = match grade {
        Grade::Base => ("Reaper",  4),
        Grade::Plus => ("Reaper+", 5),
    };
    CardDef {
        name,
        description: CardDescription::WithDamage { template: "Deal {damage} damage to ALL enemies. Heal HP equal to unblocked damage. Exhaust.", base },
        energy_cost: Energy(2),
        card_type: CardType::Attack,
    }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "reaper", Grade::Plus => "reaper-plus" }
}

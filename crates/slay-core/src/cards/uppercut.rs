use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, Target, apply_status, deal_damage};
use crate::status::{StatusEffect, resolve_damage};
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize) {
    let damage = 13;
    let (weak, vuln) = match grade { Grade::Base => (1, 1), Grade::Plus => (2, 2) };
    let raw = resolve_damage(damage, &state.player.statuses, &state.enemies[target].statuses);
    let dealt = { let e = &mut state.enemies[target]; deal_damage(raw, &mut e.hp, &mut e.block) };
    events.push(Event::PlayerAttacked { raw, damage: dealt });
    apply_status(&mut state.enemies[target].statuses, Target::Enemy, StatusEffect::Weak, weak, events);
    apply_status(&mut state.enemies[target].statuses, Target::Enemy, StatusEffect::Vulnerable, vuln, events);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, template) = match grade {
        Grade::Base => ("Uppercut",  "Deal {damage} damage. Apply 1 Weak. Apply 1 Vulnerable."),
        Grade::Plus => ("Uppercut+", "Deal {damage} damage. Apply 2 Weak. Apply 2 Vulnerable."),
    };
    CardDef { name, description: CardDescription::WithDamage { template, base: 13 }, energy_cost: Energy(2), card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "uppercut", Grade::Plus => "uppercut-plus" }
}

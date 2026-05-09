use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, apply_enemy_debuff, deal_damage};
use crate::status::{StatusEffect, resolve_damage};
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade) {
    let damage = GradeValues { base: 4, plus: 7 }.get(grade);
    let vuln = 1;
    for i in 0..state.enemies.len() {
        let raw = resolve_damage(damage, &state.player.statuses, &state.enemies[i].statuses);
        let dealt = { let e = &mut state.enemies[i]; deal_damage(raw, &mut e.hp, &mut e.block) };
        events.push(Event::PlayerAttacked { raw, damage: dealt });
    }
    for i in 0..state.enemies.len() {
        apply_enemy_debuff(state, i, StatusEffect::Vulnerable, vuln, events);
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, base) = match grade { Grade::Base => ("Thunderclap", 4), Grade::Plus => ("Thunderclap+", 7) };
    CardDef { name, description: CardDescription::WithDamage { template: "Deal {damage} damage and apply 1 Vulnerable to ALL enemies.", base }, energy_cost: Energy(1), card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "thunderclap", Grade::Plus => "thunderclap-plus" }
}

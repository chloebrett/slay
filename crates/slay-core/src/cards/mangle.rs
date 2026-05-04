use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, Target, apply_status, deal_damage};
use crate::status::{StatusEffect, resolve_damage};
use crate::types::Energy;

// NOTE: The original card reduces Strength "this turn" only (ephemeral), but we apply it
// permanently because we have no ephemeral status system yet. Behaviour diverges accordingly.
pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize) {
    let (damage, strength_loss) = match grade { Grade::Base => (15, 10), Grade::Plus => (20, 15) };
    let raw = resolve_damage(damage, &state.player.statuses, &state.enemies[target].statuses);
    let dealt = { let e = &mut state.enemies[target]; deal_damage(raw, &mut e.hp, &mut e.block) };
    events.push(Event::PlayerAttacked { raw, damage: dealt });
    apply_status(&mut state.enemies[target].statuses, Target::Enemy, StatusEffect::Strength, -strength_loss, events);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, base, template) = match grade {
        Grade::Base => ("Mangle",  15, "Deal {damage} damage. Enemy loses 10 Strength."),
        Grade::Plus => ("Mangle+", 20, "Deal {damage} damage. Enemy loses 15 Strength."),
    };
    CardDef { name, description: CardDescription::WithDamage { template, base }, energy_cost: Energy(3), card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "mangle", Grade::Plus => "mangle-plus" }
}

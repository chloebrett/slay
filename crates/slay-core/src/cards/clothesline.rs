use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, apply_enemy_debuff, deal_damage};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize) {
    let (damage, weak) = match grade { Grade::Base => (12, 2), Grade::Plus => (14, 3) };
    let raw = crate::status::resolve_damage(damage, &state.player.statuses, &state.enemies[target].statuses);
    let dealt = {
        let enemy = &mut state.enemies[target];
        deal_damage(raw, &mut enemy.hp, &mut enemy.block)
    };
    events.push(Event::PlayerAttacked { raw, damage: dealt });
    apply_enemy_debuff(state, target, StatusEffect::Weak, weak, events);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, base, template) = match grade {
        Grade::Base => ("Clothesline",  12, "Deal {damage} damage. Apply 2 Weak."),
        Grade::Plus => ("Clothesline+", 14, "Deal {damage} damage. Apply 3 Weak."),
    };
    CardDef { name, description: CardDescription::WithDamage { template, base }, energy_cost: Energy(2), card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "clothesline", Grade::Plus => "clothesline-plus" }
}

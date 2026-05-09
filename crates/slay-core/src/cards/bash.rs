use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, apply_enemy_debuff, deal_damage};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize) {
    let (damage, vuln) = match grade { Grade::Base => (8, 2), Grade::Plus => (10, 3) };
    let raw = crate::status::resolve_damage(damage, &state.player.statuses, &state.enemies[target].statuses);
    let dealt = {
        let enemy = &mut state.enemies[target];
        deal_damage(raw, &mut enemy.hp, &mut enemy.block)
    };
    events.push(Event::PlayerAttacked { raw, damage: dealt });
    apply_enemy_debuff(state, target, StatusEffect::Vulnerable, vuln, events);
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, base, template) = match grade {
        Grade::Base => ("Bash",  8, "Deal {damage} damage. Apply 2 Vulnerable."),
        Grade::Plus => ("Bash+", 10, "Deal {damage} damage. Apply 3 Vulnerable."),
    };
    CardDef { name, description: CardDescription::WithDamage { template, base }, energy_cost: Energy(2), card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "bash", Grade::Plus => "bash-plus" }
}

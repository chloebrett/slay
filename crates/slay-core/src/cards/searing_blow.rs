use super::{CardDef, CardDescription, CardType};
use crate::combat::{CombatState, Event, deal_damage};
use crate::status::resolve_damage;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, upgrades: u32, target: usize) {
    let n = upgrades as i32;
    let base = 12 + n * (n + 7) / 2;
    let raw = resolve_damage(base, &state.player.statuses, &state.enemies[target].statuses);
    let e = &mut state.enemies[target];
    let damage = deal_damage(raw, &mut e.hp, &mut e.block);
    events.push(Event::PlayerAttacked { raw, damage });
}

pub(super) fn def(upgrades: u32) -> CardDef {
    let name = if upgrades == 0 { "Searing Blow" } else { "Searing Blow+" };
    let n = upgrades as i32;
    let base = 12 + n * (n + 7) / 2;
    CardDef {
        name,
        description: CardDescription::WithDamage { template: "Deal {damage} damage.", base },
        energy_cost: Energy(2),
        card_type: CardType::Attack,
    }
}

pub(super) fn id() -> &'static str {
    "searing-blow"
}

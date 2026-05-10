use super::{CardDef, CardDescription, CardType, Card};
use crate::combat::{CombatState, Event, deal_damage};
use crate::status::resolve_damage;
use crate::types::{Energy, Hp};

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, bonus: u32, target: usize) {
    let base = 15 + bonus as i32;
    let raw = resolve_damage(base, &state.player.statuses, &state.enemies[target].statuses);
    let enemy = &mut state.enemies[target];
    let dealt = deal_damage(raw, &mut enemy.hp, &mut enemy.block);
    events.push(Event::PlayerAttacked { raw, damage: dealt });
    if state.enemies[target].hp <= Hp(0) {
        let new_bonus = bonus + 3;
        for card in state.player.deck.iter_mut() {
            if *card == Card::RitualDagger(bonus) {
                *card = Card::RitualDagger(new_bonus);
                break;
            }
        }
    }
}

pub(super) fn def(bonus: u32) -> CardDef {
    let damage = 15 + bonus as i32;
    CardDef {
        name: "Ritual Dagger",
        description: CardDescription::WithDamage { template: "Deal {damage} damage. If this kills a non-minion, permanently increase this card's damage by 3. Exhaust.", base: damage },
        energy_cost: Energy(1),
        card_type: CardType::Attack,
    }
}

pub(super) fn id() -> &'static str {
    "ritual-dagger"
}

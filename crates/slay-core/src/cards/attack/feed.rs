use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, deal_damage};
use crate::status::resolve_damage;
use crate::types::{Energy, Hp};

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize) {
    let hp_bonus = GradeValues { base: 3, plus: 4 }.get(grade);
    let raw = resolve_damage(10, &state.player.statuses, &state.enemies[target].statuses);
    let e = &mut state.enemies[target];
    let damage = deal_damage(raw, &mut e.hp, &mut e.block);
    events.push(Event::PlayerAttacked { raw, damage });
    if state.enemies[target].hp <= Hp(0) {
        state.player.max_hp.0 += hp_bonus;
        state.player.hp.0 += hp_bonus;
        events.push(Event::MaxHpIncreased { amount: hp_bonus });
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, description) = match grade {
        Grade::Base => ("Feed",  CardDescription::Static("Deal 10 damage. If Fatal, raise your Max HP by 3. Exhaust.")),
        Grade::Plus => ("Feed+", CardDescription::Static("Deal 10 damage. If Fatal, raise your Max HP by 4. Exhaust.")),
    };
    CardDef { name, description, energy_cost: Energy(1), card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "feed", Grade::Plus => "feed-plus" }
}

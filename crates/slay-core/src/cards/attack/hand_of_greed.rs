use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, deal_damage};
use crate::status::resolve_damage;
use crate::types::{Energy, Hp};

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize) {
    let base = GradeValues { base: 20, plus: 25 }.get(grade);
    let raw = resolve_damage(base, &state.player.statuses, &state.enemies[target].statuses);
    let enemy = &mut state.enemies[target];
    let dealt = deal_damage(raw, &mut enemy.hp, &mut enemy.block);
    events.push(Event::PlayerAttacked { raw, damage: dealt });
    if state.enemies[target].hp <= Hp(0) {
        state.player.gold += 20;
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Hand of Greed",  CardDescription::Static("Deal 20 damage. If this kills a non-minion, gain 20 Gold.")),
        Grade::Plus => ("Hand of Greed+", CardDescription::Static("Deal 25 damage. If this kills a non-minion, gain 25 Gold.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(2), card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "hand-of-greed", Grade::Plus => "hand-of-greed-plus" }
}

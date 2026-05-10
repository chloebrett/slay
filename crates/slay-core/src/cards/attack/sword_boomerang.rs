use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, deal_damage};
use crate::rng::Rng;
use crate::status::resolve_damage;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, rng: &mut impl Rng) {
    let hits = GradeValues { base: 3, plus: 4 }.get(grade);
    for _ in 0..hits {
        let mut living: Vec<usize> = (0..state.enemies.len())
            .filter(|&i| state.enemies[i].hp.0 > 0)
            .collect();
        if living.is_empty() { break; }
        rng.shuffle(&mut living);
        let target = living[0];
        let raw = resolve_damage(3, &state.player.statuses, &state.enemies[target].statuses);
        let enemy = &mut state.enemies[target];
        let damage = deal_damage(raw, &mut enemy.hp, &mut enemy.block);
        events.push(Event::PlayerAttacked { raw, damage });
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Sword Boomerang",  CardDescription::Static("Deal 3 damage to a random enemy 3 times.")),
        Grade::Plus => ("Sword Boomerang+", CardDescription::Static("Deal 3 damage to a random enemy 4 times.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(1), card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "sword-boomerang", Grade::Plus => "sword-boomerang-plus" }
}

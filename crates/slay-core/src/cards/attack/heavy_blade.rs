use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, deal_damage};
use crate::status::resolve_damage_with_strength_multiplier;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, target: usize) {
    let (base, str_mult) = match grade { Grade::Base => (14, 3), Grade::Plus => (21, 5) };
    let raw = resolve_damage_with_strength_multiplier(base, str_mult, &state.player.statuses, &state.enemies[target].statuses);
    let enemy = &mut state.enemies[target];
    let damage = deal_damage(raw, &mut enemy.hp, &mut enemy.block);
    events.push(Event::PlayerAttacked { raw, damage });
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Heavy Blade",  CardDescription::Static("Deal 14 damage. Strength affects this card 3 times.")),
        Grade::Plus => ("Heavy Blade+", CardDescription::Static("Deal 21 damage. Strength affects this card 5 times.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(2), card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "heavy-blade", Grade::Plus => "heavy-blade-plus" }
}

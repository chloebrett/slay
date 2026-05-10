use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event, damage_all_enemies};
use crate::rng::Rng;
use crate::status::{resolve_damage, StatusMap};
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, rng: &mut impl Rng) {
    let base_damage = GradeValues { base: 10, plus: 14 }.get(grade);
    let damage = resolve_damage(base_damage, &state.player.statuses, &StatusMap::new());
    damage_all_enemies(&mut state.enemies, events, damage);
    if !state.player.hand.is_empty() {
        let mut indices: Vec<usize> = (0..state.player.hand.len()).collect();
        rng.shuffle(&mut indices);
        let discarded = state.player.hand.remove(indices[0]);
        state.player.discard_pile.push(discarded);
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, description) = match grade {
        Grade::Base => ("All-Out Attack",  CardDescription::Static("Deal 10 damage to ALL enemies. Discard 1 card at random.")),
        Grade::Plus => ("All-Out Attack+", CardDescription::Static("Deal 14 damage to ALL enemies. Discard 1 card at random.")),
    };
    CardDef { name, description, energy_cost: Energy(1), card_type: CardType::Attack }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "all-out-attack", Grade::Plus => "all-out-attack-plus" }
}

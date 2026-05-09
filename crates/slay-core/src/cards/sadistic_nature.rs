use super::{CardDef, CardDescription, CardType, Grade, GradeValues};
use crate::combat::{CombatState, Event};
use crate::status::StatusEffect;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, _events: &mut Vec<Event>, grade: Grade) {
    let damage = GradeValues { base: 5, plus: 7 }.get(grade);
    *state.player.statuses.entry(StatusEffect::SadisticNature).or_insert(0) = damage;
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Sadistic Nature",  CardDescription::Static("Whenever an enemy receives a debuff, deal 5 damage to them.")),
        Grade::Plus => ("Sadistic Nature+", CardDescription::Static("Whenever an enemy receives a debuff, deal 7 damage to them.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(0), card_type: CardType::Power }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "sadistic-nature", Grade::Plus => "sadistic-nature-plus" }
}

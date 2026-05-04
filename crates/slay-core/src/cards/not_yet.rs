use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event};
use crate::types::{Energy, Hp};

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, amount: i32, _target: usize) {
    let new_hp = (state.player.hp.0 + amount).min(state.player.max_hp.0);
    state.player.hp = Hp(new_hp);
    events.push(Event::Healed { amount });
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Not Yet",  CardDescription::Static("Heal 10 HP.")),
        Grade::Plus => ("Not Yet+", CardDescription::Static("Heal 13 HP.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(2), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "not-yet", Grade::Plus => "not-yet-plus" }
}

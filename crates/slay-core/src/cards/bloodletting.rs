use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{CombatState, Event, damage_player_from_card};
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade) {
    let energy_gain = match grade { Grade::Base => 2, Grade::Plus => 3 };
    damage_player_from_card(state, events, 3);
    state.player.energy = Energy(state.player.energy.0 + energy_gain);
    events.push(Event::EnergyGained { amount: energy_gain });
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, desc) = match grade {
        Grade::Base => ("Bloodletting",  CardDescription::Static("Lose 3 HP. Gain 2 Energy.")),
        Grade::Plus => ("Bloodletting+", CardDescription::Static("Lose 3 HP. Gain 3 Energy.")),
    };
    CardDef { name, description: desc, energy_cost: Energy(0), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "bloodletting", Grade::Plus => "bloodletting-plus" }
}

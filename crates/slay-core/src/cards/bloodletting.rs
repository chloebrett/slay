use crate::combat::{CombatState, Event, damage_player};
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, hp_loss: i32, energy_gain: i32) {
    damage_player(state, events, hp_loss);
    state.player.energy = Energy(state.player.energy.0 + energy_gain);
    events.push(Event::EnergyGained { amount: energy_gain });
}

use crate::status::{StatusEffect, StatusMap};
use crate::types::Hp;

use super::{EnemyDamageReaction, EnemyDef, Move};

pub const DEF: EnemyDef = EnemyDef { name: "The Guardian", max_hp: Hp(240) };

pub fn on_player_attack_damage(statuses: &StatusMap, hp_lost: i32) -> Option<EnemyDamageReaction> {
    let mode = statuses.get(&StatusEffect::GuardianMode).copied().unwrap_or(0);
    if mode != 0 || hp_lost == 0 {
        return None;
    }
    let old_progress = statuses.get(&StatusEffect::ModeShiftProgress).copied().unwrap_or(0);
    let new_progress = old_progress + hp_lost;
    let count = statuses.get(&StatusEffect::ModeShiftCount).copied().unwrap_or(0);
    if new_progress < 30 + count * 10 {
        return Some(EnemyDamageReaction {
            block_gain: 0,
            status_events: vec![],
            silent_adds: vec![],
            silent_sets: vec![(StatusEffect::ModeShiftProgress, new_progress)],
            force_move: None,
        });
    }
    Some(EnemyDamageReaction {
        block_gain: 20,
        status_events: vec![(StatusEffect::SharpHide, 3)],
        silent_adds: vec![(StatusEffect::GuardianMode, 1), (StatusEffect::ModeShiftCount, 1)],
        silent_sets: vec![(StatusEffect::ModeShiftProgress, 0)],
        force_move: Some(Move::GuardianRollAttack),
    })
}

pub fn next_move(last: Option<Move>) -> Move {
    match last {
        None                           => Move::GuardianChargingUp,
        Some(Move::GuardianChargingUp) => Move::GuardianFierceBash,
        Some(Move::GuardianFierceBash) => Move::GuardianVentSteam,
        Some(Move::GuardianVentSteam)  => Move::GuardianWhirlwind,
        Some(Move::GuardianWhirlwind)  => Move::GuardianChargingUp,
        Some(Move::GuardianRollAttack) => Move::GuardianTwinSlam,
        Some(Move::GuardianTwinSlam)   => Move::GuardianWhirlwind,
        Some(_)                        => Move::GuardianChargingUp,
    }
}

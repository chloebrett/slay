use crate::types::Hp;

use super::{EnemyDef, Move};

pub const DEF: EnemyDef = EnemyDef { name: "The Guardian", max_hp: Hp(240) };

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

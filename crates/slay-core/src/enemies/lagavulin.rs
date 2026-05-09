use super::{EnemyDef, Move};
use crate::status::{StatusEffect, StatusMap, get_stacks};
use crate::types::Hp;

pub const DEF: EnemyDef = EnemyDef { name: "Lagavulin", max_hp: Hp(109) };

pub fn next_move(last: Option<Move>, statuses: &StatusMap) -> Move {
    match last {
        None => Move::LagavulinSleep,
        Some(Move::LagavulinSleep) => {
            if get_stacks(statuses, StatusEffect::Sleep) > 0 {
                Move::LagavulinSleep
            } else {
                Move::LagavulinAttackA
            }
        }
        Some(Move::LagavulinStunned) => Move::LagavulinAttackA,
        Some(Move::LagavulinAttackA) => Move::LagavulinAttackB,
        Some(Move::LagavulinAttackB) => Move::LagavulinSiphonSoul,
        Some(Move::LagavulinSiphonSoul) => Move::LagavulinAttackA,
        _ => unreachable!("unexpected last move for Lagavulin"),
    }
}

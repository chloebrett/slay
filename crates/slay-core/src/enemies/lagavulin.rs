use super::{EnemyDef, Move};
use crate::status::{StatusEffect, StatusMap, get_stacks};
use crate::types::Hp;

pub const DEF: EnemyDef = EnemyDef { name: "Lagavulin", max_hp: Hp(109) };

pub fn next_move(history: &[Move], statuses: &StatusMap) -> Move {
    let last = history.last().copied();
    let prev = history.len().checked_sub(2).and_then(|i| history.get(i)).copied();
    match last {
        None => Move::LagavulinSleep,
        Some(Move::LagavulinSleep) => {
            if get_stacks(statuses, StatusEffect::Sleep) > 0 {
                Move::LagavulinSleep
            } else {
                Move::LagavulinAttack
            }
        }
        Some(Move::LagavulinStunned) => Move::LagavulinAttack,
        Some(Move::LagavulinAttack) => {
            if prev == Some(Move::LagavulinAttack) {
                Move::LagavulinSiphonSoul
            } else {
                Move::LagavulinAttack
            }
        }
        Some(Move::LagavulinSiphonSoul) => Move::LagavulinAttack,
        _ => unreachable!("unexpected move in Lagavulin history"),
    }
}

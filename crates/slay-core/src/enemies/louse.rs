use crate::types::Hp;

use super::{EnemyDef, Move};

pub const DEF: EnemyDef = EnemyDef { name: "Louse", max_hp: Hp(20) };

pub fn next_move(last: Option<Move>) -> Move {
    match last {
        None | Some(Move::LouseBlock) => Move::LouseBite,
        _ => Move::LouseBlock,
    }
}

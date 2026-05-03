use crate::types::Hp;

use super::{EnemyDef, Move};

pub const DEF: EnemyDef = EnemyDef { name: "Fungibeast", max_hp: Hp(22) };

pub fn next_move(last: Option<Move>) -> Move {
    match last {
        Some(Move::FungiLight) => Move::FungiHeavy,
        _ => Move::FungiLight,
    }
}

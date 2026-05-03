use crate::types::Hp;

use super::{EnemyDef, Move};

pub const DEF: EnemyDef = EnemyDef { name: "Cultist", max_hp: Hp(50) };

pub fn next_move(last: Option<Move>) -> Move {
    match last {
        None => Move::Incantation,
        _ => Move::DarkStrike,
    }
}

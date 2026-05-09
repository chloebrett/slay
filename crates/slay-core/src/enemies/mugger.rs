use crate::types::Hp;

use super::{EnemyDef, Move};

pub const DEF: EnemyDef = EnemyDef { name: "Mugger", max_hp: Hp(48) };

pub fn next_move(history: &[Move]) -> Move {
    match history.len() {
        0 => Move::MuggerMug,
        1 => Move::MuggerLunge,
        2 => Move::MuggerMug,
        3 => Move::MuggerSmokeBomb,
        _ => Move::MuggerFlee,
    }
}

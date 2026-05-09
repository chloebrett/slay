use crate::types::Hp;

use super::{EnemyDef, Move};

pub const DEF: EnemyDef = EnemyDef { name: "Looter", max_hp: Hp(44) };

pub fn next_move(history: &[Move]) -> Move {
    match history.len() {
        0 => Move::LooterMug,
        1 => Move::LooterLunge,
        2 => Move::LooterMug,
        3 => Move::LooterSmokeBomb,
        _ => Move::LooterFlee,
    }
}

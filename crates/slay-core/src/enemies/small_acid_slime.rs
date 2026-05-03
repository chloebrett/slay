use crate::types::Hp;

use super::{EnemyDef, Move};

pub const DEF: EnemyDef = EnemyDef { name: "Acid Slime (S)", max_hp: Hp(10) };

pub fn next_move(last: Option<Move>) -> Move {
    match last {
        None | Some(Move::Lick) => Move::AcidTackle,
        _ => Move::Lick,
    }
}

use crate::types::Hp;

use super::{EnemyDef, Move};

pub const DEF: EnemyDef = EnemyDef { name: "Spike Slime (S)", max_hp: Hp(10) };

pub fn next_move(_last: Option<Move>) -> Move {
    Move::FlameTackle
}

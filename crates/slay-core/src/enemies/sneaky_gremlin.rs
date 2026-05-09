use crate::types::Hp;

use super::{EnemyDef, Move};

pub const DEF: EnemyDef = EnemyDef { name: "Sneaky Gremlin", max_hp: Hp(10) };

pub fn next_move() -> Move {
    Move::GremlinPuncture
}

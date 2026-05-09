use crate::types::Hp;

use super::{EnemyDef, Move};

pub const DEF: EnemyDef = EnemyDef { name: "Fat Gremlin", max_hp: Hp(13) };

pub fn next_move() -> Move {
    Move::GremlinSmash
}

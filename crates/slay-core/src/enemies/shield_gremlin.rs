use crate::types::Hp;

use super::{EnemyDef, Move};

pub const DEF: EnemyDef = EnemyDef { name: "Shield Gremlin", max_hp: Hp(12) };

pub fn next_move(allies_alive: usize) -> Move {
    if allies_alive > 0 { Move::ShieldProtect } else { Move::ShieldBash }
}

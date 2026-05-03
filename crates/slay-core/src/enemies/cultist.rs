use crate::types::Hp;

use super::{EnemyDef, Intent};

pub const DEF: EnemyDef = EnemyDef {
    name: "Cultist",
    max_hp: Hp(50),
};

pub fn next_intent(turn: u32) -> Intent {
    if turn == 1 {
        Intent::Defend(0)
    } else {
        Intent::Attack(6)
    }
}

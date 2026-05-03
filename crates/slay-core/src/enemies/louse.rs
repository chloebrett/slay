use crate::types::Hp;

use super::{EnemyDef, Intent};

pub const DEF: EnemyDef = EnemyDef {
    name: "Louse",
    max_hp: Hp(20),
};

pub fn next_intent(turn: u32) -> Intent {
    if turn % 2 == 1 {
        Intent::Attack(8)
    } else {
        Intent::Defend(5)
    }
}

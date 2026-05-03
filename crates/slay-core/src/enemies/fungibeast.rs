use crate::types::Hp;

use super::{EnemyDef, Intent};

pub const DEF: EnemyDef = EnemyDef {
    name: "Fungibeast",
    max_hp: Hp(22),
};

pub fn next_intent(turn: u32) -> Intent {
    if turn % 2 == 1 {
        Intent::Attack(6)
    } else {
        Intent::Attack(10)
    }
}

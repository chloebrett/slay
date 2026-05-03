use crate::rng::Rng;
use crate::types::Hp;

use super::{EnemyDef, Move};

pub const DEF: EnemyDef = EnemyDef { name: "Red Louse", max_hp: Hp(12) };

pub fn next_move(last: Option<Move>, rng: &mut impl Rng) -> Move {
    match last {
        None | Some(Move::Grow) => Move::RedLouseBite,
        _ => {
            // 75% Bite, 25% Grow — weighted candidates, no repeat of Grow
            let mut candidates = [
                Move::RedLouseBite, Move::RedLouseBite, Move::RedLouseBite,
                Move::Grow,
            ];
            rng.shuffle(&mut candidates);
            candidates[0]
        }
    }
}

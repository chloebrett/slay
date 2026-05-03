use crate::rng::Rng;
use crate::types::Hp;

use super::{EnemyDef, Move};

pub const DEF: EnemyDef = EnemyDef { name: "Green Louse", max_hp: Hp(12) };

pub fn next_move(last: Option<Move>, rng: &mut impl Rng) -> Move {
    match last {
        None | Some(Move::SpitWeb) => Move::GreenBite,
        _ => {
            // 75% Bite, 25% Spit Web — no repeat of Spit Web
            let mut candidates = [
                Move::GreenBite, Move::GreenBite, Move::GreenBite,
                Move::SpitWeb,
            ];
            rng.shuffle(&mut candidates);
            candidates[0]
        }
    }
}

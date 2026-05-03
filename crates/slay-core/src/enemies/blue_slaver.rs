use crate::rng::Rng;
use crate::types::Hp;

use super::{EnemyDef, Move};

pub const DEF: EnemyDef = EnemyDef { name: "Blue Slaver", max_hp: Hp(48) };

pub fn next_move(last: Option<Move>, rng: &mut impl Rng) -> Move {
    let Some(last) = last else { return Move::BlueStab };

    // 60% Stab, 40% Rake — no repeat of last move (6:4 ratio → 3:2)
    let mut candidates: Vec<Move> = [
        Move::BlueStab, Move::BlueStab, Move::BlueStab,
        Move::Rake, Move::Rake,
    ]
    .into_iter()
    .filter(|&m| m != last)
    .collect();

    rng.shuffle(&mut candidates);
    candidates[0]
}

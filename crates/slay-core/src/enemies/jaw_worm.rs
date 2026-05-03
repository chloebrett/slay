use crate::rng::Rng;
use crate::types::Hp;

use super::{EnemyDef, Move};

pub const DEF: EnemyDef = EnemyDef { name: "Jaw Worm", max_hp: Hp(40) };

pub fn next_move(last: Option<Move>, rng: &mut impl Rng) -> Move {
    let Some(last) = last else { return Move::Chomp };

    // Weighted candidates: Bellow 45%, Thrash 30%, Chomp 25% (GCD = 5 → 9:6:5)
    let mut candidates: Vec<Move> = [
        Move::Bellow, Move::Bellow, Move::Bellow, Move::Bellow, Move::Bellow,
        Move::Bellow, Move::Bellow, Move::Bellow, Move::Bellow,
        Move::Thrash, Move::Thrash, Move::Thrash, Move::Thrash, Move::Thrash, Move::Thrash,
        Move::Chomp,  Move::Chomp,  Move::Chomp,  Move::Chomp,  Move::Chomp,
    ]
    .into_iter()
    .filter(|&m| m != last)
    .collect();

    rng.shuffle(&mut candidates);
    candidates[0]
}

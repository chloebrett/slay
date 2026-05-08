use crate::rng::Rng;
use crate::types::Hp;

use super::{EnemyDef, Move};

pub const DEF: EnemyDef = EnemyDef { name: "Gremlin Nob", max_hp: Hp(84) };

pub fn next_move(last: Option<Move>, rng: &mut impl Rng) -> Move {
    let Some(last) = last else { return Move::NobBellow };

    // 33% SkullBash, 67% BullRush; no repeats
    let mut candidates: Vec<Move> = [
        Move::SkullBash,
        Move::BullRush, Move::BullRush,
    ]
    .into_iter()
    .filter(|&m| m != last)
    .collect();

    rng.shuffle(&mut candidates);
    candidates[0]
}

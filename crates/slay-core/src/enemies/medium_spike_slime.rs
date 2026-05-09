use crate::rng::Rng;
use crate::types::Hp;

use super::{EnemyDef, Move};

pub const DEF: EnemyDef = EnemyDef { name: "Spike Slime (M)", max_hp: Hp(30) };

pub fn next_move(history: &[Move], rng: &mut impl Rng) -> Move {
    let last = history.last().copied();
    let second_last = history.len().checked_sub(2).and_then(|i| history.get(i)).copied();
    let repeated_twice = last == second_last && last.is_some();

    let mut candidates: Vec<Move> = [
        Move::MediumSpikeFlameTackle,
        Move::MediumSpikeLick, Move::MediumSpikeLick,
    ]
    .into_iter()
    .filter(|&m| !(repeated_twice && Some(m) == last))
    .collect();

    rng.shuffle(&mut candidates);
    candidates[0]
}

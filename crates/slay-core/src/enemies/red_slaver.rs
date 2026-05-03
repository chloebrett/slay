use crate::rng::Rng;
use crate::types::Hp;

use super::{EnemyDef, Move};

pub const DEF: EnemyDef = EnemyDef { name: "Red Slaver", max_hp: Hp(48) };

pub fn next_move(last: Option<Move>, rng: &mut impl Rng) -> Move {
    let Some(last) = last else { return Move::RedStab };

    // 55% Stab, 45% Scrape, plus Entangle once (filters itself after use via no-repeat rule)
    // Weights: 11 RedStab, 9 Scrape, 1 SlaveEntangle (21 total)
    let mut candidates: Vec<Move> = [
        Move::RedStab,       Move::RedStab,       Move::RedStab,
        Move::RedStab,       Move::RedStab,       Move::RedStab,
        Move::RedStab,       Move::RedStab,       Move::RedStab,
        Move::RedStab,       Move::RedStab,
        Move::Scrape,        Move::Scrape,         Move::Scrape,
        Move::Scrape,        Move::Scrape,         Move::Scrape,
        Move::Scrape,        Move::Scrape,         Move::Scrape,
        Move::SlaveEntangle,
    ]
    .into_iter()
    .filter(|&m| m != last)
    .collect();

    rng.shuffle(&mut candidates);
    candidates[0]
}

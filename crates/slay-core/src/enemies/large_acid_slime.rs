use crate::rng::Rng;
use crate::types::Hp;

use super::{EnemyDamageReaction, EnemyDef, Move};

pub const DEF: EnemyDef = EnemyDef { name: "Acid Slime (L)", max_hp: Hp(67) };

pub fn next_move(history: &[Move], rng: &mut impl Rng) -> Move {
    let last = history.last().copied();
    let second_last = history.len().checked_sub(2).and_then(|i| history.get(i)).copied();
    let repeated_twice = last == second_last && last.is_some();

    let mut candidates: Vec<Move> = [
        Move::LargeAcidCorrosiveSpit,
        Move::LargeAcidLick,
        Move::LargeAcidTackle,
    ]
    .into_iter()
    .filter(|&m| !(repeated_twice && Some(m) == last))
    .collect();

    rng.shuffle(&mut candidates);
    candidates[0]
}

pub fn on_player_attack_damage(current_hp: Hp, max_hp: Hp) -> Option<EnemyDamageReaction> {
    if current_hp.0 <= max_hp.0 / 2 {
        Some(EnemyDamageReaction {
            block_gain: 0,
            status_events: vec![],
            silent_adds: vec![],
            silent_sets: vec![],
            force_move: Some(Move::LargeAcidSplit),
        })
    } else {
        None
    }
}

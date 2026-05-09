use crate::types::Hp;

use super::{EnemyDamageReaction, EnemyDef, Move};

pub const DEF: EnemyDef = EnemyDef { name: "Slime Boss", max_hp: Hp(140) };

pub fn next_move(history: &[Move]) -> Move {
    match history.len() % 3 {
        0 => Move::SlimeBossGoopSpray,
        1 => Move::SlimeBossPreparing,
        _ => Move::SlimeBossSlam,
    }
}

pub fn on_player_attack_damage(current_hp: Hp, max_hp: Hp) -> Option<EnemyDamageReaction> {
    if current_hp.0 * 2 <= max_hp.0 {
        Some(EnemyDamageReaction {
            block_gain: 0,
            status_events: vec![],
            silent_adds: vec![],
            silent_sets: vec![],
            force_move: Some(Move::SlimeBossSplit),
        })
    } else {
        None
    }
}

use crate::status::{StatusEffect, StatusMap};
use crate::types::Hp;

use super::{EnemyDamageReaction, EnemyDef, Move};

pub const DEF: EnemyDef = EnemyDef { name: "Mad Gremlin", max_hp: Hp(20) };

pub fn next_move() -> Move {
    Move::GremlinScratch
}

pub fn on_player_attack_damage(_statuses: &StatusMap, _hp_lost: i32, _current_hp: Hp, _max_hp: Hp) -> Option<EnemyDamageReaction> {
    Some(EnemyDamageReaction {
        block_gain: 0,
        status_events: vec![(StatusEffect::Strength, 1)],
        silent_adds: vec![],
        silent_sets: vec![],
        force_move: None,
    })
}

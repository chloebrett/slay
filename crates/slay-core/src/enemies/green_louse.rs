use crate::rng::Rng;
use crate::status::{StatusEffect, StatusMap, get_stacks};
use crate::types::Hp;

use super::{EnemyDamageReaction, EnemyDef, Move};

pub const DEF: EnemyDef = EnemyDef { name: "Green Louse", max_hp: Hp(12) };

pub fn on_player_attack_damage(statuses: &StatusMap) -> Option<EnemyDamageReaction> {
    let curl_up = get_stacks(statuses, StatusEffect::CurlUp);
    if curl_up <= 0 {
        return None;
    }
    Some(EnemyDamageReaction {
        block_gain: curl_up,
        status_events: vec![],
        silent_adds: vec![],
        silent_sets: vec![(StatusEffect::CurlUp, 0)],
        force_move: None,
    })
}

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

use crate::types::Hp;

use super::{EnemyDef, Move};

pub const DEF: EnemyDef = EnemyDef { name: "Sentry", max_hp: Hp(38) };

pub fn next_move(last: Option<Move>) -> Move {
    match last {
        Some(Move::SentryBeam) => Move::SentryBolt,
        _ => Move::SentryBeam,
    }
}

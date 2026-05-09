use super::{EnemyDef, Move};
use crate::types::Hp;

pub const DEF: EnemyDef = EnemyDef { name: "Hexaghost", max_hp: Hp(250) };

pub fn next_move(history: &[Move]) -> Move {
    let already_infernoed = history.contains(&Move::HexaghostInferno);
    let sear = if already_infernoed { Move::HexaghostSearUpgraded } else { Move::HexaghostSear };
    match history.len() {
        0 => Move::HexaghostActivate,
        1 => Move::HexaghostDivider,
        n => match (n - 2) % 7 {
            0 => sear,
            1 => Move::HexaghostTackle,
            2 => sear,
            3 => Move::HexaghostInflame,
            4 => Move::HexaghostTackle,
            5 => sear,
            _ => Move::HexaghostInferno,
        },
    }
}

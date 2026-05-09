use crate::types::Hp;

use super::{EnemyDef, Move};

pub const DEF: EnemyDef = EnemyDef { name: "Gremlin Wizard", max_hp: Hp(21) };

pub fn next_move(history: &[Move]) -> Move {
    let consecutive_chargings = history.iter().rev()
        .take_while(|&&m| m == Move::WizardCharging)
        .count();
    let has_blasted = history.contains(&Move::WizardUltimateBlast);

    let threshold = if has_blasted { 3 } else { 2 };
    if consecutive_chargings >= threshold {
        Move::WizardUltimateBlast
    } else {
        Move::WizardCharging
    }
}

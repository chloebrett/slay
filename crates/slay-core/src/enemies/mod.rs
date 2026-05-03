mod cultist;
mod fungibeast;
mod louse;

use crate::types::Hp;

#[derive(Debug, Clone, PartialEq)]
pub enum EnemyKind {
    Louse,
    Fungibeast,
    Cultist,
}

#[derive(Debug, Clone, Copy)]
pub struct EnemyDef {
    pub name: &'static str,
    pub max_hp: Hp,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Intent {
    Attack(i32),
    Defend(i32),
}

impl EnemyKind {
    pub fn def(&self) -> EnemyDef {
        match self {
            EnemyKind::Louse => louse::DEF,
            EnemyKind::Fungibeast => fungibeast::DEF,
            EnemyKind::Cultist => cultist::DEF,
        }
    }

    pub fn name(&self) -> &'static str { self.def().name }
    pub fn max_hp(&self) -> Hp { self.def().max_hp }
}

pub fn next_intent(kind: &EnemyKind, turn: u32) -> Intent {
    match kind {
        EnemyKind::Louse => louse::next_intent(turn),
        EnemyKind::Fungibeast => fungibeast::next_intent(turn),
        EnemyKind::Cultist => cultist::next_intent(turn),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Hp;

    #[test]
    fn louse_has_20_hp() {
        assert_eq!(EnemyKind::Louse.max_hp(), Hp(20));
    }

    #[test]
    fn louse_attacks_8_on_odd_turns() {
        assert_eq!(next_intent(&EnemyKind::Louse, 1), Intent::Attack(8));
        assert_eq!(next_intent(&EnemyKind::Louse, 3), Intent::Attack(8));
    }

    #[test]
    fn louse_defends_5_on_even_turns() {
        assert_eq!(next_intent(&EnemyKind::Louse, 2), Intent::Defend(5));
        assert_eq!(next_intent(&EnemyKind::Louse, 4), Intent::Defend(5));
    }

    #[test]
    fn fungibeast_has_22_hp() {
        assert_eq!(EnemyKind::Fungibeast.max_hp(), Hp(22));
    }

    #[test]
    fn cultist_has_50_hp() {
        assert_eq!(EnemyKind::Cultist.max_hp(), Hp(50));
    }

    #[test]
    fn cultist_incantation_on_turn_1() {
        assert_eq!(next_intent(&EnemyKind::Cultist, 1), Intent::Defend(0));
    }

    #[test]
    fn cultist_dark_strike_on_turn_2() {
        assert_eq!(next_intent(&EnemyKind::Cultist, 2), Intent::Attack(6));
    }

    #[test]
    fn cultist_dark_strike_on_all_subsequent_turns() {
        assert_eq!(next_intent(&EnemyKind::Cultist, 3), Intent::Attack(6));
        assert_eq!(next_intent(&EnemyKind::Cultist, 10), Intent::Attack(6));
    }

    #[test]
    fn fungibeast_attacks_6_on_odd_turns() {
        assert_eq!(next_intent(&EnemyKind::Fungibeast, 1), Intent::Attack(6));
        assert_eq!(next_intent(&EnemyKind::Fungibeast, 3), Intent::Attack(6));
    }

    #[test]
    fn fungibeast_attacks_10_on_even_turns() {
        assert_eq!(next_intent(&EnemyKind::Fungibeast, 2), Intent::Attack(10));
        assert_eq!(next_intent(&EnemyKind::Fungibeast, 4), Intent::Attack(10));
    }
}

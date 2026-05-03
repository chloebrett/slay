mod louse;

use crate::types::Hp;

#[derive(Debug, Clone, PartialEq)]
pub enum EnemyKind {
    Louse,
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
        }
    }

    pub fn name(&self) -> &'static str { self.def().name }
    pub fn max_hp(&self) -> Hp { self.def().max_hp }
}

pub fn next_intent(kind: &EnemyKind, turn: u32) -> Intent {
    match kind {
        EnemyKind::Louse => louse::next_intent(turn),
    }
}

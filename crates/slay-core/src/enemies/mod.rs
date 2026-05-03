mod cultist;
mod fungibeast;
mod louse;

use crate::rng::Rng;
use crate::status::StatusEffect;
use crate::types::Hp;

#[derive(Debug, Clone, PartialEq)]
pub enum EnemyKind {
    Louse,
    Fungibeast,
    Cultist,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    // Louse
    LouseBite,
    LouseBlock,
    // Fungibeast
    FungiLight,
    FungiHeavy,
    // Cultist
    Incantation,
    DarkStrike,
}

#[derive(Debug, Clone, Copy)]
pub enum Effect {
    DealDamage(i32),
    GainBlock(i32),
    GainStatus(StatusEffect, i32),  // applies to self
    ApplyStatus(StatusEffect, i32), // applies to player
}

pub struct MoveDef {
    pub name: &'static str,
    pub effects: &'static [Effect],
}

impl Move {
    pub fn def(self) -> MoveDef {
        match self {
            Move::LouseBite    => MoveDef { name: "Bite",        effects: &[Effect::DealDamage(8)] },
            Move::LouseBlock   => MoveDef { name: "Block",       effects: &[Effect::GainBlock(5)] },
            Move::FungiLight   => MoveDef { name: "Chomp",       effects: &[Effect::DealDamage(6)] },
            Move::FungiHeavy   => MoveDef { name: "Slam",        effects: &[Effect::DealDamage(10)] },
            Move::Incantation  => MoveDef { name: "Incantation", effects: &[Effect::GainStatus(StatusEffect::Ritual, 3)] },
            Move::DarkStrike   => MoveDef { name: "Dark Strike", effects: &[Effect::DealDamage(6)] },
        }
    }

    pub fn intent(self) -> Intent {
        let effects = self.def().effects;
        let damage: i32 = effects.iter().filter_map(|e| {
            if let Effect::DealDamage(n) = e { Some(*n) } else { None }
        }).sum();
        let block: i32 = effects.iter().filter_map(|e| {
            if let Effect::GainBlock(n) = e { Some(*n) } else { None }
        }).sum();
        match (damage, block) {
            (d, b) if d > 0 && b > 0 => Intent::AttackDefend(d, b),
            (d, _) if d > 0           => Intent::Attack(d),
            (_, b) if b > 0           => Intent::Defend(b),
            _                         => Intent::Buff,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Intent {
    Attack(i32),
    Defend(i32),
    AttackDefend(i32, i32),
    Buff,
}

pub struct EnemyDef {
    pub name: &'static str,
    pub max_hp: Hp,
}

impl EnemyKind {
    pub fn def(&self) -> EnemyDef {
        match self {
            EnemyKind::Louse     => louse::DEF,
            EnemyKind::Fungibeast => fungibeast::DEF,
            EnemyKind::Cultist   => cultist::DEF,
        }
    }

    pub fn name(&self) -> &'static str { self.def().name }
    pub fn max_hp(&self) -> Hp { self.def().max_hp }
}

pub fn next_move(kind: &EnemyKind, last: Option<Move>, rng: &mut impl Rng) -> Move {
    match kind {
        EnemyKind::Louse     => louse::next_move(last),
        EnemyKind::Fungibeast => fungibeast::next_move(last),
        EnemyKind::Cultist   => cultist::next_move(last),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::NoOpRng;

    fn rng() -> NoOpRng { NoOpRng }

    #[test]
    fn louse_has_20_hp() {
        assert_eq!(EnemyKind::Louse.max_hp(), Hp(20));
    }

    #[test]
    fn louse_bites_first_turn() {
        assert_eq!(next_move(&EnemyKind::Louse, None, &mut rng()), Move::LouseBite);
    }

    #[test]
    fn louse_blocks_after_biting() {
        assert_eq!(next_move(&EnemyKind::Louse, Some(Move::LouseBite), &mut rng()), Move::LouseBlock);
    }

    #[test]
    fn louse_bites_after_blocking() {
        assert_eq!(next_move(&EnemyKind::Louse, Some(Move::LouseBlock), &mut rng()), Move::LouseBite);
    }

    #[test]
    fn fungibeast_has_22_hp() {
        assert_eq!(EnemyKind::Fungibeast.max_hp(), Hp(22));
    }

    #[test]
    fn fungibeast_light_first_turn() {
        assert_eq!(next_move(&EnemyKind::Fungibeast, None, &mut rng()), Move::FungiLight);
    }

    #[test]
    fn fungibeast_heavy_after_light() {
        assert_eq!(next_move(&EnemyKind::Fungibeast, Some(Move::FungiLight), &mut rng()), Move::FungiHeavy);
    }

    #[test]
    fn fungibeast_light_after_heavy() {
        assert_eq!(next_move(&EnemyKind::Fungibeast, Some(Move::FungiHeavy), &mut rng()), Move::FungiLight);
    }

    #[test]
    fn cultist_has_50_hp() {
        assert_eq!(EnemyKind::Cultist.max_hp(), Hp(50));
    }

    #[test]
    fn cultist_incantation_on_first_turn() {
        assert_eq!(next_move(&EnemyKind::Cultist, None, &mut rng()), Move::Incantation);
    }

    #[test]
    fn cultist_dark_strike_after_incantation() {
        assert_eq!(next_move(&EnemyKind::Cultist, Some(Move::Incantation), &mut rng()), Move::DarkStrike);
    }

    #[test]
    fn cultist_dark_strike_repeats() {
        assert_eq!(next_move(&EnemyKind::Cultist, Some(Move::DarkStrike), &mut rng()), Move::DarkStrike);
    }

    #[test]
    fn incantation_intent_is_buff() {
        assert_eq!(Move::Incantation.intent(), Intent::Buff);
    }

    #[test]
    fn dark_strike_intent_is_attack_6() {
        assert_eq!(Move::DarkStrike.intent(), Intent::Attack(6));
    }

    #[test]
    fn louse_bite_intent_is_attack_8() {
        assert_eq!(Move::LouseBite.intent(), Intent::Attack(8));
    }

    #[test]
    fn louse_block_intent_is_defend_5() {
        assert_eq!(Move::LouseBlock.intent(), Intent::Defend(5));
    }
}

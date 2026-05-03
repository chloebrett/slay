mod cultist;
mod fungibeast;
mod jaw_worm;
mod louse;
mod red_louse;
mod small_spike_slime;

use crate::cards::Card;
use crate::rng::Rng;
use crate::status::StatusEffect;
use crate::types::Hp;

#[derive(Debug, Clone, PartialEq)]
pub enum EnemyKind {
    Louse,
    Fungibeast,
    Cultist,
    JawWorm,
    SmallSpikeSlime,
    RedLouse,
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
    // Jaw Worm
    Chomp,
    Thrash,
    Bellow,
    // Small Spike Slime
    FlameTackle,
    // Red Louse
    RedLouseBite,
    Grow,
}

#[derive(Debug, Clone)]
pub enum Effect {
    DealDamage(i32),
    GainBlock(i32),
    GainStatus(StatusEffect, i32),  // applies to self
    ApplyStatus(StatusEffect, i32), // applies to player
    AddToDiscard(Card),             // adds a card to the player's discard pile
}

pub struct MoveDef {
    pub name: &'static str,
    pub effects: Vec<Effect>,
}

impl Move {
    pub fn def(self) -> MoveDef {
        match self {
            Move::LouseBite    => MoveDef { name: "Bite",        effects: vec![Effect::DealDamage(8)] },
            Move::LouseBlock   => MoveDef { name: "Block",       effects: vec![Effect::GainBlock(5)] },
            Move::FungiLight   => MoveDef { name: "Chomp",       effects: vec![Effect::DealDamage(6)] },
            Move::FungiHeavy   => MoveDef { name: "Slam",        effects: vec![Effect::DealDamage(10)] },
            Move::Incantation  => MoveDef { name: "Incantation", effects: vec![Effect::GainStatus(StatusEffect::Ritual, 3)] },
            Move::DarkStrike   => MoveDef { name: "Dark Strike", effects: vec![Effect::DealDamage(6)] },
            Move::Chomp  => MoveDef { name: "Chomp",  effects: vec![Effect::DealDamage(11)] },
            Move::Thrash => MoveDef { name: "Thrash", effects: vec![Effect::DealDamage(7), Effect::GainBlock(5)] },
            Move::Bellow => MoveDef { name: "Bellow", effects: vec![Effect::GainStatus(StatusEffect::Strength, 3), Effect::GainBlock(6)] },
            Move::FlameTackle      => MoveDef { name: "Flame Tackle",  effects: vec![Effect::DealDamage(5), Effect::AddToDiscard(Card::Dazed)] },
            Move::RedLouseBite     => MoveDef { name: "Bite",          effects: vec![Effect::DealDamage(6)] },
            Move::Grow             => MoveDef { name: "Grow",          effects: vec![Effect::GainStatus(StatusEffect::Strength, 3)] },
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
        let buffs_self = effects.iter().any(|e| matches!(e, Effect::GainStatus(_, _)));
        match (damage, block, buffs_self) {
            (d, b, _)    if d > 0 && b > 0 => Intent::AttackDefend(d, b),
            (d, _, _)    if d > 0           => Intent::Attack(d),
            (_, b, false) if b > 0          => Intent::Defend(b),
            _                               => Intent::Buff,
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
            EnemyKind::Louse      => louse::DEF,
            EnemyKind::Fungibeast => fungibeast::DEF,
            EnemyKind::Cultist    => cultist::DEF,
            EnemyKind::JawWorm        => jaw_worm::DEF,
            EnemyKind::SmallSpikeSlime => small_spike_slime::DEF,
            EnemyKind::RedLouse        => red_louse::DEF,
        }
    }

    pub fn name(&self) -> &'static str { self.def().name }
    pub fn max_hp(&self) -> Hp { self.def().max_hp }
}

pub fn next_move(kind: &EnemyKind, last: Option<Move>, rng: &mut impl Rng) -> Move {
    match kind {
        EnemyKind::Louse      => louse::next_move(last),
        EnemyKind::Fungibeast => fungibeast::next_move(last),
        EnemyKind::Cultist    => cultist::next_move(last),
        EnemyKind::JawWorm         => jaw_worm::next_move(last, rng),
        EnemyKind::SmallSpikeSlime => small_spike_slime::next_move(last),
        EnemyKind::RedLouse        => red_louse::next_move(last, rng),
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
    fn jaw_worm_has_40_hp() {
        assert_eq!(EnemyKind::JawWorm.max_hp(), Hp(40));
    }

    #[test]
    fn jaw_worm_chomps_on_first_turn() {
        assert_eq!(next_move(&EnemyKind::JawWorm, None, &mut rng()), Move::Chomp);
    }

    #[test]
    fn jaw_worm_never_repeats_last_move() {
        for last in [Move::Chomp, Move::Thrash, Move::Bellow] {
            let next = next_move(&EnemyKind::JawWorm, Some(last), &mut rng());
            assert_ne!(next, last, "repeated {last:?}");
        }
    }

    #[test]
    fn chomp_is_attack_11() {
        assert_eq!(Move::Chomp.intent(), Intent::Attack(11));
    }

    #[test]
    fn thrash_is_attack_7_defend_5() {
        assert_eq!(Move::Thrash.intent(), Intent::AttackDefend(7, 5));
    }

    #[test]
    fn bellow_is_buff() {
        assert_eq!(Move::Bellow.intent(), Intent::Buff);
    }

    #[test]
    fn small_spike_slime_has_10_hp() {
        assert_eq!(EnemyKind::SmallSpikeSlime.max_hp(), Hp(10));
    }

    #[test]
    fn small_spike_slime_always_flame_tackles() {
        assert_eq!(next_move(&EnemyKind::SmallSpikeSlime, None, &mut rng()), Move::FlameTackle);
        assert_eq!(next_move(&EnemyKind::SmallSpikeSlime, Some(Move::FlameTackle), &mut rng()), Move::FlameTackle);
    }

    #[test]
    fn flame_tackle_intent_is_attack_5() {
        assert_eq!(Move::FlameTackle.intent(), Intent::Attack(5));
    }

    #[test]
    fn red_louse_has_12_hp() {
        assert_eq!(EnemyKind::RedLouse.max_hp(), Hp(12));
    }

    #[test]
    fn red_louse_bites_on_first_turn() {
        assert_eq!(next_move(&EnemyKind::RedLouse, None, &mut rng()), Move::RedLouseBite);
    }

    #[test]
    fn red_louse_bites_after_grow() {
        assert_eq!(next_move(&EnemyKind::RedLouse, Some(Move::Grow), &mut rng()), Move::RedLouseBite);
    }

    #[test]
    fn red_louse_never_repeats_grow() {
        let next = next_move(&EnemyKind::RedLouse, Some(Move::Grow), &mut rng());
        assert_ne!(next, Move::Grow);
    }

    #[test]
    fn grow_intent_is_buff() {
        assert_eq!(Move::Grow.intent(), Intent::Buff);
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

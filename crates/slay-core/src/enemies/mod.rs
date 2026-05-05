mod blue_slaver;
mod cultist;
mod fungibeast;
mod green_louse;
mod jaw_worm;
mod red_louse;
mod red_slaver;
mod small_acid_slime;
mod small_spike_slime;

use crate::cards::Card;
use crate::rng::Rng;
use crate::status::StatusEffect;
use crate::types::Hp;

#[derive(Debug, Clone, PartialEq)]
pub enum EnemyKind {
    Fungibeast,
    Cultist,
    JawWorm,
    SmallSpikeSlime,
    RedLouse,
    GreenLouse,
    SmallAcidSlime,
    BlueSlaver,
    RedSlaver,
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
    // Green Louse
    GreenBite,
    SpitWeb,
    // Small Acid Slime
    AcidTackle,
    Lick,
    // Blue Slaver
    BlueStab,
    Rake,
    // Red Slaver
    RedStab,
    Scrape,
    SlaveEntangle,
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
            Move::RedLouseBite  => MoveDef { name: "Bite",          effects: vec![Effect::DealDamage(6)] },
            Move::Grow          => MoveDef { name: "Grow",          effects: vec![Effect::GainStatus(StatusEffect::Strength, 3)] },
            Move::GreenBite     => MoveDef { name: "Bite",          effects: vec![Effect::DealDamage(6)] },
            Move::SpitWeb       => MoveDef { name: "Spit Web",      effects: vec![Effect::ApplyStatus(StatusEffect::Weak, 2)] },
            Move::AcidTackle    => MoveDef { name: "Tackle",        effects: vec![Effect::DealDamage(3)] },
            Move::Lick          => MoveDef { name: "Lick",          effects: vec![Effect::ApplyStatus(StatusEffect::Weak, 1)] },
            Move::BlueStab      => MoveDef { name: "Stab",          effects: vec![Effect::DealDamage(12)] },
            Move::Rake          => MoveDef { name: "Rake",          effects: vec![Effect::DealDamage(7), Effect::ApplyStatus(StatusEffect::Weak, 1)] },
            Move::RedStab       => MoveDef { name: "Stab",          effects: vec![Effect::DealDamage(13)] },
            Move::Scrape        => MoveDef { name: "Scrape",        effects: vec![Effect::DealDamage(8), Effect::ApplyStatus(StatusEffect::Vulnerable, 1)] },
            Move::SlaveEntangle => MoveDef { name: "Entangle",      effects: vec![Effect::ApplyStatus(StatusEffect::Entangle, 1)] },
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
        let debuffs_player = effects.iter().any(|e| matches!(e, Effect::ApplyStatus(_, _)));
        match (damage, block, buffs_self, debuffs_player) {
            (d, b, _, _)         if d > 0 && b > 0 => Intent::AttackDefend(d, b),
            (d, _, _, _)         if d > 0           => Intent::Attack(d),
            (_, b, false, false) if b > 0           => Intent::Defend(b),
            (0, 0, false, true)                     => Intent::Debuff,
            _                                       => Intent::Buff,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Intent {
    Attack(i32),
    Defend(i32),
    AttackDefend(i32, i32),
    Buff,
    Debuff,
}

pub struct EnemyDef {
    pub name: &'static str,
    pub max_hp: Hp,
}

impl EnemyKind {
    pub fn def(&self) -> EnemyDef {
        match self {
            EnemyKind::Fungibeast      => fungibeast::DEF,
            EnemyKind::Cultist         => cultist::DEF,
            EnemyKind::JawWorm         => jaw_worm::DEF,
            EnemyKind::SmallSpikeSlime => small_spike_slime::DEF,
            EnemyKind::RedLouse        => red_louse::DEF,
            EnemyKind::GreenLouse      => green_louse::DEF,
            EnemyKind::SmallAcidSlime  => small_acid_slime::DEF,
            EnemyKind::BlueSlaver      => blue_slaver::DEF,
            EnemyKind::RedSlaver       => red_slaver::DEF,
        }
    }

    pub fn name(&self) -> &'static str { self.def().name }
    pub fn max_hp(&self) -> Hp { self.def().max_hp }

    pub fn id(&self) -> &'static str {
        match self {
            EnemyKind::Fungibeast      => "fungibeast",
            EnemyKind::Cultist         => "cultist",
            EnemyKind::JawWorm         => "jaw-worm",
            EnemyKind::SmallSpikeSlime => "small-spike-slime",
            EnemyKind::RedLouse        => "red-louse",
            EnemyKind::GreenLouse      => "green-louse",
            EnemyKind::SmallAcidSlime  => "small-acid-slime",
            EnemyKind::BlueSlaver      => "blue-slaver",
            EnemyKind::RedSlaver       => "red-slaver",
        }
    }

    pub fn from_id(s: &str) -> Option<EnemyKind> {
        match s {
            "fungibeast"       => Some(EnemyKind::Fungibeast),
            "cultist"          => Some(EnemyKind::Cultist),
            "jaw-worm"         => Some(EnemyKind::JawWorm),
            "small-spike-slime" => Some(EnemyKind::SmallSpikeSlime),
            "red-louse"        => Some(EnemyKind::RedLouse),
            "green-louse"      => Some(EnemyKind::GreenLouse),
            "small-acid-slime" => Some(EnemyKind::SmallAcidSlime),
            "blue-slaver"      => Some(EnemyKind::BlueSlaver),
            "red-slaver"       => Some(EnemyKind::RedSlaver),
            _                  => None,
        }
    }
}

pub fn next_move(kind: &EnemyKind, last: Option<Move>, rng: &mut impl Rng) -> Move {
    match kind {
        EnemyKind::Fungibeast      => fungibeast::next_move(last),
        EnemyKind::Cultist         => cultist::next_move(last),
        EnemyKind::JawWorm         => jaw_worm::next_move(last, rng),
        EnemyKind::SmallSpikeSlime => small_spike_slime::next_move(last),
        EnemyKind::RedLouse        => red_louse::next_move(last, rng),
        EnemyKind::GreenLouse      => green_louse::next_move(last, rng),
        EnemyKind::SmallAcidSlime  => small_acid_slime::next_move(last),
        EnemyKind::BlueSlaver      => blue_slaver::next_move(last, rng),
        EnemyKind::RedSlaver       => red_slaver::next_move(last, rng),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::NoOpRng;

    fn rng() -> NoOpRng { NoOpRng }

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

    // --- Enemy IDs ---

    #[test]
    fn all_enemy_ids_round_trip() {
        let kinds = [
            EnemyKind::Fungibeast,
            EnemyKind::Cultist,
            EnemyKind::JawWorm,
            EnemyKind::SmallSpikeSlime,
            EnemyKind::RedLouse,
            EnemyKind::GreenLouse,
            EnemyKind::SmallAcidSlime,
            EnemyKind::BlueSlaver,
            EnemyKind::RedSlaver,
        ];
        for kind in &kinds {
            let id = kind.id();
            assert_eq!(EnemyKind::from_id(id), Some(kind.clone()), "round-trip failed for {id}");
        }
    }

    #[test]
    fn unknown_enemy_id_returns_none() {
        assert_eq!(EnemyKind::from_id("dragon"), None);
        assert_eq!(EnemyKind::from_id(""), None);
    }

    #[test]
    fn enemy_ids_are_kebab_case() {
        assert_eq!(EnemyKind::Fungibeast.id(), "fungibeast");
        assert_eq!(EnemyKind::Cultist.id(), "cultist");
        assert_eq!(EnemyKind::JawWorm.id(), "jaw-worm");
        assert_eq!(EnemyKind::SmallSpikeSlime.id(), "small-spike-slime");
        assert_eq!(EnemyKind::RedLouse.id(), "red-louse");
        assert_eq!(EnemyKind::GreenLouse.id(), "green-louse");
        assert_eq!(EnemyKind::SmallAcidSlime.id(), "small-acid-slime");
        assert_eq!(EnemyKind::BlueSlaver.id(), "blue-slaver");
        assert_eq!(EnemyKind::RedSlaver.id(), "red-slaver");
    }

    // --- Green Louse ---

    #[test]
    fn green_louse_has_12_hp() {
        assert_eq!(EnemyKind::GreenLouse.max_hp(), Hp(12));
    }

    #[test]
    fn green_louse_bites_on_first_turn() {
        assert_eq!(next_move(&EnemyKind::GreenLouse, None, &mut rng()), Move::GreenBite);
    }

    #[test]
    fn green_louse_never_repeats_spit_web() {
        let next = next_move(&EnemyKind::GreenLouse, Some(Move::SpitWeb), &mut rng());
        assert_ne!(next, Move::SpitWeb);
    }

    #[test]
    fn green_bite_is_attack_6() {
        assert_eq!(Move::GreenBite.intent(), Intent::Attack(6));
    }

    #[test]
    fn spit_web_is_debuff() {
        assert_eq!(Move::SpitWeb.intent(), Intent::Debuff);
    }

    // --- Small Acid Slime ---

    #[test]
    fn small_acid_slime_has_10_hp() {
        assert_eq!(EnemyKind::SmallAcidSlime.max_hp(), Hp(10));
    }

    #[test]
    fn small_acid_slime_tackles_on_first_turn() {
        assert_eq!(next_move(&EnemyKind::SmallAcidSlime, None, &mut rng()), Move::AcidTackle);
    }

    #[test]
    fn small_acid_slime_licks_after_tackle() {
        assert_eq!(next_move(&EnemyKind::SmallAcidSlime, Some(Move::AcidTackle), &mut rng()), Move::Lick);
    }

    #[test]
    fn small_acid_slime_tackles_after_lick() {
        assert_eq!(next_move(&EnemyKind::SmallAcidSlime, Some(Move::Lick), &mut rng()), Move::AcidTackle);
    }

    #[test]
    fn acid_tackle_is_attack_3() {
        assert_eq!(Move::AcidTackle.intent(), Intent::Attack(3));
    }

    #[test]
    fn lick_is_debuff() {
        assert_eq!(Move::Lick.intent(), Intent::Debuff);
    }

    // --- Blue Slaver ---

    #[test]
    fn blue_slaver_has_48_hp() {
        assert_eq!(EnemyKind::BlueSlaver.max_hp(), Hp(48));
    }

    #[test]
    fn blue_slaver_stabs_on_first_turn() {
        assert_eq!(next_move(&EnemyKind::BlueSlaver, None, &mut rng()), Move::BlueStab);
    }

    #[test]
    fn blue_slaver_never_repeats_last_move() {
        for last in [Move::BlueStab, Move::Rake] {
            let next = next_move(&EnemyKind::BlueSlaver, Some(last), &mut rng());
            assert_ne!(next, last, "repeated {last:?}");
        }
    }

    #[test]
    fn blue_stab_is_attack_12() {
        assert_eq!(Move::BlueStab.intent(), Intent::Attack(12));
    }

    #[test]
    fn rake_is_attack_7() {
        assert_eq!(Move::Rake.intent(), Intent::Attack(7));
    }

    // --- Red Slaver ---

    #[test]
    fn red_slaver_has_48_hp() {
        assert_eq!(EnemyKind::RedSlaver.max_hp(), Hp(48));
    }

    #[test]
    fn red_slaver_stabs_on_first_turn() {
        assert_eq!(next_move(&EnemyKind::RedSlaver, None, &mut rng()), Move::RedStab);
    }

    #[test]
    fn red_slaver_never_repeats_last_move() {
        for last in [Move::RedStab, Move::Scrape, Move::SlaveEntangle] {
            let next = next_move(&EnemyKind::RedSlaver, Some(last), &mut rng());
            assert_ne!(next, last, "repeated {last:?}");
        }
    }

    #[test]
    fn red_slaver_never_uses_entangle_twice() {
        let next = next_move(&EnemyKind::RedSlaver, Some(Move::SlaveEntangle), &mut rng());
        assert_ne!(next, Move::SlaveEntangle);
    }

    #[test]
    fn red_stab_is_attack_13() {
        assert_eq!(Move::RedStab.intent(), Intent::Attack(13));
    }

    #[test]
    fn scrape_is_attack_8() {
        assert_eq!(Move::Scrape.intent(), Intent::Attack(8));
    }

    #[test]
    fn slave_entangle_is_debuff() {
        assert_eq!(Move::SlaveEntangle.intent(), Intent::Debuff);
    }
}

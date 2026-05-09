mod blue_slaver;
mod cultist;
mod fungibeast;
mod green_louse;
mod gremlin_nob;
mod jaw_worm;
mod lagavulin;
mod large_acid_slime;
mod large_spike_slime;
mod medium_acid_slime;
mod medium_spike_slime;
mod red_louse;
mod red_slaver;
mod small_acid_slime;
mod small_spike_slime;
mod the_guardian;

use crate::cards::Card;
use crate::rng::Rng;
use crate::status::{StatusEffect, StatusMap};
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
    TheGuardian,
    GremlinNob,
    Lagavulin,
    LargeSpike,
    MediumSpike,
    LargeAcid,
    MediumAcid,
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
    // The Guardian
    GuardianChargingUp,
    GuardianFierceBash,
    GuardianVentSteam,
    GuardianWhirlwind,
    GuardianRollAttack,
    GuardianTwinSlam,
    // Gremlin Nob
    NobBellow,
    SkullBash,
    BullRush,
    // Lagavulin
    LagavulinSleep,
    LagavulinStunned,
    LagavulinAttack,
    LagavulinSiphonSoul,
    // Spike Slime (L)
    LargeSpikeFlameTackle,
    LargeSpikeLick,
    LargeSpikeSplit,
    // Spike Slime (M)
    MediumSpikeFlameTackle,
    MediumSpikeLick,
    // Acid Slime (L)
    LargeAcidCorrosiveSpit,
    LargeAcidLick,
    LargeAcidTackle,
    LargeAcidSplit,
    // Acid Slime (M)
    MediumAcidCorrosiveSpit,
    MediumAcidLick,
    MediumAcidTackle,
}

#[derive(Debug, Clone)]
pub enum Effect {
    DealDamage(i32),
    GainBlock(i32),
    GainStatus(StatusEffect, i32),      // applies to self
    ApplyStatus(StatusEffect, i32),     // applies to player
    AddToDiscard(Card),                 // adds a card to the player's discard pile
    ClearSelfStatus(StatusEffect),      // removes all stacks of a status from self
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
            Move::GuardianChargingUp  => MoveDef { name: "Charging Up",  effects: vec![Effect::GainBlock(9)] },
            Move::GuardianFierceBash  => MoveDef { name: "Fierce Bash",  effects: vec![Effect::DealDamage(32)] },
            Move::GuardianVentSteam   => MoveDef { name: "Vent Steam",   effects: vec![Effect::ApplyStatus(StatusEffect::Weak, 2), Effect::ApplyStatus(StatusEffect::Vulnerable, 2)] },
            Move::GuardianWhirlwind   => MoveDef { name: "Whirlwind",    effects: vec![Effect::DealDamage(5), Effect::DealDamage(5), Effect::DealDamage(5), Effect::DealDamage(5)] },
            Move::GuardianRollAttack  => MoveDef { name: "Roll Attack",  effects: vec![Effect::DealDamage(9)] },
            Move::GuardianTwinSlam    => MoveDef { name: "Twin Slam",    effects: vec![Effect::DealDamage(8), Effect::DealDamage(8), Effect::ClearSelfStatus(StatusEffect::SharpHide), Effect::ClearSelfStatus(StatusEffect::GuardianMode)] },
            Move::NobBellow  => MoveDef { name: "Bellow",     effects: vec![Effect::GainStatus(StatusEffect::Enrage, 2)] },
            Move::SkullBash  => MoveDef { name: "Skull Bash", effects: vec![Effect::DealDamage(6), Effect::ApplyStatus(StatusEffect::Vulnerable, 2)] },
            Move::BullRush   => MoveDef { name: "Bull Rush",  effects: vec![Effect::DealDamage(14)] },
            Move::LagavulinSleep      => MoveDef { name: "Sleep",       effects: vec![] },
            Move::LagavulinStunned    => MoveDef { name: "Stunned",     effects: vec![] },
            Move::LagavulinAttack     => MoveDef { name: "Attack",      effects: vec![Effect::DealDamage(18)] },
            Move::LagavulinSiphonSoul => MoveDef { name: "Siphon Soul", effects: vec![Effect::ApplyStatus(StatusEffect::Strength, -1), Effect::ApplyStatus(StatusEffect::Dexterity, -1)] },
            Move::LargeSpikeFlameTackle  => MoveDef { name: "Flame Tackle",    effects: vec![Effect::DealDamage(16), Effect::AddToDiscard(Card::Slimed), Effect::AddToDiscard(Card::Slimed)] },
            Move::LargeSpikeLick         => MoveDef { name: "Lick",            effects: vec![Effect::ApplyStatus(StatusEffect::Frail, 2)] },
            Move::LargeSpikeSplit        => MoveDef { name: "Split",           effects: vec![] },
            Move::MediumSpikeFlameTackle => MoveDef { name: "Flame Tackle",    effects: vec![Effect::DealDamage(8), Effect::AddToDiscard(Card::Slimed)] },
            Move::MediumSpikeLick        => MoveDef { name: "Lick",            effects: vec![Effect::ApplyStatus(StatusEffect::Frail, 1)] },
            Move::LargeAcidCorrosiveSpit => MoveDef { name: "Corrosive Spit",  effects: vec![Effect::DealDamage(11), Effect::AddToDiscard(Card::Slimed), Effect::AddToDiscard(Card::Slimed)] },
            Move::LargeAcidLick          => MoveDef { name: "Lick",            effects: vec![Effect::ApplyStatus(StatusEffect::Weak, 2)] },
            Move::LargeAcidTackle        => MoveDef { name: "Tackle",          effects: vec![Effect::DealDamage(16)] },
            Move::LargeAcidSplit         => MoveDef { name: "Split",           effects: vec![] },
            Move::MediumAcidCorrosiveSpit => MoveDef { name: "Corrosive Spit", effects: vec![Effect::DealDamage(7), Effect::AddToDiscard(Card::Slimed)] },
            Move::MediumAcidLick          => MoveDef { name: "Lick",           effects: vec![Effect::ApplyStatus(StatusEffect::Weak, 1)] },
            Move::MediumAcidTackle        => MoveDef { name: "Tackle",         effects: vec![Effect::DealDamage(10)] },
        }
    }

    pub fn intent(self) -> Intent {
        if matches!(self, Move::LargeSpikeSplit | Move::LargeAcidSplit) {
            return Intent::Split;
        }
        let effects = self.def().effects;
        let damage: i32 = effects.iter().filter_map(|e| {
            if let Effect::DealDamage(n) = e { Some(*n) } else { None }
        }).sum();
        let block: i32 = effects.iter().filter_map(|e| {
            if let Effect::GainBlock(n) = e { Some(*n) } else { None }
        }).sum();
        let buffs_self = effects.iter().any(|e| matches!(e, Effect::GainStatus(_, _)));
        let debuffs_player = effects.iter().any(|e| matches!(e, Effect::ApplyStatus(_, _) | Effect::AddToDiscard(_)));
        match (damage, block, buffs_self, debuffs_player) {
            (d, b, _, _)         if d > 0 && b > 0           => Intent::AttackDefend(d, b),
            (d, _, _, true)      if d > 0                     => Intent::AttackDebuff(d),
            (d, _, _, _)         if d > 0                     => Intent::Attack(d),
            (_, b, false, false) if b > 0                     => Intent::Defend(b),
            (0, 0, false, true)                               => Intent::Debuff,
            _                                                 => Intent::Buff,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Intent {
    Attack(i32),
    AttackDebuff(i32),
    Defend(i32),
    AttackDefend(i32, i32),
    Buff,
    Debuff,
    Split,
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
            EnemyKind::TheGuardian     => the_guardian::DEF,
            EnemyKind::GremlinNob      => gremlin_nob::DEF,
            EnemyKind::Lagavulin       => lagavulin::DEF,
            EnemyKind::LargeSpike      => large_spike_slime::DEF,
            EnemyKind::MediumSpike     => medium_spike_slime::DEF,
            EnemyKind::LargeAcid       => large_acid_slime::DEF,
            EnemyKind::MediumAcid      => medium_acid_slime::DEF,
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
            EnemyKind::TheGuardian     => "the-guardian",
            EnemyKind::GremlinNob      => "gremlin-nob",
            EnemyKind::Lagavulin       => "lagavulin",
            EnemyKind::LargeSpike      => "large-spike-slime",
            EnemyKind::MediumSpike     => "medium-spike-slime",
            EnemyKind::LargeAcid       => "large-acid-slime",
            EnemyKind::MediumAcid      => "medium-acid-slime",
        }
    }

    pub fn from_id(s: &str) -> Option<EnemyKind> {
        match s {
            "fungibeast"        => Some(EnemyKind::Fungibeast),
            "cultist"           => Some(EnemyKind::Cultist),
            "jaw-worm"          => Some(EnemyKind::JawWorm),
            "small-spike-slime" => Some(EnemyKind::SmallSpikeSlime),
            "red-louse"         => Some(EnemyKind::RedLouse),
            "green-louse"       => Some(EnemyKind::GreenLouse),
            "small-acid-slime"  => Some(EnemyKind::SmallAcidSlime),
            "blue-slaver"       => Some(EnemyKind::BlueSlaver),
            "red-slaver"        => Some(EnemyKind::RedSlaver),
            "the-guardian"      => Some(EnemyKind::TheGuardian),
            "gremlin-nob"        => Some(EnemyKind::GremlinNob),
            "lagavulin"          => Some(EnemyKind::Lagavulin),
            "large-spike-slime"  => Some(EnemyKind::LargeSpike),
            "medium-spike-slime" => Some(EnemyKind::MediumSpike),
            "large-acid-slime"   => Some(EnemyKind::LargeAcid),
            "medium-acid-slime"  => Some(EnemyKind::MediumAcid),
            _                    => None,
        }
    }
}

/// Returned by `on_player_attack_damage` to describe mutations + events from an enemy reaction.
pub struct EnemyDamageReaction {
    pub block_gain: i32,
    pub status_events: Vec<(StatusEffect, i32)>,  // applied + emit StatusApplied events
    pub silent_adds: Vec<(StatusEffect, i32)>,     // applied silently (add stacks)
    pub silent_sets: Vec<(StatusEffect, i32)>,     // applied silently (set absolute value)
    pub force_move: Option<Move>,
}

/// Called after a player card deals HP damage to `kind`. Returns a reaction if the enemy
/// has any damage-triggered behaviour, `None` otherwise.
pub fn on_player_attack_damage(
    kind: &EnemyKind,
    statuses: &StatusMap,
    hp_lost: i32,
    current_hp: Hp,
    max_hp: Hp,
) -> Option<EnemyDamageReaction> {
    match kind {
        EnemyKind::TheGuardian => the_guardian::on_player_attack_damage(statuses, hp_lost),
        EnemyKind::LargeSpike  => large_spike_slime::on_player_attack_damage(current_hp, max_hp),
        EnemyKind::LargeAcid   => large_acid_slime::on_player_attack_damage(current_hp, max_hp),
        _ => None,
    }
}

pub fn next_move(kind: &EnemyKind, history: &[Move], statuses: &StatusMap, rng: &mut impl Rng) -> Move {
    let last = history.last().copied();
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
        EnemyKind::TheGuardian     => the_guardian::next_move(last),
        EnemyKind::GremlinNob      => gremlin_nob::next_move(last, rng),
        EnemyKind::Lagavulin       => lagavulin::next_move(history, statuses),
        EnemyKind::LargeSpike      => large_spike_slime::next_move(history, rng),
        EnemyKind::MediumSpike     => medium_spike_slime::next_move(history, rng),
        EnemyKind::LargeAcid       => large_acid_slime::next_move(history, rng),
        EnemyKind::MediumAcid      => medium_acid_slime::next_move(history, rng),
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
        assert_eq!(next_move(&EnemyKind::Fungibeast, &[], &StatusMap::new(), &mut rng()), Move::FungiLight);
    }

    #[test]
    fn fungibeast_heavy_after_light() {
        assert_eq!(next_move(&EnemyKind::Fungibeast, &[Move::FungiLight], &StatusMap::new(), &mut rng()), Move::FungiHeavy);
    }

    #[test]
    fn fungibeast_light_after_heavy() {
        assert_eq!(next_move(&EnemyKind::Fungibeast, &[Move::FungiHeavy], &StatusMap::new(), &mut rng()), Move::FungiLight);
    }

    #[test]
    fn cultist_has_50_hp() {
        assert_eq!(EnemyKind::Cultist.max_hp(), Hp(50));
    }

    #[test]
    fn cultist_incantation_on_first_turn() {
        assert_eq!(next_move(&EnemyKind::Cultist, &[], &StatusMap::new(), &mut rng()), Move::Incantation);
    }

    #[test]
    fn cultist_dark_strike_after_incantation() {
        assert_eq!(next_move(&EnemyKind::Cultist, &[Move::Incantation], &StatusMap::new(), &mut rng()), Move::DarkStrike);
    }

    #[test]
    fn cultist_dark_strike_repeats() {
        assert_eq!(next_move(&EnemyKind::Cultist, &[Move::DarkStrike], &StatusMap::new(), &mut rng()), Move::DarkStrike);
    }

    #[test]
    fn jaw_worm_has_40_hp() {
        assert_eq!(EnemyKind::JawWorm.max_hp(), Hp(40));
    }

    #[test]
    fn jaw_worm_chomps_on_first_turn() {
        assert_eq!(next_move(&EnemyKind::JawWorm, &[], &StatusMap::new(), &mut rng()), Move::Chomp);
    }

    #[test]
    fn jaw_worm_never_repeats_last_move() {
        for last in [Move::Chomp, Move::Thrash, Move::Bellow] {
            let next = next_move(&EnemyKind::JawWorm, &[last], &StatusMap::new(), &mut rng());
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
        assert_eq!(next_move(&EnemyKind::SmallSpikeSlime, &[], &StatusMap::new(), &mut rng()), Move::FlameTackle);
        assert_eq!(next_move(&EnemyKind::SmallSpikeSlime, &[Move::FlameTackle], &StatusMap::new(), &mut rng()), Move::FlameTackle);
    }

    #[test]
    fn flame_tackle_intent_is_attack_debuff_5() {
        assert_eq!(Move::FlameTackle.intent(), Intent::AttackDebuff(5));
    }

    #[test]
    fn red_louse_has_12_hp() {
        assert_eq!(EnemyKind::RedLouse.max_hp(), Hp(12));
    }

    #[test]
    fn red_louse_bites_on_first_turn() {
        assert_eq!(next_move(&EnemyKind::RedLouse, &[], &StatusMap::new(), &mut rng()), Move::RedLouseBite);
    }

    #[test]
    fn red_louse_bites_after_grow() {
        assert_eq!(next_move(&EnemyKind::RedLouse, &[Move::Grow], &StatusMap::new(), &mut rng()), Move::RedLouseBite);
    }

    #[test]
    fn red_louse_never_repeats_grow() {
        let next = next_move(&EnemyKind::RedLouse, &[Move::Grow], &StatusMap::new(), &mut rng());
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
        assert_eq!(next_move(&EnemyKind::GreenLouse, &[], &StatusMap::new(), &mut rng()), Move::GreenBite);
    }

    #[test]
    fn green_louse_never_repeats_spit_web() {
        let next = next_move(&EnemyKind::GreenLouse, &[Move::SpitWeb], &StatusMap::new(), &mut rng());
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
        assert_eq!(next_move(&EnemyKind::SmallAcidSlime, &[], &StatusMap::new(), &mut rng()), Move::AcidTackle);
    }

    #[test]
    fn small_acid_slime_licks_after_tackle() {
        assert_eq!(next_move(&EnemyKind::SmallAcidSlime, &[Move::AcidTackle], &StatusMap::new(), &mut rng()), Move::Lick);
    }

    #[test]
    fn small_acid_slime_tackles_after_lick() {
        assert_eq!(next_move(&EnemyKind::SmallAcidSlime, &[Move::Lick], &StatusMap::new(), &mut rng()), Move::AcidTackle);
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
        assert_eq!(next_move(&EnemyKind::BlueSlaver, &[], &StatusMap::new(), &mut rng()), Move::BlueStab);
    }

    #[test]
    fn blue_slaver_never_repeats_last_move() {
        for last in [Move::BlueStab, Move::Rake] {
            let next = next_move(&EnemyKind::BlueSlaver, &[last], &StatusMap::new(), &mut rng());
            assert_ne!(next, last, "repeated {last:?}");
        }
    }

    #[test]
    fn blue_stab_is_attack_12() {
        assert_eq!(Move::BlueStab.intent(), Intent::Attack(12));
    }

    #[test]
    fn rake_is_attack_debuff_7() {
        assert_eq!(Move::Rake.intent(), Intent::AttackDebuff(7));
    }

    // --- Red Slaver ---

    #[test]
    fn red_slaver_has_48_hp() {
        assert_eq!(EnemyKind::RedSlaver.max_hp(), Hp(48));
    }

    #[test]
    fn red_slaver_stabs_on_first_turn() {
        assert_eq!(next_move(&EnemyKind::RedSlaver, &[], &StatusMap::new(), &mut rng()), Move::RedStab);
    }

    #[test]
    fn red_slaver_never_repeats_last_move() {
        for last in [Move::RedStab, Move::Scrape, Move::SlaveEntangle] {
            let next = next_move(&EnemyKind::RedSlaver, &[last], &StatusMap::new(), &mut rng());
            assert_ne!(next, last, "repeated {last:?}");
        }
    }

    #[test]
    fn red_slaver_never_uses_entangle_twice() {
        let next = next_move(&EnemyKind::RedSlaver, &[Move::SlaveEntangle], &StatusMap::new(), &mut rng());
        assert_ne!(next, Move::SlaveEntangle);
    }

    #[test]
    fn red_stab_is_attack_13() {
        assert_eq!(Move::RedStab.intent(), Intent::Attack(13));
    }

    #[test]
    fn scrape_is_attack_debuff_8() {
        assert_eq!(Move::Scrape.intent(), Intent::AttackDebuff(8));
    }

    #[test]
    fn slave_entangle_is_debuff() {
        assert_eq!(Move::SlaveEntangle.intent(), Intent::Debuff);
    }

    // --- The Guardian ---

    #[test]
    fn guardian_has_240_hp() {
        assert_eq!(EnemyKind::TheGuardian.max_hp(), Hp(240));
    }

    #[test]
    fn guardian_name_is_the_guardian() {
        assert_eq!(EnemyKind::TheGuardian.name(), "The Guardian");
    }

    #[test]
    fn guardian_charging_up_is_buff() {
        assert_eq!(Move::GuardianChargingUp.intent(), Intent::Defend(9));
    }

    #[test]
    fn guardian_fierce_bash_is_attack_32() {
        assert_eq!(Move::GuardianFierceBash.intent(), Intent::Attack(32));
    }

    #[test]
    fn guardian_vent_steam_is_debuff() {
        assert_eq!(Move::GuardianVentSteam.intent(), Intent::Debuff);
    }

    #[test]
    fn guardian_whirlwind_is_attack_20() {
        assert_eq!(Move::GuardianWhirlwind.intent(), Intent::Attack(20));
    }

    #[test]
    fn guardian_roll_attack_is_attack_9() {
        assert_eq!(Move::GuardianRollAttack.intent(), Intent::Attack(9));
    }

    #[test]
    fn guardian_twin_slam_is_attack_16() {
        assert_eq!(Move::GuardianTwinSlam.intent(), Intent::Attack(16));
    }

    #[test]
    fn guardian_first_move_is_charging_up() {
        assert_eq!(next_move(&EnemyKind::TheGuardian, &[], &StatusMap::new(), &mut rng()), Move::GuardianChargingUp);
    }

    #[test]
    fn guardian_fierce_bash_after_charging_up() {
        assert_eq!(next_move(&EnemyKind::TheGuardian, &[Move::GuardianChargingUp], &StatusMap::new(), &mut rng()), Move::GuardianFierceBash);
    }

    #[test]
    fn guardian_vent_steam_after_fierce_bash() {
        assert_eq!(next_move(&EnemyKind::TheGuardian, &[Move::GuardianFierceBash], &StatusMap::new(), &mut rng()), Move::GuardianVentSteam);
    }

    #[test]
    fn guardian_whirlwind_after_vent_steam() {
        assert_eq!(next_move(&EnemyKind::TheGuardian, &[Move::GuardianVentSteam], &StatusMap::new(), &mut rng()), Move::GuardianWhirlwind);
    }

    #[test]
    fn guardian_charging_up_after_whirlwind() {
        assert_eq!(next_move(&EnemyKind::TheGuardian, &[Move::GuardianWhirlwind], &StatusMap::new(), &mut rng()), Move::GuardianChargingUp);
    }

    #[test]
    fn guardian_roll_attack_after_twin_slam_setup() {
        assert_eq!(next_move(&EnemyKind::TheGuardian, &[Move::GuardianRollAttack], &StatusMap::new(), &mut rng()), Move::GuardianTwinSlam);
    }

    #[test]
    fn guardian_twin_slam_leads_to_whirlwind() {
        assert_eq!(next_move(&EnemyKind::TheGuardian, &[Move::GuardianTwinSlam], &StatusMap::new(), &mut rng()), Move::GuardianWhirlwind);
    }

    // --- Gremlin Nob ---

    #[test]
    fn gremlin_nob_has_84_hp() {
        assert_eq!(EnemyKind::GremlinNob.max_hp(), Hp(84));
    }

    #[test]
    fn gremlin_nob_name_is_gremlin_nob() {
        assert_eq!(EnemyKind::GremlinNob.name(), "Gremlin Nob");
    }

    #[test]
    fn gremlin_nob_first_move_is_bellow() {
        assert_eq!(next_move(&EnemyKind::GremlinNob, &[], &StatusMap::new(), &mut rng()), Move::NobBellow);
    }

    #[test]
    fn gremlin_nob_never_bellows_twice() {
        let next = next_move(&EnemyKind::GremlinNob, &[Move::NobBellow], &StatusMap::new(), &mut rng());
        assert_ne!(next, Move::NobBellow);
    }

    #[test]
    fn gremlin_nob_after_bellow_uses_skull_bash_or_bull_rush() {
        let next = next_move(&EnemyKind::GremlinNob, &[Move::NobBellow], &StatusMap::new(), &mut rng());
        assert!(
            matches!(next, Move::SkullBash | Move::BullRush),
            "expected SkullBash or BullRush, got {next:?}"
        );
    }

    #[test]
    fn gremlin_nob_never_repeats_last_move() {
        for last in [Move::SkullBash, Move::BullRush] {
            let next = next_move(&EnemyKind::GremlinNob, &[last], &StatusMap::new(), &mut rng());
            assert_ne!(next, last, "repeated {last:?}");
        }
    }

    #[test]
    fn gremlin_nob_id_round_trips() {
        assert_eq!(EnemyKind::from_id("gremlin-nob"), Some(EnemyKind::GremlinNob));
        assert_eq!(EnemyKind::GremlinNob.id(), "gremlin-nob");
    }

    // --- Gremlin Nob moves ---

    #[test]
    fn nob_bellow_intent_is_buff() {
        assert_eq!(Move::NobBellow.intent(), Intent::Buff);
    }

    #[test]
    fn skull_bash_intent_is_attack_debuff_6() {
        assert_eq!(Move::SkullBash.intent(), Intent::AttackDebuff(6));
    }

    #[test]
    fn bull_rush_intent_is_attack_14() {
        assert_eq!(Move::BullRush.intent(), Intent::Attack(14));
    }

    #[test]
    fn nob_bellow_name_is_bellow() {
        assert_eq!(Move::NobBellow.def().name, "Bellow");
    }

    #[test]
    fn skull_bash_name_is_skull_bash() {
        assert_eq!(Move::SkullBash.def().name, "Skull Bash");
    }

    #[test]
    fn bull_rush_name_is_bull_rush() {
        assert_eq!(Move::BullRush.def().name, "Bull Rush");
    }

    // --- Lagavulin ---

    fn sleeping_statuses() -> StatusMap {
        let mut m = StatusMap::new();
        m.insert(StatusEffect::Sleep, 3);
        m
    }

    #[test]
    fn lagavulin_has_109_hp() {
        assert_eq!(EnemyKind::Lagavulin.max_hp(), Hp(109));
    }

    #[test]
    fn lagavulin_is_named_lagavulin() {
        assert_eq!(EnemyKind::Lagavulin.name(), "Lagavulin");
    }

    #[test]
    fn lagavulin_first_move_is_sleep() {
        assert_eq!(next_move(&EnemyKind::Lagavulin, &[], &sleeping_statuses(), &mut rng()), Move::LagavulinSleep);
    }

    #[test]
    fn lagavulin_stays_sleeping_while_sleep_active() {
        let statuses = sleeping_statuses();
        assert_eq!(next_move(&EnemyKind::Lagavulin, &[Move::LagavulinSleep], &statuses, &mut rng()), Move::LagavulinSleep);
    }

    #[test]
    fn lagavulin_wakes_naturally_when_sleep_expires() {
        assert_eq!(next_move(&EnemyKind::Lagavulin, &[Move::LagavulinSleep], &StatusMap::new(), &mut rng()), Move::LagavulinAttack);
    }

    #[test]
    fn lagavulin_attacks_after_stun() {
        assert_eq!(next_move(&EnemyKind::Lagavulin, &[Move::LagavulinStunned], &StatusMap::new(), &mut rng()), Move::LagavulinAttack);
    }

    #[test]
    fn lagavulin_awake_cycle_first_attack_then_second() {
        assert_eq!(next_move(&EnemyKind::Lagavulin, &[Move::LagavulinSiphonSoul, Move::LagavulinAttack], &StatusMap::new(), &mut rng()), Move::LagavulinAttack);
    }

    #[test]
    fn lagavulin_awake_cycle_second_attack_then_siphon() {
        assert_eq!(next_move(&EnemyKind::Lagavulin, &[Move::LagavulinAttack, Move::LagavulinAttack], &StatusMap::new(), &mut rng()), Move::LagavulinSiphonSoul);
    }

    #[test]
    fn lagavulin_awake_cycle_siphon_then_attack() {
        assert_eq!(next_move(&EnemyKind::Lagavulin, &[Move::LagavulinSiphonSoul], &StatusMap::new(), &mut rng()), Move::LagavulinAttack);
    }

    #[test]
    fn lagavulin_attack_deals_18_damage() {
        assert_eq!(Move::LagavulinAttack.intent(), Intent::Attack(18));
    }

    #[test]
    fn lagavulin_siphon_soul_is_debuff() {
        assert_eq!(Move::LagavulinSiphonSoul.intent(), Intent::Debuff);
    }

    #[test]
    fn lagavulin_sleep_is_buff() {
        assert_eq!(Move::LagavulinSleep.intent(), Intent::Buff);
    }

    #[test]
    fn lagavulin_id_round_trips() {
        assert_eq!(EnemyKind::from_id("lagavulin"), Some(EnemyKind::Lagavulin));
        assert_eq!(EnemyKind::Lagavulin.id(), "lagavulin");
    }

    // --- Large Spike Slime ---

    #[test]
    fn large_spike_slime_has_67_hp() {
        assert_eq!(EnemyKind::LargeSpike.max_hp(), Hp(67));
    }

    #[test]
    fn large_spike_slime_is_named_correctly() {
        assert_eq!(EnemyKind::LargeSpike.name(), "Spike Slime (L)");
    }

    #[test]
    fn large_spike_slime_id_round_trips() {
        assert_eq!(EnemyKind::LargeSpike.id(), "large-spike-slime");
        assert_eq!(EnemyKind::from_id("large-spike-slime"), Some(EnemyKind::LargeSpike));
    }

    #[test]
    fn large_spike_first_move_is_flame_tackle_or_lick() {
        let mv = next_move(&EnemyKind::LargeSpike, &[], &StatusMap::new(), &mut rng());
        assert!(matches!(mv, Move::LargeSpikeFlameTackle | Move::LargeSpikeLick));
    }

    #[test]
    fn large_spike_never_repeats_same_move_three_times() {
        for repeated in [Move::LargeSpikeFlameTackle, Move::LargeSpikeLick] {
            let mv = next_move(&EnemyKind::LargeSpike, &[repeated, repeated], &StatusMap::new(), &mut rng());
            assert_ne!(mv, repeated, "should not repeat {repeated:?} a third time");
        }
    }

    #[test]
    fn large_spike_split_at_or_below_half_hp() {
        let reaction = on_player_attack_damage(&EnemyKind::LargeSpike, &StatusMap::new(), 1, Hp(33), Hp(67));
        assert_eq!(reaction.map(|r| r.force_move).flatten(), Some(Move::LargeSpikeSplit));
    }

    #[test]
    fn large_spike_no_split_above_half_hp() {
        let reaction = on_player_attack_damage(&EnemyKind::LargeSpike, &StatusMap::new(), 1, Hp(34), Hp(67));
        assert!(reaction.is_none());
    }

    // --- Medium Spike Slime ---

    #[test]
    fn medium_spike_slime_has_30_hp() {
        assert_eq!(EnemyKind::MediumSpike.max_hp(), Hp(30));
    }

    #[test]
    fn medium_spike_slime_is_named_correctly() {
        assert_eq!(EnemyKind::MediumSpike.name(), "Spike Slime (M)");
    }

    #[test]
    fn medium_spike_slime_id_round_trips() {
        assert_eq!(EnemyKind::MediumSpike.id(), "medium-spike-slime");
        assert_eq!(EnemyKind::from_id("medium-spike-slime"), Some(EnemyKind::MediumSpike));
    }

    #[test]
    fn medium_spike_first_move_is_flame_tackle_or_lick() {
        let mv = next_move(&EnemyKind::MediumSpike, &[], &StatusMap::new(), &mut rng());
        assert!(matches!(mv, Move::MediumSpikeFlameTackle | Move::MediumSpikeLick));
    }

    #[test]
    fn medium_spike_never_repeats_same_move_three_times() {
        for repeated in [Move::MediumSpikeFlameTackle, Move::MediumSpikeLick] {
            let mv = next_move(&EnemyKind::MediumSpike, &[repeated, repeated], &StatusMap::new(), &mut rng());
            assert_ne!(mv, repeated);
        }
    }

    // --- Large Acid Slime ---

    #[test]
    fn large_acid_slime_has_67_hp() {
        assert_eq!(EnemyKind::LargeAcid.max_hp(), Hp(67));
    }

    #[test]
    fn large_acid_slime_is_named_correctly() {
        assert_eq!(EnemyKind::LargeAcid.name(), "Acid Slime (L)");
    }

    #[test]
    fn large_acid_slime_id_round_trips() {
        assert_eq!(EnemyKind::LargeAcid.id(), "large-acid-slime");
        assert_eq!(EnemyKind::from_id("large-acid-slime"), Some(EnemyKind::LargeAcid));
    }

    #[test]
    fn large_acid_first_move_is_one_of_three() {
        let mv = next_move(&EnemyKind::LargeAcid, &[], &StatusMap::new(), &mut rng());
        assert!(matches!(mv, Move::LargeAcidCorrosiveSpit | Move::LargeAcidLick | Move::LargeAcidTackle));
    }

    #[test]
    fn large_acid_never_repeats_same_move_three_times() {
        for repeated in [Move::LargeAcidCorrosiveSpit, Move::LargeAcidLick, Move::LargeAcidTackle] {
            let mv = next_move(&EnemyKind::LargeAcid, &[repeated, repeated], &StatusMap::new(), &mut rng());
            assert_ne!(mv, repeated);
        }
    }

    #[test]
    fn large_acid_split_at_or_below_half_hp() {
        let reaction = on_player_attack_damage(&EnemyKind::LargeAcid, &StatusMap::new(), 1, Hp(33), Hp(67));
        assert_eq!(reaction.map(|r| r.force_move).flatten(), Some(Move::LargeAcidSplit));
    }

    #[test]
    fn large_acid_no_split_above_half_hp() {
        let reaction = on_player_attack_damage(&EnemyKind::LargeAcid, &StatusMap::new(), 1, Hp(34), Hp(67));
        assert!(reaction.is_none());
    }

    // --- Medium Acid Slime ---

    #[test]
    fn medium_acid_slime_has_30_hp() {
        assert_eq!(EnemyKind::MediumAcid.max_hp(), Hp(30));
    }

    #[test]
    fn medium_acid_slime_is_named_correctly() {
        assert_eq!(EnemyKind::MediumAcid.name(), "Acid Slime (M)");
    }

    #[test]
    fn medium_acid_slime_id_round_trips() {
        assert_eq!(EnemyKind::MediumAcid.id(), "medium-acid-slime");
        assert_eq!(EnemyKind::from_id("medium-acid-slime"), Some(EnemyKind::MediumAcid));
    }

    #[test]
    fn medium_acid_first_move_is_one_of_three() {
        let mv = next_move(&EnemyKind::MediumAcid, &[], &StatusMap::new(), &mut rng());
        assert!(matches!(mv, Move::MediumAcidCorrosiveSpit | Move::MediumAcidLick | Move::MediumAcidTackle));
    }

    #[test]
    fn medium_acid_never_repeats_same_move_three_times() {
        for repeated in [Move::MediumAcidCorrosiveSpit, Move::MediumAcidLick, Move::MediumAcidTackle] {
            let mv = next_move(&EnemyKind::MediumAcid, &[repeated, repeated], &StatusMap::new(), &mut rng());
            assert_ne!(mv, repeated);
        }
    }

    // --- Large Spike Slime moves ---

    #[test]
    fn large_spike_flame_tackle_deals_16_dmg_and_adds_2_slimed() {
        let def = Move::LargeSpikeFlameTackle.def();
        let damage: i32 = def.effects.iter().filter_map(|e| if let Effect::DealDamage(n) = e { Some(*n) } else { None }).sum();
        let slimed_count = def.effects.iter().filter(|e| matches!(e, Effect::AddToDiscard(Card::Slimed))).count();
        assert_eq!(damage, 16);
        assert_eq!(slimed_count, 2);
    }

    #[test]
    fn large_spike_lick_applies_2_frail() {
        let def = Move::LargeSpikeLick.def();
        let frail: i32 = def.effects.iter().filter_map(|e| {
            if let Effect::ApplyStatus(StatusEffect::Frail, n) = e { Some(*n) } else { None }
        }).sum();
        assert_eq!(frail, 2);
    }

    #[test]
    fn large_spike_split_intent_is_split() {
        assert_eq!(Move::LargeSpikeSplit.intent(), Intent::Split);
    }

    #[test]
    fn large_spike_flame_tackle_intent_is_attack_debuff_16() {
        assert_eq!(Move::LargeSpikeFlameTackle.intent(), Intent::AttackDebuff(16));
    }

    #[test]
    fn large_spike_lick_intent_is_debuff() {
        assert_eq!(Move::LargeSpikeLick.intent(), Intent::Debuff);
    }

    // --- Medium Spike Slime moves ---

    #[test]
    fn medium_spike_flame_tackle_deals_8_dmg_and_adds_1_slimed() {
        let def = Move::MediumSpikeFlameTackle.def();
        let damage: i32 = def.effects.iter().filter_map(|e| if let Effect::DealDamage(n) = e { Some(*n) } else { None }).sum();
        let slimed_count = def.effects.iter().filter(|e| matches!(e, Effect::AddToDiscard(Card::Slimed))).count();
        assert_eq!(damage, 8);
        assert_eq!(slimed_count, 1);
    }

    #[test]
    fn medium_spike_lick_applies_1_frail() {
        let def = Move::MediumSpikeLick.def();
        let frail: i32 = def.effects.iter().filter_map(|e| {
            if let Effect::ApplyStatus(StatusEffect::Frail, n) = e { Some(*n) } else { None }
        }).sum();
        assert_eq!(frail, 1);
    }

    // --- Large Acid Slime moves ---

    #[test]
    fn large_acid_corrosive_spit_deals_11_dmg_and_adds_2_slimed() {
        let def = Move::LargeAcidCorrosiveSpit.def();
        let damage: i32 = def.effects.iter().filter_map(|e| if let Effect::DealDamage(n) = e { Some(*n) } else { None }).sum();
        let slimed_count = def.effects.iter().filter(|e| matches!(e, Effect::AddToDiscard(Card::Slimed))).count();
        assert_eq!(damage, 11);
        assert_eq!(slimed_count, 2);
    }

    #[test]
    fn large_acid_lick_applies_2_weak() {
        let def = Move::LargeAcidLick.def();
        let weak: i32 = def.effects.iter().filter_map(|e| {
            if let Effect::ApplyStatus(StatusEffect::Weak, n) = e { Some(*n) } else { None }
        }).sum();
        assert_eq!(weak, 2);
    }

    #[test]
    fn large_acid_tackle_deals_16_dmg() {
        let def = Move::LargeAcidTackle.def();
        let damage: i32 = def.effects.iter().filter_map(|e| if let Effect::DealDamage(n) = e { Some(*n) } else { None }).sum();
        assert_eq!(damage, 16);
    }

    #[test]
    fn large_acid_split_intent_is_split() {
        assert_eq!(Move::LargeAcidSplit.intent(), Intent::Split);
    }

    // --- Medium Acid Slime moves ---

    #[test]
    fn medium_acid_corrosive_spit_deals_7_dmg_and_adds_1_slimed() {
        let def = Move::MediumAcidCorrosiveSpit.def();
        let damage: i32 = def.effects.iter().filter_map(|e| if let Effect::DealDamage(n) = e { Some(*n) } else { None }).sum();
        let slimed_count = def.effects.iter().filter(|e| matches!(e, Effect::AddToDiscard(Card::Slimed))).count();
        assert_eq!(damage, 7);
        assert_eq!(slimed_count, 1);
    }

    #[test]
    fn medium_acid_lick_applies_1_weak() {
        let def = Move::MediumAcidLick.def();
        let weak: i32 = def.effects.iter().filter_map(|e| {
            if let Effect::ApplyStatus(StatusEffect::Weak, n) = e { Some(*n) } else { None }
        }).sum();
        assert_eq!(weak, 1);
    }

    #[test]
    fn medium_acid_tackle_deals_10_dmg() {
        let def = Move::MediumAcidTackle.def();
        let damage: i32 = def.effects.iter().filter_map(|e| if let Effect::DealDamage(n) = e { Some(*n) } else { None }).sum();
        assert_eq!(damage, 10);
    }
}

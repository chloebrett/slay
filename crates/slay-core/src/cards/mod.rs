mod all_for_one;
mod all_out_attack;
mod armaments;
mod anger;
mod bash;
mod ascenders_bane;
mod barricade;
mod berserk;
mod burning_pact;
mod brutality;
mod ghostly_armor;
mod combust;
mod evolve;
mod fire_breathing;
mod feed;
mod fiend_fire;
mod flex;
mod perfected_strike;
mod power_through;
mod reaper;
mod whirlwind;
mod immolate;
mod intimidate;
mod shockwave;
mod carnage;
mod clash;
mod dark_embrace;
mod demon_form;
mod burn;
mod clumsy;
mod curse_of_the_bell;
mod decay;
mod doubt;
mod injury;
mod parasite;
mod regret;
mod shame;
mod wound;
mod bloodletting;
mod body_slam;
mod bludgeon;
mod cleave;
mod clothesline;
mod dazed;
mod slimed;
mod deadly_poison;
mod defend;
mod disarm;
mod finesse;
mod flash_of_steel;
mod good_instincts;
mod swift_strike;
mod entrench;
mod feel_no_pain;
mod hemokinesis;
mod impervious;
mod inflame;
mod iron_wave;
mod juggernaut;
mod limit_break;
mod pommel_strike;
mod pummel;
mod reckless_charge;
mod rupture;
mod seeing_red;
mod shrug_it_off;
mod spot_weakness;
mod strike;
mod thunderclap;
mod true_grit;
mod heavy_blade;
mod sword_boomerang;
mod twin_strike;
mod uppercut;
mod searing_blow;
mod second_wind;
mod sentinel;
mod warcry;
mod wild_strike;

use crate::status::{StatusEffect, StatusMap, resolve_damage};
use crate::types::Energy;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Card {
    Strike(Grade),
    Defend(Grade),
    Bash(Grade),
    Clothesline(Grade),
    Inflame(Grade),
    DeadlyPoison(Grade),
    Disarm,
    Finesse(Grade),
    FlashOfSteel(Grade),
    GoodInstincts(Grade),
    SwiftStrike(Grade),
    Cleave(Grade),
    IronWave(Grade),
    SpotWeakness(Grade),
    TwinStrike(Grade),
    Bludgeon(Grade),
    Impervious(Grade),
    SeeingRed(Grade),
    Pummel(Grade),
    Uppercut(Grade),
    TrueGrit(Grade),
    Thunderclap(Grade),
    PommelStrike(Grade),
    ShrugItOff(Grade),
    RecklessCharge(Grade),
    Entrench(Grade),
    Bloodletting(Grade),
    Hemokinesis(Grade),
    BodySlam(Grade),
    Anger(Grade),
    Carnage(Grade),
    Clash(Grade),
    WildStrike(Grade),
    HeavyBlade(Grade),
    SwordBoomerang(Grade),
    // Power cards
    Barricade(Grade),
    DemonForm(Grade),
    FeelNoPain(Grade),
    DarkEmbrace(Grade),
    Juggernaut(Grade),
    Rupture(Grade),
    Berserk(Grade),
    Brutality(Grade),
    Combust(Grade),
    Evolve(Grade),
    FireBreathing(Grade),
    Feed(Grade),
    FiendFire(Grade),
    Flex(Grade),
    PerfectedStrike(Grade),
    PowerThrough(Grade),
    BurningPact(Grade),
    Warcry(Grade),
    Armaments(Grade),
    GhostlyArmor(Grade),
    SearingBlow(u32),
    SecondWind(Grade),
    Sentinel(Grade),
    AllOutAttack(Grade),
    AllForOne(Grade),
    Reaper(Grade),
    Whirlwind(Grade),
    Immolate(Grade),
    Intimidate(Grade),
    Shockwave(Grade),
    // Skill (exhausts on base)
    LimitBreak(Grade),
    Dazed,
    Slimed,
    Injury,
    Clumsy,
    Decay,
    Regret,
    Wound,
    Burn,
    Doubt,
    Shame,
    Parasite,
    CurseOfTheBell,
    AscendersBane,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Grade { Base, Plus }

pub struct GradeValues<T> { pub base: T, pub plus: T }

impl<T: Copy> GradeValues<T> {
    pub fn get(self, grade: Grade) -> T {
        match grade { Grade::Base => self.base, Grade::Plus => self.plus }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OnExhaustHook {
    GainEnergy(i32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CardType {
    Attack,
    Skill,
    Power,
    Curse,
    Status,
}

#[derive(Debug, Clone, Copy)]
pub enum EndOfTurnHook {
    BlockableDamage(i32),
    DirectHpLoss(i32),
    ApplyPlayerStatus { effect: StatusEffect, amount: i32 },
}

#[derive(Debug, Clone, Copy)]
pub enum CardDescription {
    Static(&'static str),
    WithDamage { template: &'static str, base: i32 },
}

#[derive(Debug, Clone, Copy)]
pub struct CardDef {
    pub name: &'static str,
    pub description: CardDescription,
    pub energy_cost: Energy,
    pub card_type: CardType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CardCost {
    Fixed(Energy),
    X,
}

impl CardCost {
    pub fn is_affordable(self, available: Energy) -> bool {
        match self {
            CardCost::Fixed(cost) => available >= cost,
            CardCost::X => true,
        }
    }

    pub fn display(self) -> String {
        match self {
            CardCost::Fixed(e) => e.0.to_string(),
            CardCost::X => "X".to_string(),
        }
    }
}

impl Card {
    pub fn def(&self) -> CardDef {
        match self {
            Card::Strike(g)       => strike::def(*g),
            Card::Defend(g)       => defend::def(*g),
            Card::Bash(g)         => bash::def(*g),
            Card::Clothesline(g)  => clothesline::def(*g),
            Card::Inflame(g)      => inflame::def(*g),
            Card::DeadlyPoison(g)  => deadly_poison::def(*g),
            Card::Disarm           => disarm::def(),
            Card::Finesse(g)       => finesse::def(*g),
            Card::FlashOfSteel(g)  => flash_of_steel::def(*g),
            Card::GoodInstincts(g) => good_instincts::def(*g),
            Card::SwiftStrike(g)   => swift_strike::def(*g),
            Card::Cleave(g)       => cleave::def(*g),
            Card::IronWave(g)     => iron_wave::def(*g),
            Card::SpotWeakness(g) => spot_weakness::def(*g),
            Card::TwinStrike(g)   => twin_strike::def(*g),
            Card::Bludgeon(g)     => bludgeon::def(*g),
            Card::Impervious(g)   => impervious::def(*g),
            Card::SeeingRed(g)    => seeing_red::def(*g),
            Card::Pummel(g)       => pummel::def(*g),
            Card::Uppercut(g)     => uppercut::def(*g),
            Card::TrueGrit(g)     => true_grit::def(*g),
            Card::Thunderclap(g)  => thunderclap::def(*g),
            Card::PommelStrike(g) => pommel_strike::def(*g),
            Card::ShrugItOff(g)   => shrug_it_off::def(*g),
            Card::RecklessCharge(g) => reckless_charge::def(*g),
            Card::Entrench(g)     => entrench::def(*g),
            Card::Bloodletting(g) => bloodletting::def(*g),
            Card::Hemokinesis(g)  => hemokinesis::def(*g),
            Card::BodySlam(g)     => body_slam::def(*g),
            Card::Anger(g)        => anger::def(*g),
            Card::Carnage(g)      => carnage::def(*g),
            Card::Clash(g)        => clash::def(*g),
            Card::WildStrike(g)   => wild_strike::def(*g),
            Card::HeavyBlade(g)   => heavy_blade::def(*g),
            Card::SwordBoomerang(g) => sword_boomerang::def(*g),
            Card::Barricade(g)    => barricade::def(*g),
            Card::DemonForm(g)    => demon_form::def(*g),
            Card::FeelNoPain(g)   => feel_no_pain::def(*g),
            Card::DarkEmbrace(g)  => dark_embrace::def(*g),
            Card::Juggernaut(g)   => juggernaut::def(*g),
            Card::Rupture(g)      => rupture::def(*g),
            Card::Berserk(g)      => berserk::def(*g),
            Card::Combust(g)      => combust::def(*g),
            Card::Evolve(g)       => evolve::def(*g),
            Card::FireBreathing(g) => fire_breathing::def(*g),
            Card::Feed(g)           => feed::def(*g),
            Card::FiendFire(g)      => fiend_fire::def(*g),
            Card::Flex(g)           => flex::def(*g),
            Card::PerfectedStrike(g) => perfected_strike::def(*g),
            Card::PowerThrough(g)    => power_through::def(*g),
            Card::BurningPact(g)     => burning_pact::def(*g),
            Card::Warcry(g)          => warcry::def(*g),
            Card::Armaments(g)       => armaments::def(*g),
            Card::GhostlyArmor(g)   => ghostly_armor::def(*g),
            Card::SearingBlow(n)     => searing_blow::def(*n),
            Card::SecondWind(g)      => second_wind::def(*g),
            Card::Sentinel(g)        => sentinel::def(*g),
            Card::AllOutAttack(g)    => all_out_attack::def(*g),
            Card::AllForOne(g)       => all_for_one::def(*g),
            Card::Reaper(g)          => reaper::def(*g),
            Card::Whirlwind(g)       => whirlwind::def(*g),
            Card::Immolate(g)     => immolate::def(*g),
            Card::Intimidate(g)   => intimidate::def(*g),
            Card::Shockwave(g)    => shockwave::def(*g),
            Card::Brutality(g)    => brutality::def(*g),
            Card::LimitBreak(g)   => limit_break::def(*g),
            Card::Dazed           => dazed::def(),
            Card::Slimed          => slimed::def(),
            Card::Injury          => injury::def(),
            Card::Clumsy          => clumsy::def(),
            Card::Decay           => decay::def(),
            Card::Regret          => regret::def(),
            Card::Wound           => wound::def(),
            Card::Burn            => burn::def(),
            Card::Doubt           => doubt::def(),
            Card::Shame           => shame::def(),
            Card::Parasite        => parasite::def(),
            Card::CurseOfTheBell  => curse_of_the_bell::def(),
            Card::AscendersBane   => ascenders_bane::def(),
        }
    }

    pub fn is_playable(&self) -> bool {
        !matches!(self, Card::Dazed | Card::Injury | Card::Clumsy | Card::Decay | Card::Regret |
            Card::Wound | Card::Burn | Card::Doubt | Card::Shame |
            Card::Parasite | Card::CurseOfTheBell | Card::AscendersBane)
    }

    pub fn is_ethereal(&self) -> bool {
        matches!(self, Card::Dazed | Card::Clumsy | Card::AscendersBane | Card::Carnage(_) | Card::GhostlyArmor(_))
    }

    pub fn is_innate(&self) -> bool {
        matches!(self, Card::Brutality(Grade::Plus))
    }

    pub fn end_of_turn_hook(&self, hand_size: i32) -> Option<EndOfTurnHook> {
        match self {
            Card::Decay  => Some(decay::end_of_turn_hook()),
            Card::Burn   => Some(burn::end_of_turn_hook()),
            Card::Regret => Some(regret::end_of_turn_hook(hand_size)),
            Card::Doubt  => Some(doubt::end_of_turn_hook()),
            Card::Shame  => Some(shame::end_of_turn_hook()),
            _ => None,
        }
    }

    pub fn on_exhaust_hook(&self) -> Option<OnExhaustHook> {
        match self {
            Card::Sentinel(g) => Some(OnExhaustHook::GainEnergy(GradeValues { base: 2, plus: 3 }.get(*g))),
            _ => None,
        }
    }

    pub fn exhausts(&self) -> bool {
        matches!(self, Card::Disarm | Card::Impervious(_) | Card::SeeingRed(_) | Card::Pummel(_) | Card::Carnage(_) | Card::LimitBreak(Grade::Base) | Card::Intimidate(_) | Card::Shockwave(_) | Card::FiendFire(_) | Card::Reaper(_) | Card::Feed(_) | Card::Warcry(_) | Card::Slimed)
    }

    pub fn grade(&self) -> Option<Grade> {
        match self {
            Card::Strike(g) | Card::Defend(g) | Card::Bash(g) | Card::Clothesline(g) |
            Card::Inflame(g) | Card::DeadlyPoison(g) | Card::Cleave(g) | Card::IronWave(g) |
            Card::SpotWeakness(g) | Card::TwinStrike(g) | Card::Bludgeon(g) | Card::Impervious(g) |
            Card::SeeingRed(g) | Card::Pummel(g) | Card::Uppercut(g) | Card::TrueGrit(g) |
            Card::Thunderclap(g) | Card::PommelStrike(g) | Card::ShrugItOff(g) |
            Card::RecklessCharge(g) | Card::Entrench(g) | Card::Bloodletting(g) |
            Card::Hemokinesis(g) | Card::BodySlam(g) | Card::Anger(g) |
            Card::Carnage(g) | Card::Clash(g) | Card::WildStrike(g) |
            Card::HeavyBlade(g) | Card::SwordBoomerang(g) |
            Card::Barricade(g) | Card::DemonForm(g) |
            Card::FeelNoPain(g) | Card::DarkEmbrace(g) |
            Card::Juggernaut(g) | Card::Rupture(g) |
            Card::Berserk(g) | Card::Brutality(g) | Card::Combust(g)
            | Card::Evolve(g) | Card::FireBreathing(g) | Card::Feed(g) | Card::FiendFire(g) | Card::Flex(g) | Card::PerfectedStrike(g) | Card::PowerThrough(g) | Card::BurningPact(g) | Card::Warcry(g) | Card::Armaments(g) | Card::GhostlyArmor(g) | Card::SecondWind(g) | Card::Sentinel(g) | Card::AllOutAttack(g) | Card::AllForOne(g) | Card::Reaper(g) | Card::Whirlwind(g)
            | Card::Immolate(g) | Card::Intimidate(g) | Card::Shockwave(g) | Card::LimitBreak(g)
            | Card::Finesse(g) | Card::FlashOfSteel(g) | Card::GoodInstincts(g) | Card::SwiftStrike(g) => Some(*g),
            Card::SearingBlow(_) |
            Card::Disarm | Card::Dazed | Card::Slimed | Card::Injury | Card::Clumsy | Card::Decay | Card::Regret |
            Card::Wound | Card::Burn | Card::Doubt | Card::Shame |
            Card::Parasite | Card::CurseOfTheBell | Card::AscendersBane => None,
        }
    }

    fn with_grade(&self, g: Grade) -> Card {
        match self {
            Card::Strike(_)       => Card::Strike(g),
            Card::Defend(_)       => Card::Defend(g),
            Card::Bash(_)         => Card::Bash(g),
            Card::Clothesline(_)  => Card::Clothesline(g),
            Card::Inflame(_)      => Card::Inflame(g),
            Card::DeadlyPoison(_) => Card::DeadlyPoison(g),
            Card::Cleave(_)       => Card::Cleave(g),
            Card::IronWave(_)       => Card::IronWave(g),
            Card::SpotWeakness(_)   => Card::SpotWeakness(g),
            Card::TwinStrike(_)     => Card::TwinStrike(g),
            Card::Bludgeon(_)       => Card::Bludgeon(g),
            Card::Impervious(_)     => Card::Impervious(g),
            Card::SeeingRed(_)      => Card::SeeingRed(g),
            Card::Pummel(_)         => Card::Pummel(g),
            Card::Uppercut(_)       => Card::Uppercut(g),
            Card::TrueGrit(_)       => Card::TrueGrit(g),
            Card::Thunderclap(_)    => Card::Thunderclap(g),
            Card::PommelStrike(_)   => Card::PommelStrike(g),
            Card::ShrugItOff(_)     => Card::ShrugItOff(g),
            Card::RecklessCharge(_) => Card::RecklessCharge(g),
            Card::Entrench(_)       => Card::Entrench(g),
            Card::Bloodletting(_) => Card::Bloodletting(g),
            Card::Hemokinesis(_)  => Card::Hemokinesis(g),
            Card::BodySlam(_)     => Card::BodySlam(g),
            Card::Anger(_)        => Card::Anger(g),
            Card::Carnage(_)      => Card::Carnage(g),
            Card::Clash(_)        => Card::Clash(g),
            Card::WildStrike(_)   => Card::WildStrike(g),
            Card::HeavyBlade(_)   => Card::HeavyBlade(g),
            Card::SwordBoomerang(_) => Card::SwordBoomerang(g),
            Card::Barricade(_)    => Card::Barricade(g),
            Card::DemonForm(_)    => Card::DemonForm(g),
            Card::FeelNoPain(_)   => Card::FeelNoPain(g),
            Card::DarkEmbrace(_)  => Card::DarkEmbrace(g),
            Card::Juggernaut(_)   => Card::Juggernaut(g),
            Card::Rupture(_)      => Card::Rupture(g),
            Card::Berserk(_)      => Card::Berserk(g),
            Card::Combust(_)      => Card::Combust(g),
            Card::Evolve(_)       => Card::Evolve(g),
            Card::FireBreathing(_) => Card::FireBreathing(g),
            Card::Feed(_)           => Card::Feed(g),
            Card::FiendFire(_)      => Card::FiendFire(g),
            Card::Flex(_)           => Card::Flex(g),
            Card::PerfectedStrike(_) => Card::PerfectedStrike(g),
            Card::PowerThrough(_)    => Card::PowerThrough(g),
            Card::BurningPact(_)     => Card::BurningPact(g),
            Card::Warcry(_)          => Card::Warcry(g),
            Card::Armaments(_)       => Card::Armaments(g),
            Card::GhostlyArmor(_)   => Card::GhostlyArmor(g),
            Card::SecondWind(_)      => Card::SecondWind(g),
            Card::Sentinel(_)        => Card::Sentinel(g),
            Card::AllOutAttack(_)    => Card::AllOutAttack(g),
            Card::AllForOne(_)       => Card::AllForOne(g),
            Card::Reaper(_)          => Card::Reaper(g),
            Card::Whirlwind(_)       => Card::Whirlwind(g),
            Card::Immolate(_)     => Card::Immolate(g),
            Card::Intimidate(_)   => Card::Intimidate(g),
            Card::Shockwave(_)    => Card::Shockwave(g),
            Card::Brutality(_)    => Card::Brutality(g),
            Card::LimitBreak(_)    => Card::LimitBreak(g),
            Card::Finesse(_)       => Card::Finesse(g),
            Card::FlashOfSteel(_)  => Card::FlashOfSteel(g),
            Card::GoodInstincts(_) => Card::GoodInstincts(g),
            Card::SwiftStrike(_)   => Card::SwiftStrike(g),
            Card::SearingBlow(_) => unreachable!(),
            Card::Disarm | Card::Dazed | Card::Slimed | Card::Injury | Card::Clumsy | Card::Decay | Card::Regret |
            Card::Wound | Card::Burn | Card::Doubt | Card::Shame |
            Card::Parasite | Card::CurseOfTheBell | Card::AscendersBane => unreachable!(),
        }
    }

    pub fn upgrade(&self) -> Option<Card> {
        if let Card::SearingBlow(n) = self {
            return Some(Card::SearingBlow(n + 1));
        }
        match self.grade()? {
            Grade::Base => Some(self.with_grade(Grade::Plus)),
            Grade::Plus => None,
        }
    }

    pub fn card_type(&self) -> CardType { self.def().card_type }

    pub fn name(&self) -> &'static str { self.def().name }
    pub fn energy_cost(&self) -> Energy { self.def().energy_cost }

    pub fn card_cost(&self) -> CardCost {
        match self {
            Card::Whirlwind(_) => CardCost::X,
            _ => CardCost::Fixed(self.energy_cost()),
        }
    }

    pub fn description(&self) -> String {
        match self.def().description {
            CardDescription::Static(s) => s.to_string(),
            CardDescription::WithDamage { template, base } => {
                template.replace("{damage}", &base.to_string())
            }
        }
    }

    pub fn effective_description(&self, attacker: &StatusMap, defender: &StatusMap) -> String {
        match self.def().description {
            CardDescription::Static(s) => s.to_string(),
            CardDescription::WithDamage { template, base } => {
                let eff = resolve_damage(base, attacker, defender);
                let num = if eff != base { format!("*{eff}*") } else { eff.to_string() };
                template.replace("{damage}", &num)
            }
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            Card::Strike(g)       => strike::id(*g),
            Card::Defend(g)       => defend::id(*g),
            Card::Bash(g)         => bash::id(*g),
            Card::Clothesline(g)  => clothesline::id(*g),
            Card::Inflame(g)      => inflame::id(*g),
            Card::DeadlyPoison(g)  => deadly_poison::id(*g),
            Card::Disarm           => disarm::id(),
            Card::Finesse(g)       => finesse::id(*g),
            Card::FlashOfSteel(g)  => flash_of_steel::id(*g),
            Card::GoodInstincts(g) => good_instincts::id(*g),
            Card::SwiftStrike(g)   => swift_strike::id(*g),
            Card::Cleave(g)       => cleave::id(*g),
            Card::IronWave(g)     => iron_wave::id(*g),
            Card::SpotWeakness(g)   => spot_weakness::id(*g),
            Card::TwinStrike(g)     => twin_strike::id(*g),
            Card::Bludgeon(g)       => bludgeon::id(*g),
            Card::Impervious(g)     => impervious::id(*g),
            Card::SeeingRed(g)      => seeing_red::id(*g),
            Card::Pummel(g)         => pummel::id(*g),
            Card::Uppercut(g)       => uppercut::id(*g),
            Card::TrueGrit(g)       => true_grit::id(*g),
            Card::Thunderclap(g)    => thunderclap::id(*g),
            Card::PommelStrike(g)   => pommel_strike::id(*g),
            Card::ShrugItOff(g)     => shrug_it_off::id(*g),
            Card::RecklessCharge(g) => reckless_charge::id(*g),
            Card::Entrench(g)       => entrench::id(*g),
            Card::Bloodletting(g) => bloodletting::id(*g),
            Card::Hemokinesis(g)  => hemokinesis::id(*g),
            Card::BodySlam(g)     => body_slam::id(*g),
            Card::Anger(g)        => anger::id(*g),
            Card::Carnage(g)      => carnage::id(*g),
            Card::Clash(g)        => clash::id(*g),
            Card::WildStrike(g)   => wild_strike::id(*g),
            Card::HeavyBlade(g)   => heavy_blade::id(*g),
            Card::SwordBoomerang(g) => sword_boomerang::id(*g),
            Card::Barricade(g)    => barricade::id(*g),
            Card::DemonForm(g)    => demon_form::id(*g),
            Card::FeelNoPain(g)   => feel_no_pain::id(*g),
            Card::DarkEmbrace(g)  => dark_embrace::id(*g),
            Card::Juggernaut(g)   => juggernaut::id(*g),
            Card::Rupture(g)      => rupture::id(*g),
            Card::Berserk(g)      => berserk::id(*g),
            Card::Combust(g)      => combust::id(*g),
            Card::Evolve(g)       => evolve::id(*g),
            Card::FireBreathing(g) => fire_breathing::id(*g),
            Card::Feed(g)           => feed::id(*g),
            Card::FiendFire(g)      => fiend_fire::id(*g),
            Card::Flex(g)           => flex::id(*g),
            Card::PerfectedStrike(g) => perfected_strike::id(*g),
            Card::PowerThrough(g)    => power_through::id(*g),
            Card::BurningPact(g)     => burning_pact::id(*g),
            Card::Warcry(g)          => warcry::id(*g),
            Card::Armaments(g)       => armaments::id(*g),
            Card::GhostlyArmor(g)   => ghostly_armor::id(*g),
            Card::SearingBlow(_)     => searing_blow::id(),
            Card::SecondWind(g)      => second_wind::id(*g),
            Card::Sentinel(g)        => sentinel::id(*g),
            Card::AllOutAttack(g)    => all_out_attack::id(*g),
            Card::AllForOne(g)       => all_for_one::id(*g),
            Card::Reaper(g)          => reaper::id(*g),
            Card::Whirlwind(g)       => whirlwind::id(*g),
            Card::Immolate(g)     => immolate::id(*g),
            Card::Intimidate(g)   => intimidate::id(*g),
            Card::Shockwave(g)    => shockwave::id(*g),
            Card::Brutality(g)    => brutality::id(*g),
            Card::LimitBreak(g)   => limit_break::id(*g),
            Card::Dazed           => dazed::id(),
            Card::Slimed          => slimed::id(),
            Card::Injury          => injury::id(),
            Card::Clumsy          => clumsy::id(),
            Card::Decay           => decay::id(),
            Card::Regret          => regret::id(),
            Card::Wound           => wound::id(),
            Card::Burn            => burn::id(),
            Card::Doubt           => doubt::id(),
            Card::Shame           => shame::id(),
            Card::Parasite        => parasite::id(),
            Card::CurseOfTheBell  => curse_of_the_bell::id(),
            Card::AscendersBane   => ascenders_bane::id(),
        }
    }

    pub fn from_id(s: &str) -> Option<Card> {
        use Grade::{Base, Plus};
        let all: &[Card] = &[
            Card::Strike(Base),       Card::Strike(Plus),
            Card::Defend(Base),       Card::Defend(Plus),
            Card::Bash(Base),         Card::Bash(Plus),
            Card::Clothesline(Base),  Card::Clothesline(Plus),
            Card::Inflame(Base),      Card::Inflame(Plus),
            Card::DeadlyPoison(Base), Card::DeadlyPoison(Plus),
            Card::Disarm,
            Card::Finesse(Base),       Card::Finesse(Plus),
            Card::FlashOfSteel(Base),  Card::FlashOfSteel(Plus),
            Card::GoodInstincts(Base), Card::GoodInstincts(Plus),
            Card::SwiftStrike(Base),   Card::SwiftStrike(Plus),
            Card::Cleave(Base),       Card::Cleave(Plus),
            Card::IronWave(Base),     Card::IronWave(Plus),
            Card::SpotWeakness(Base),   Card::SpotWeakness(Plus),
            Card::TwinStrike(Base),     Card::TwinStrike(Plus),
            Card::Bludgeon(Base),       Card::Bludgeon(Plus),
            Card::Impervious(Base),     Card::Impervious(Plus),
            Card::SeeingRed(Base),      Card::SeeingRed(Plus),
            Card::Pummel(Base),         Card::Pummel(Plus),
            Card::Uppercut(Base),       Card::Uppercut(Plus),
            Card::TrueGrit(Base),       Card::TrueGrit(Plus),
            Card::Thunderclap(Base),    Card::Thunderclap(Plus),
            Card::PommelStrike(Base),   Card::PommelStrike(Plus),
            Card::ShrugItOff(Base),     Card::ShrugItOff(Plus),
            Card::RecklessCharge(Base), Card::RecklessCharge(Plus),
            Card::Entrench(Base),       Card::Entrench(Plus),
            Card::Bloodletting(Base), Card::Bloodletting(Plus),
            Card::Hemokinesis(Base),  Card::Hemokinesis(Plus),
            Card::BodySlam(Base),     Card::BodySlam(Plus),
            Card::Anger(Base),        Card::Anger(Plus),
            Card::Carnage(Base),      Card::Carnage(Plus),
            Card::Clash(Base),        Card::Clash(Plus),
            Card::WildStrike(Base),   Card::WildStrike(Plus),
            Card::HeavyBlade(Base),   Card::HeavyBlade(Plus),
            Card::SwordBoomerang(Base), Card::SwordBoomerang(Plus),
            Card::Barricade(Base),    Card::Barricade(Plus),
            Card::DemonForm(Base),    Card::DemonForm(Plus),
            Card::FeelNoPain(Base),   Card::FeelNoPain(Plus),
            Card::DarkEmbrace(Base),  Card::DarkEmbrace(Plus),
            Card::Juggernaut(Base),   Card::Juggernaut(Plus),
            Card::Rupture(Base),      Card::Rupture(Plus),
            Card::Berserk(Base),      Card::Berserk(Plus),
            Card::Combust(Base),      Card::Combust(Plus),
            Card::Evolve(Base),       Card::Evolve(Plus),
            Card::FireBreathing(Base), Card::FireBreathing(Plus),
            Card::Feed(Base),            Card::Feed(Plus),
            Card::FiendFire(Base),       Card::FiendFire(Plus),
            Card::Flex(Base),            Card::Flex(Plus),
            Card::PerfectedStrike(Base), Card::PerfectedStrike(Plus),
            Card::PowerThrough(Base),    Card::PowerThrough(Plus),
            Card::BurningPact(Base),     Card::BurningPact(Plus),
            Card::Warcry(Base),          Card::Warcry(Plus),
            Card::Armaments(Base),       Card::Armaments(Plus),
            Card::GhostlyArmor(Base),   Card::GhostlyArmor(Plus),
            Card::SearingBlow(0),
            Card::SecondWind(Base),      Card::SecondWind(Plus),
            Card::Sentinel(Base),        Card::Sentinel(Plus),
            Card::AllOutAttack(Base),    Card::AllOutAttack(Plus),
            Card::AllForOne(Base),       Card::AllForOne(Plus),
            Card::Reaper(Base),          Card::Reaper(Plus),
            Card::Whirlwind(Base),       Card::Whirlwind(Plus),
            Card::Immolate(Base),     Card::Immolate(Plus),
            Card::Intimidate(Base),   Card::Intimidate(Plus),
            Card::Shockwave(Base),    Card::Shockwave(Plus),
            Card::Brutality(Base),    Card::Brutality(Plus),
            Card::LimitBreak(Base),   Card::LimitBreak(Plus),
            Card::Dazed,
            Card::Slimed,
            Card::Injury,
            Card::Clumsy,
            Card::Decay,
            Card::Regret,
            Card::Wound,
            Card::Burn,
            Card::Doubt,
            Card::Shame,
            Card::Parasite,
            Card::CurseOfTheBell,
            Card::AscendersBane,
        ];
        all.iter().find(|c| c.id() == s).cloned()
    }

    pub fn effective_damage(&self, attacker: &StatusMap, defender: &StatusMap) -> Option<i32> {
        match self.def().description {
            CardDescription::WithDamage { base, .. } => Some(resolve_damage(base, attacker, defender)),
            CardDescription::Static(_) => None,
        }
    }
}

pub fn apply(card: &Card, state: &mut crate::combat::CombatState, events: &mut Vec<crate::combat::Event>, target: usize, rng: &mut impl crate::rng::Rng, x_value: i32) {
    match card {
        Card::Strike(g)       => strike::apply(state, events, *g, target),
        Card::Defend(g)       => defend::apply(state, events, *g, target, rng),
        Card::Bash(g)         => bash::apply(state, events, *g, target),
        Card::Clothesline(g)  => clothesline::apply(state, events, *g, target),
        Card::Inflame(g)      => inflame::apply(state, events, *g, target),
        Card::DeadlyPoison(g)  => deadly_poison::apply(state, events, *g, target),
        Card::Disarm           => disarm::apply(state, events, target),
        Card::Finesse(g)       => finesse::apply(state, events, *g, rng),
        Card::FlashOfSteel(g)  => flash_of_steel::apply(state, events, *g, target, rng),
        Card::GoodInstincts(g) => good_instincts::apply(state, events, *g, rng),
        Card::SwiftStrike(g)   => swift_strike::apply(state, events, *g, target),
        Card::Cleave(g)        => cleave::apply(state, events, *g),
        Card::IronWave(g)     => iron_wave::apply(state, events, *g, target, rng),
        Card::SpotWeakness(g)   => spot_weakness::apply(state, events, *g, target),
        Card::TwinStrike(g)     => twin_strike::apply(state, events, *g, target),
        Card::Bludgeon(g)       => bludgeon::apply(state, events, *g, target),
        Card::Impervious(g)     => impervious::apply(state, events, *g, target, rng),
        Card::SeeingRed(g)      => seeing_red::apply(state, events, *g),
        Card::Pummel(g)         => pummel::apply(state, events, *g, target),
        Card::Uppercut(g)       => uppercut::apply(state, events, *g, target),
        Card::TrueGrit(g)       => true_grit::apply(state, events, *g, rng),
        Card::Thunderclap(g)    => thunderclap::apply(state, events, *g),
        Card::PommelStrike(g)   => pommel_strike::apply(state, events, *g, target, rng),
        Card::ShrugItOff(g)     => shrug_it_off::apply(state, events, *g, rng),
        Card::RecklessCharge(g) => reckless_charge::apply(state, events, *g, target, rng),
        Card::Entrench(g)       => entrench::apply(state, events, *g, rng),
        Card::Bloodletting(g) => bloodletting::apply(state, events, *g),
        Card::Hemokinesis(g)  => hemokinesis::apply(state, events, *g, target),
        Card::BodySlam(_)     => body_slam::apply(state, events, target),
        Card::Anger(g)        => anger::apply(state, events, *g, target),
        Card::Carnage(g)      => carnage::apply(state, events, *g, target),
        Card::Clash(g)        => clash::apply(state, events, *g, target),
        Card::WildStrike(g)   => wild_strike::apply(state, events, *g, target, rng),
        Card::HeavyBlade(g)   => heavy_blade::apply(state, events, *g, target),
        Card::SwordBoomerang(g) => sword_boomerang::apply(state, events, *g, rng),
        Card::Barricade(g)    => barricade::apply(state, events, *g, target),
        Card::DemonForm(g)    => demon_form::apply(state, events, *g, target),
        Card::FeelNoPain(g)   => feel_no_pain::apply(state, events, *g, target),
        Card::DarkEmbrace(g)  => dark_embrace::apply(state, events, *g, target),
        Card::Juggernaut(g)   => juggernaut::apply(state, events, *g, target),
        Card::Rupture(g)      => rupture::apply(state, events, *g, target),
        Card::Berserk(g)      => berserk::apply(state, events, *g, target),
        Card::Combust(g)      => combust::apply(state, events, *g, target),
        Card::Evolve(g)       => evolve::apply(state, events, *g, target),
        Card::FireBreathing(g) => fire_breathing::apply(state, events, *g, target),
        Card::Feed(g)           => feed::apply(state, events, *g, target),
        Card::FiendFire(g)      => fiend_fire::apply(state, events, *g, target, rng),
        Card::Flex(g)           => flex::apply(state, events, *g, target),
        Card::PerfectedStrike(g) => perfected_strike::apply(state, events, *g, target),
        Card::PowerThrough(g)    => power_through::apply(state, events, *g, target, rng),
        Card::BurningPact(g)     => burning_pact::apply(state, *g),
        Card::Warcry(g)          => warcry::apply(state, events, *g, rng),
        Card::Armaments(g)       => armaments::apply(state, events, *g, rng),
        Card::GhostlyArmor(g)   => ghostly_armor::apply(state, events, *g, rng),
        Card::SearingBlow(n)     => searing_blow::apply(state, events, *n, target),
        Card::SecondWind(g)      => second_wind::apply(state, events, *g, rng),
        Card::Sentinel(g)        => sentinel::apply(state, events, *g, rng),
        Card::AllOutAttack(g)    => all_out_attack::apply(state, events, *g, rng),
        Card::AllForOne(g)       => all_for_one::apply(state, events, *g, target),
        Card::Reaper(g)          => reaper::apply(state, events, *g, target),
        Card::Whirlwind(g)       => whirlwind::apply(state, events, *g, x_value),
        Card::Immolate(g)     => immolate::apply(state, events, *g, target),
        Card::Intimidate(g)   => intimidate::apply(state, events, *g, target),
        Card::Shockwave(g)    => shockwave::apply(state, events, *g, target),
        Card::Brutality(g)    => brutality::apply(state, events, *g, target),
        Card::LimitBreak(g)   => limit_break::apply(state, events, *g, target),
        Card::Slimed => {} // playable, but no effect — just exhausts
        Card::Dazed | Card::Injury | Card::Clumsy | Card::Decay | Card::Regret |
        Card::Wound | Card::Burn | Card::Doubt | Card::Shame |
        Card::Parasite | Card::CurseOfTheBell | Card::AscendersBane => {} // unplayable
    }
}

pub fn reward_pool() -> Vec<Card> {
    use Grade::Base;
    vec![
        Card::Bash(Base), Card::Clothesline(Base), Card::Inflame(Base), Card::DeadlyPoison(Base),
        Card::Cleave(Base), Card::IronWave(Base), Card::TwinStrike(Base), Card::Bludgeon(Base),
        Card::Impervious(Base), Card::SeeingRed(Base), Card::Pummel(Base), Card::Uppercut(Base),
        Card::SpotWeakness(Base), Card::Thunderclap(Base),
        Card::PommelStrike(Base), Card::ShrugItOff(Base),
        Card::RecklessCharge(Base), Card::TrueGrit(Base), Card::Bloodletting(Base), Card::Hemokinesis(Base),
        Card::BodySlam(Base), Card::Anger(Base), Card::Entrench(Base),
        Card::Carnage(Base), Card::Clash(Base), Card::WildStrike(Base),
        Card::HeavyBlade(Base), Card::SwordBoomerang(Base),
        Card::Barricade(Base), Card::DemonForm(Base),
        Card::FeelNoPain(Base), Card::DarkEmbrace(Base),
        Card::Juggernaut(Base), Card::Rupture(Base),
        Card::Berserk(Base), Card::Brutality(Base), Card::Combust(Base),
        Card::Evolve(Base), Card::FireBreathing(Base), Card::Flex(Base),
        Card::Feed(Base), Card::FiendFire(Base), Card::PerfectedStrike(Base), Card::PowerThrough(Base), Card::BurningPact(Base), Card::Warcry(Base), Card::Armaments(Base), Card::GhostlyArmor(Base), Card::SearingBlow(0), Card::SecondWind(Base), Card::Sentinel(Base), Card::AllOutAttack(Base), Card::AllForOne(Base), Card::Reaper(Base), Card::Whirlwind(Base), Card::Immolate(Base), Card::Intimidate(Base), Card::Shockwave(Base), Card::LimitBreak(Base),
    ]
}

pub fn starter_deck() -> Vec<Card> {
    use Grade::Base;
    let mut deck = Vec::new();
    for _ in 0..5 {
        deck.push(Card::Strike(Base));
    }
    for _ in 0..3 {
        deck.push(Card::Defend(Base));
    }
    deck.push(Card::Bash(Base));
    deck.push(Card::Inflame(Base));
    deck.push(Card::DeadlyPoison(Base));
    deck.push(Card::Disarm);
    deck
}


#[cfg(test)]
mod tests;

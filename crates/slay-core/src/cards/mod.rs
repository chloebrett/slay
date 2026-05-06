mod anger;
mod bash;
mod ascenders_bane;
mod carnage;
mod clash;
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
mod deadly_poison;
mod defend;
mod disarm;
mod entrench;
mod hemokinesis;
mod impervious;
mod inflame;
mod iron_wave;
mod pommel_strike;
mod pummel;
mod reckless_charge;
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
mod wild_strike;

use crate::status::{StatusEffect, StatusMap, resolve_damage};
use crate::types::Energy;

#[derive(Debug, Clone, PartialEq)]
pub enum Card {
    Strike(Grade),
    Defend(Grade),
    Bash(Grade),
    Clothesline(Grade),
    Inflame(Grade),
    DeadlyPoison(Grade),
    Disarm,
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
    Dazed,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Grade { Base, Plus }

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

impl Card {
    pub fn def(&self) -> CardDef {
        match self {
            Card::Strike(g)       => strike::def(*g),
            Card::Defend(g)       => defend::def(*g),
            Card::Bash(g)         => bash::def(*g),
            Card::Clothesline(g)  => clothesline::def(*g),
            Card::Inflame(g)      => inflame::def(*g),
            Card::DeadlyPoison(g) => deadly_poison::def(*g),
            Card::Disarm          => disarm::def(),
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
            Card::Dazed           => dazed::def(),
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
        matches!(self, Card::Dazed | Card::Clumsy | Card::AscendersBane | Card::Carnage(_))
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

    pub fn exhausts(&self) -> bool {
        matches!(self, Card::Disarm | Card::Impervious(_) | Card::SeeingRed(_) | Card::Pummel(_) | Card::Carnage(_))
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
            Card::HeavyBlade(g) | Card::SwordBoomerang(g) => Some(*g),
            Card::Disarm | Card::Dazed | Card::Injury | Card::Clumsy | Card::Decay | Card::Regret |
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
            Card::Disarm | Card::Dazed | Card::Injury | Card::Clumsy | Card::Decay | Card::Regret |
            Card::Wound | Card::Burn | Card::Doubt | Card::Shame |
            Card::Parasite | Card::CurseOfTheBell | Card::AscendersBane => unreachable!(),
        }
    }

    pub fn upgrade(&self) -> Option<Card> {
        match self.grade()? {
            Grade::Base => Some(self.with_grade(Grade::Plus)),
            Grade::Plus => None,
        }
    }

    pub fn card_type(&self) -> CardType { self.def().card_type }

    pub fn name(&self) -> &'static str { self.def().name }
    pub fn energy_cost(&self) -> Energy { self.def().energy_cost }

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
            Card::DeadlyPoison(g) => deadly_poison::id(*g),
            Card::Disarm          => disarm::id(),
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
            Card::Dazed           => dazed::id(),
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
            Card::Dazed,
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

pub fn apply(card: &Card, state: &mut crate::combat::CombatState, events: &mut Vec<crate::combat::Event>, target: usize, rng: &mut impl crate::rng::Rng) {
    match card {
        Card::Strike(g)       => strike::apply(state, events, *g, target),
        Card::Defend(g)       => defend::apply(state, events, *g, target),
        Card::Bash(g)         => bash::apply(state, events, *g, target),
        Card::Clothesline(g)  => clothesline::apply(state, events, *g, target),
        Card::Inflame(g)      => inflame::apply(state, events, *g, target),
        Card::DeadlyPoison(g) => deadly_poison::apply(state, events, *g, target),
        Card::Disarm          => disarm::apply(state, events, target),
        Card::Cleave(g)       => cleave::apply(state, events, *g),
        Card::IronWave(g)     => iron_wave::apply(state, events, *g, target),
        Card::SpotWeakness(g)   => spot_weakness::apply(state, events, *g, target),
        Card::TwinStrike(g)     => twin_strike::apply(state, events, *g, target),
        Card::Bludgeon(g)       => bludgeon::apply(state, events, *g, target),
        Card::Impervious(g)     => impervious::apply(state, events, *g, target),
        Card::SeeingRed(g)      => seeing_red::apply(state, events, *g),
        Card::Pummel(g)         => pummel::apply(state, events, *g, target),
        Card::Uppercut(g)       => uppercut::apply(state, events, *g, target),
        Card::TrueGrit(g)       => true_grit::apply(state, events, *g, rng),
        Card::Thunderclap(g)    => thunderclap::apply(state, events, *g),
        Card::PommelStrike(g)   => pommel_strike::apply(state, events, *g, target, rng),
        Card::ShrugItOff(g)     => shrug_it_off::apply(state, events, *g, rng),
        Card::RecklessCharge(g) => reckless_charge::apply(state, events, *g, target, rng),
        Card::Entrench(g)       => entrench::apply(state, events, *g),
        Card::Bloodletting(g) => bloodletting::apply(state, events, *g),
        Card::Hemokinesis(g)  => hemokinesis::apply(state, events, *g, target),
        Card::BodySlam(_)     => body_slam::apply(state, events, target),
        Card::Anger(g)        => anger::apply(state, events, *g, target),
        Card::Carnage(g)      => carnage::apply(state, events, *g, target),
        Card::Clash(g)        => clash::apply(state, events, *g, target),
        Card::WildStrike(g)   => wild_strike::apply(state, events, *g, target, rng),
        Card::HeavyBlade(g)   => heavy_blade::apply(state, events, *g, target),
        Card::SwordBoomerang(g) => sword_boomerang::apply(state, events, *g, rng),
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

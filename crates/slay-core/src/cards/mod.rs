mod anger;
mod bash;
mod blood_wall;
mod body_slam;
mod bloodletting;
mod breakthrough;
mod bludgeon;
mod cleave;
mod clothesline;
mod dazed;
mod deadly_poison;
mod defend;
mod disarm;
mod hemokinesis;
mod impervious;
mod inflame;
mod iron_wave;
mod mangle;
mod not_yet;
mod pommel_strike;
mod shrug_it_off;
mod strike;
mod taunt;
mod thunderclap;
mod tremble;
mod twin_strike;
mod uppercut;

use crate::status::{StatusMap, resolve_damage};
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
    Tremble(Grade),
    TwinStrike(Grade),
    Bludgeon(Grade),
    Impervious(Grade),
    NotYet(Grade),
    Mangle(Grade),
    Uppercut(Grade),
    Taunt(Grade),
    Thunderclap(Grade),
    PommelStrike(Grade),
    ShrugItOff(Grade),
    Breakthrough(Grade),
    BloodWall(Grade),
    Bloodletting(Grade),
    Hemokinesis(Grade),
    BodySlam(Grade),
    Anger(Grade),
    Dazed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Grade { Base, Plus }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CardType {
    Attack,
    Skill,
    Power,
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
            Card::Tremble(g)      => tremble::def(*g),
            Card::TwinStrike(g)   => twin_strike::def(*g),
            Card::Bludgeon(g)     => bludgeon::def(*g),
            Card::Impervious(g)   => impervious::def(*g),
            Card::NotYet(g)       => not_yet::def(*g),
            Card::Mangle(g)       => mangle::def(*g),
            Card::Uppercut(g)     => uppercut::def(*g),
            Card::Taunt(g)        => taunt::def(*g),
            Card::Thunderclap(g)  => thunderclap::def(*g),
            Card::PommelStrike(g) => pommel_strike::def(*g),
            Card::ShrugItOff(g)   => shrug_it_off::def(*g),
            Card::Breakthrough(g) => breakthrough::def(*g),
            Card::BloodWall(g)    => blood_wall::def(*g),
            Card::Bloodletting(g) => bloodletting::def(*g),
            Card::Hemokinesis(g)  => hemokinesis::def(*g),
            Card::BodySlam(g)     => body_slam::def(*g),
            Card::Anger(g)        => anger::def(*g),
            Card::Dazed           => dazed::def(),
        }
    }

    pub fn is_playable(&self) -> bool {
        !matches!(self, Card::Dazed)
    }

    pub fn exhausts(&self) -> bool {
        matches!(self, Card::Disarm | Card::Impervious(_) | Card::Dazed)
    }

    pub fn grade(&self) -> Option<Grade> {
        match self {
            Card::Strike(g) | Card::Defend(g) | Card::Bash(g) | Card::Clothesline(g) |
            Card::Inflame(g) | Card::DeadlyPoison(g) | Card::Cleave(g) | Card::IronWave(g) |
            Card::Tremble(g) | Card::TwinStrike(g) | Card::Bludgeon(g) | Card::Impervious(g) |
            Card::NotYet(g) | Card::Mangle(g) | Card::Uppercut(g) | Card::Taunt(g) |
            Card::Thunderclap(g) | Card::PommelStrike(g) | Card::ShrugItOff(g) |
            Card::Breakthrough(g) | Card::BloodWall(g) | Card::Bloodletting(g) |
            Card::Hemokinesis(g) | Card::BodySlam(g) | Card::Anger(g) => Some(*g),
            Card::Disarm | Card::Dazed => None,
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
            Card::IronWave(_)     => Card::IronWave(g),
            Card::Tremble(_)      => Card::Tremble(g),
            Card::TwinStrike(_)   => Card::TwinStrike(g),
            Card::Bludgeon(_)     => Card::Bludgeon(g),
            Card::Impervious(_)   => Card::Impervious(g),
            Card::NotYet(_)       => Card::NotYet(g),
            Card::Mangle(_)       => Card::Mangle(g),
            Card::Uppercut(_)     => Card::Uppercut(g),
            Card::Taunt(_)        => Card::Taunt(g),
            Card::Thunderclap(_)  => Card::Thunderclap(g),
            Card::PommelStrike(_) => Card::PommelStrike(g),
            Card::ShrugItOff(_)   => Card::ShrugItOff(g),
            Card::Breakthrough(_) => Card::Breakthrough(g),
            Card::BloodWall(_)    => Card::BloodWall(g),
            Card::Bloodletting(_) => Card::Bloodletting(g),
            Card::Hemokinesis(_)  => Card::Hemokinesis(g),
            Card::BodySlam(_)     => Card::BodySlam(g),
            Card::Anger(_)        => Card::Anger(g),
            Card::Disarm | Card::Dazed => unreachable!(),
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
            Card::Tremble(g)      => tremble::id(*g),
            Card::TwinStrike(g)   => twin_strike::id(*g),
            Card::Bludgeon(g)     => bludgeon::id(*g),
            Card::Impervious(g)   => impervious::id(*g),
            Card::NotYet(g)       => not_yet::id(*g),
            Card::Mangle(g)       => mangle::id(*g),
            Card::Uppercut(g)     => uppercut::id(*g),
            Card::Taunt(g)        => taunt::id(*g),
            Card::Thunderclap(g)  => thunderclap::id(*g),
            Card::PommelStrike(g) => pommel_strike::id(*g),
            Card::ShrugItOff(g)   => shrug_it_off::id(*g),
            Card::Breakthrough(g) => breakthrough::id(*g),
            Card::BloodWall(g)    => blood_wall::id(*g),
            Card::Bloodletting(g) => bloodletting::id(*g),
            Card::Hemokinesis(g)  => hemokinesis::id(*g),
            Card::BodySlam(g)     => body_slam::id(*g),
            Card::Anger(g)        => anger::id(*g),
            Card::Dazed           => dazed::id(),
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
            Card::Tremble(Base),      Card::Tremble(Plus),
            Card::TwinStrike(Base),   Card::TwinStrike(Plus),
            Card::Bludgeon(Base),     Card::Bludgeon(Plus),
            Card::Impervious(Base),   Card::Impervious(Plus),
            Card::NotYet(Base),       Card::NotYet(Plus),
            Card::Mangle(Base),       Card::Mangle(Plus),
            Card::Uppercut(Base),     Card::Uppercut(Plus),
            Card::Taunt(Base),        Card::Taunt(Plus),
            Card::Thunderclap(Base),  Card::Thunderclap(Plus),
            Card::PommelStrike(Base), Card::PommelStrike(Plus),
            Card::ShrugItOff(Base),   Card::ShrugItOff(Plus),
            Card::Breakthrough(Base), Card::Breakthrough(Plus),
            Card::BloodWall(Base),    Card::BloodWall(Plus),
            Card::Bloodletting(Base), Card::Bloodletting(Plus),
            Card::Hemokinesis(Base),  Card::Hemokinesis(Plus),
            Card::BodySlam(Base),     Card::BodySlam(Plus),
            Card::Anger(Base),        Card::Anger(Plus),
            Card::Dazed,
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
        Card::Tremble(g)      => tremble::apply(state, events, *g, target),
        Card::TwinStrike(g)   => twin_strike::apply(state, events, *g, target),
        Card::Bludgeon(g)     => bludgeon::apply(state, events, *g, target),
        Card::Impervious(g)   => impervious::apply(state, events, *g, target),
        Card::NotYet(g)       => not_yet::apply(state, events, *g, target),
        Card::Mangle(g)       => mangle::apply(state, events, *g, target),
        Card::Uppercut(g)     => uppercut::apply(state, events, *g, target),
        Card::Taunt(g)        => taunt::apply(state, events, *g, target),
        Card::Thunderclap(g)  => thunderclap::apply(state, events, *g),
        Card::PommelStrike(g) => pommel_strike::apply(state, events, *g, target, rng),
        Card::ShrugItOff(g)   => shrug_it_off::apply(state, events, *g, rng),
        Card::Breakthrough(g) => breakthrough::apply(state, events, *g),
        Card::BloodWall(g)    => blood_wall::apply(state, events, *g),
        Card::Bloodletting(g) => bloodletting::apply(state, events, *g),
        Card::Hemokinesis(g)  => hemokinesis::apply(state, events, *g, target),
        Card::BodySlam(_)     => body_slam::apply(state, events, target),
        Card::Anger(g)        => anger::apply(state, events, *g, target),
        Card::Dazed           => {} // unplayable — guarded before reaching apply()
    }
}

pub fn reward_pool() -> Vec<Card> {
    use Grade::Base;
    vec![
        Card::Bash(Base), Card::Clothesline(Base), Card::Inflame(Base), Card::DeadlyPoison(Base),
        Card::Cleave(Base), Card::IronWave(Base), Card::TwinStrike(Base), Card::Bludgeon(Base),
        Card::Impervious(Base), Card::NotYet(Base), Card::Mangle(Base), Card::Uppercut(Base),
        Card::Taunt(Base), Card::Thunderclap(Base),
        Card::PommelStrike(Base), Card::ShrugItOff(Base),
        Card::Breakthrough(Base), Card::BloodWall(Base), Card::Bloodletting(Base), Card::Hemokinesis(Base),
        Card::BodySlam(Base), Card::Anger(Base),
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

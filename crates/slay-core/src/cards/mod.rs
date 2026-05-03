mod bash;
mod blood_wall;
mod bloodletting;
mod breakthrough;
mod cleave;
mod clothesline;
mod deadly_poison;
mod defend;
mod disarm;
mod hemokinesis;
mod inflame;
mod iron_wave;
mod mangle;
mod not_yet;
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
    Strike,
    StrikePlus,
    Defend,
    DefendPlus,
    Bash,
    BashPlus,
    Clothesline,
    ClotheslinePlus,
    Inflame,
    InflamePlus,
    DeadlyPoison,
    DeadlyPoisonPlus,
    Disarm,
    Cleave,
    CleavePlus,
    IronWave,
    IronWavePlus,
    Tremble,
    TremblePlus,
    TwinStrike,
    TwinStrikePlus,
    Bludgeon,
    BludgeonPlus,
    Impervious,
    ImperviousPlus,
    NotYet,
    NotYetPlus,
    Mangle,
    ManglePlus,
    Uppercut,
    UppercutPlus,
    Taunt,
    TauntPlus,
    Thunderclap,
    ThunderclapPlus,
    Breakthrough,
    BreakthroughPlus,
    BloodWall,
    BloodWallPlus,
    Bloodletting,
    BloodlettingPlus,
    Hemokinesis,
    HemokinesisPlus,
}

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
            Card::Strike => CardDef {
                name: "Strike",
                description: CardDescription::WithDamage { template: "Deal {damage} damage.", base: 6 },
                energy_cost: Energy(1),
                card_type: CardType::Attack,
            },
            Card::StrikePlus => CardDef {
                name: "Strike+",
                description: CardDescription::WithDamage { template: "Deal {damage} damage.", base: 9 },
                energy_cost: Energy(1),
                card_type: CardType::Attack,
            },
            Card::Defend => CardDef {
                name: "Defend",
                description: CardDescription::Static("Gain 5 block."),
                energy_cost: Energy(1),
                card_type: CardType::Skill,
            },
            Card::DefendPlus => CardDef {
                name: "Defend+",
                description: CardDescription::Static("Gain 8 block."),
                energy_cost: Energy(1),
                card_type: CardType::Skill,
            },
            Card::Bash => CardDef {
                name: "Bash",
                description: CardDescription::WithDamage { template: "Deal {damage} damage. Apply 2 Vulnerable.", base: 8 },
                energy_cost: Energy(2),
                card_type: CardType::Attack,
            },
            Card::BashPlus => CardDef {
                name: "Bash+",
                description: CardDescription::WithDamage { template: "Deal {damage} damage. Apply 3 Vulnerable.", base: 10 },
                energy_cost: Energy(2),
                card_type: CardType::Attack,
            },
            Card::Clothesline => CardDef {
                name: "Clothesline",
                description: CardDescription::WithDamage { template: "Deal {damage} damage. Apply 2 Weak.", base: 12 },
                energy_cost: Energy(2),
                card_type: CardType::Attack,
            },
            Card::ClotheslinePlus => CardDef {
                name: "Clothesline+",
                description: CardDescription::WithDamage { template: "Deal {damage} damage. Apply 3 Weak.", base: 14 },
                energy_cost: Energy(2),
                card_type: CardType::Attack,
            },
            Card::Inflame => CardDef {
                name: "Inflame",
                description: CardDescription::Static("Gain 2 Strength."),
                energy_cost: Energy(1),
                card_type: CardType::Power,
            },
            Card::InflamePlus => CardDef {
                name: "Inflame+",
                description: CardDescription::Static("Gain 3 Strength."),
                energy_cost: Energy(1),
                card_type: CardType::Power,
            },
            Card::DeadlyPoison => CardDef {
                name: "Deadly Poison",
                description: CardDescription::Static("Apply 5 Poison."),
                energy_cost: Energy(1),
                card_type: CardType::Skill,
            },
            Card::DeadlyPoisonPlus => CardDef {
                name: "Deadly Poison+",
                description: CardDescription::Static("Apply 7 Poison."),
                energy_cost: Energy(1),
                card_type: CardType::Skill,
            },
            Card::Disarm => CardDef {
                name: "Disarm",
                description: CardDescription::Static("Enemy loses 2 Strength. Exhaust."),
                energy_cost: Energy(1),
                card_type: CardType::Skill,
            },
            Card::Cleave => CardDef {
                name: "Cleave",
                description: CardDescription::WithDamage { template: "Deal {damage} damage to ALL enemies.", base: 8 },
                energy_cost: Energy(1),
                card_type: CardType::Attack,
            },
            Card::CleavePlus => CardDef {
                name: "Cleave+",
                description: CardDescription::WithDamage { template: "Deal {damage} damage to ALL enemies.", base: 11 },
                energy_cost: Energy(1),
                card_type: CardType::Attack,
            },
            Card::IronWave => CardDef {
                name: "Iron Wave",
                description: CardDescription::WithDamage { template: "Gain 5 Block. Deal {damage} damage.", base: 5 },
                energy_cost: Energy(1),
                card_type: CardType::Attack,
            },
            Card::IronWavePlus => CardDef {
                name: "Iron Wave+",
                description: CardDescription::WithDamage { template: "Gain 7 Block. Deal {damage} damage.", base: 7 },
                energy_cost: Energy(1),
                card_type: CardType::Attack,
            },
            Card::Tremble => CardDef {
                name: "Tremble",
                description: CardDescription::Static("Apply 3 Vulnerable."),
                energy_cost: Energy(1),
                card_type: CardType::Skill,
            },
            Card::TremblePlus => CardDef {
                name: "Tremble+",
                description: CardDescription::Static("Apply 4 Vulnerable."),
                energy_cost: Energy(1),
                card_type: CardType::Skill,
            },
            Card::TwinStrike => CardDef {
                name: "Twin Strike",
                description: CardDescription::WithDamage { template: "Deal {damage} damage twice.", base: 5 },
                energy_cost: Energy(1),
                card_type: CardType::Attack,
            },
            Card::TwinStrikePlus => CardDef {
                name: "Twin Strike+",
                description: CardDescription::WithDamage { template: "Deal {damage} damage twice.", base: 7 },
                energy_cost: Energy(1),
                card_type: CardType::Attack,
            },
            Card::Bludgeon => CardDef {
                name: "Bludgeon",
                description: CardDescription::WithDamage { template: "Deal {damage} damage.", base: 32 },
                energy_cost: Energy(3),
                card_type: CardType::Attack,
            },
            Card::BludgeonPlus => CardDef {
                name: "Bludgeon+",
                description: CardDescription::WithDamage { template: "Deal {damage} damage.", base: 42 },
                energy_cost: Energy(3),
                card_type: CardType::Attack,
            },
            Card::Impervious => CardDef {
                name: "Impervious",
                description: CardDescription::Static("Gain 30 Block. Exhaust."),
                energy_cost: Energy(2),
                card_type: CardType::Skill,
            },
            Card::ImperviousPlus => CardDef {
                name: "Impervious+",
                description: CardDescription::Static("Gain 40 Block. Exhaust."),
                energy_cost: Energy(2),
                card_type: CardType::Skill,
            },
            Card::NotYet => CardDef {
                name: "Not Yet",
                description: CardDescription::Static("Heal 10 HP."),
                energy_cost: Energy(2),
                card_type: CardType::Skill,
            },
            Card::NotYetPlus => CardDef {
                name: "Not Yet+",
                description: CardDescription::Static("Heal 13 HP."),
                energy_cost: Energy(2),
                card_type: CardType::Skill,
            },
            Card::Mangle => CardDef {
                name: "Mangle",
                description: CardDescription::WithDamage { template: "Deal {damage} damage. Enemy loses 10 Strength.", base: 15 },
                energy_cost: Energy(3),
                card_type: CardType::Attack,
            },
            Card::ManglePlus => CardDef {
                name: "Mangle+",
                description: CardDescription::WithDamage { template: "Deal {damage} damage. Enemy loses 15 Strength.", base: 20 },
                energy_cost: Energy(3),
                card_type: CardType::Attack,
            },
            Card::Uppercut => CardDef {
                name: "Uppercut",
                description: CardDescription::WithDamage { template: "Deal {damage} damage. Apply 1 Weak. Apply 1 Vulnerable.", base: 13 },
                energy_cost: Energy(2),
                card_type: CardType::Attack,
            },
            Card::UppercutPlus => CardDef {
                name: "Uppercut+",
                description: CardDescription::WithDamage { template: "Deal {damage} damage. Apply 2 Weak. Apply 2 Vulnerable.", base: 13 },
                energy_cost: Energy(2),
                card_type: CardType::Attack,
            },
            Card::Taunt => CardDef {
                name: "Taunt",
                description: CardDescription::Static("Gain 7 Block. Apply 1 Vulnerable."),
                energy_cost: Energy(1),
                card_type: CardType::Skill,
            },
            Card::TauntPlus => CardDef {
                name: "Taunt+",
                description: CardDescription::Static("Gain 8 Block. Apply 2 Vulnerable."),
                energy_cost: Energy(1),
                card_type: CardType::Skill,
            },
            Card::Thunderclap => CardDef {
                name: "Thunderclap",
                description: CardDescription::WithDamage { template: "Deal {damage} damage and apply 1 Vulnerable to ALL enemies.", base: 4 },
                energy_cost: Energy(1),
                card_type: CardType::Attack,
            },
            Card::ThunderclapPlus => CardDef {
                name: "Thunderclap+",
                description: CardDescription::WithDamage { template: "Deal {damage} damage and apply 1 Vulnerable to ALL enemies.", base: 7 },
                energy_cost: Energy(1),
                card_type: CardType::Attack,
            },
            Card::Breakthrough => CardDef {
                name: "Breakthrough",
                description: CardDescription::WithDamage { template: "Lose 1 HP. Deal {damage} damage to ALL enemies.", base: 9 },
                energy_cost: Energy(1),
                card_type: CardType::Attack,
            },
            Card::BreakthroughPlus => CardDef {
                name: "Breakthrough+",
                description: CardDescription::WithDamage { template: "Lose 1 HP. Deal {damage} damage to ALL enemies.", base: 13 },
                energy_cost: Energy(1),
                card_type: CardType::Attack,
            },
            Card::BloodWall => CardDef {
                name: "Blood Wall",
                description: CardDescription::Static("Lose 2 HP. Gain 16 Block."),
                energy_cost: Energy(2),
                card_type: CardType::Skill,
            },
            Card::BloodWallPlus => CardDef {
                name: "Blood Wall+",
                description: CardDescription::Static("Lose 2 HP. Gain 20 Block."),
                energy_cost: Energy(2),
                card_type: CardType::Skill,
            },
            Card::Bloodletting => CardDef {
                name: "Bloodletting",
                description: CardDescription::Static("Lose 3 HP. Gain 2 Energy."),
                energy_cost: Energy(0),
                card_type: CardType::Skill,
            },
            Card::BloodlettingPlus => CardDef {
                name: "Bloodletting+",
                description: CardDescription::Static("Lose 3 HP. Gain 3 Energy."),
                energy_cost: Energy(0),
                card_type: CardType::Skill,
            },
            Card::Hemokinesis => CardDef {
                name: "Hemokinesis",
                description: CardDescription::WithDamage { template: "Lose 2 HP. Deal {damage} damage.", base: 15 },
                energy_cost: Energy(1),
                card_type: CardType::Attack,
            },
            Card::HemokinesisPlus => CardDef {
                name: "Hemokinesis+",
                description: CardDescription::WithDamage { template: "Lose 2 HP. Deal {damage} damage.", base: 20 },
                energy_cost: Energy(1),
                card_type: CardType::Attack,
            },
        }
    }

    pub fn exhausts(&self) -> bool {
        matches!(self, Card::Disarm | Card::Impervious | Card::ImperviousPlus)
    }

    pub fn upgrade(&self) -> Option<Card> {
        match self {
            Card::Strike => Some(Card::StrikePlus),
            Card::Defend => Some(Card::DefendPlus),
            Card::Bash => Some(Card::BashPlus),
            Card::Clothesline => Some(Card::ClotheslinePlus),
            Card::Inflame => Some(Card::InflamePlus),
            Card::DeadlyPoison => Some(Card::DeadlyPoisonPlus),
            Card::Cleave      => Some(Card::CleavePlus),
            Card::IronWave    => Some(Card::IronWavePlus),
            Card::Tremble     => Some(Card::TremblePlus),
            Card::TwinStrike  => Some(Card::TwinStrikePlus),
            Card::Bludgeon    => Some(Card::BludgeonPlus),
            Card::Impervious  => Some(Card::ImperviousPlus),
            Card::NotYet      => Some(Card::NotYetPlus),
            Card::Mangle      => Some(Card::ManglePlus),
            Card::Uppercut    => Some(Card::UppercutPlus),
            Card::Taunt       => Some(Card::TauntPlus),
            Card::Thunderclap  => Some(Card::ThunderclapPlus),
            Card::Breakthrough => Some(Card::BreakthroughPlus),
            Card::BloodWall    => Some(Card::BloodWallPlus),
            Card::Bloodletting => Some(Card::BloodlettingPlus),
            Card::Hemokinesis  => Some(Card::HemokinesisPlus),
            _ => None,
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

    pub fn effective_damage(&self, attacker: &StatusMap, defender: &StatusMap) -> Option<i32> {
        match self.def().description {
            CardDescription::WithDamage { base, .. } => Some(resolve_damage(base, attacker, defender)),
            CardDescription::Static(_) => None,
        }
    }
}

pub fn reward_pool() -> Vec<Card> {
    vec![
        Card::Bash, Card::Clothesline, Card::Inflame, Card::DeadlyPoison,
        Card::Cleave, Card::IronWave, Card::TwinStrike, Card::Bludgeon,
        Card::Impervious, Card::NotYet, Card::Mangle, Card::Uppercut,
        Card::Taunt, Card::Thunderclap,
    ]
}

pub fn starter_deck() -> Vec<Card> {
    let mut deck = Vec::new();
    for _ in 0..5 {
        deck.push(Card::Strike);
    }
    for _ in 0..3 {
        deck.push(Card::Defend);
    }
    deck.push(Card::Bash);
    deck.push(Card::Inflame);
    deck.push(Card::DeadlyPoison);
    deck.push(Card::Disarm);
    deck
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::{combat_with_hand, combat_with_two_enemies, apply_combat_command, CombatPhase, Event, Target};
    use crate::run::{Command, CommandError};
    use crate::status::StatusEffect;
    use crate::types::{Block, Energy, Hp};
    use crate::rng::NoOpRng;

    fn rng() -> NoOpRng { NoOpRng }

    fn apply_command(
        state: crate::combat::CombatState,
        cmd: Command,
        rng: &mut impl crate::rng::Rng,
    ) -> Result<(crate::combat::CombatState, Vec<Event>), CommandError> {
        apply_combat_command(state, cmd, rng)
    }

    // --- Strike ---

    #[test]
    fn strike_deals_6_damage_to_enemy() {
        let state = combat_with_hand(vec![Card::Strike]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(14));
    }

    #[test]
    fn strike_emits_player_attacked_event() {
        let state = combat_with_hand(vec![Card::Strike]);
        let (_, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(events.contains(&Event::PlayerAttacked { raw: 6, damage: 6 }));
    }

    #[test]
    fn strike_killing_enemy_yields_victory() {
        let mut state = combat_with_hand(vec![Card::Strike]);
        state.enemies[0].hp = Hp(1);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.phase, CombatPhase::Victory);
    }

    #[test]
    fn strike_killing_enemy_emits_enemy_died_event() {
        let mut state = combat_with_hand(vec![Card::Strike]);
        state.enemies[0].hp = Hp(1);
        let (_, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(events.contains(&Event::EnemyDied));
    }

    #[test]
    fn strike_moves_to_discard_after_play() {
        let state = combat_with_hand(vec![Card::Strike]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 0);
        assert_eq!(state.player.discard_pile, vec![Card::Strike]);
    }

    #[test]
    fn strike_goes_to_discard_not_exhaust() {
        let state = combat_with_hand(vec![Card::Strike]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.discard_pile, vec![Card::Strike]);
        assert!(state.player.exhaust_pile.is_empty());
    }

    // --- Defend ---

    #[test]
    fn defend_grants_5_block() {
        let state = combat_with_hand(vec![Card::Defend]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(5));
    }

    #[test]
    fn defend_emits_player_blocked_event() {
        let state = combat_with_hand(vec![Card::Defend]);
        let (_, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(events.contains(&Event::PlayerBlocked { amount: 5 }));
    }

    // --- Bash ---

    #[test]
    fn bash_deals_8_damage_to_enemy() {
        let state = combat_with_hand(vec![Card::Bash]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(12));
    }

    #[test]
    fn bash_costs_2_energy() {
        let state = combat_with_hand(vec![Card::Bash]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.energy, Energy(1));
    }

    #[test]
    fn bash_applies_2_vulnerable_to_enemy() {
        let state = combat_with_hand(vec![Card::Bash]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Vulnerable), Some(&2));
    }

    #[test]
    fn bash_emits_status_applied_event() {
        let state = combat_with_hand(vec![Card::Bash]);
        let (_, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(events.contains(&Event::StatusApplied {
            target: Target::Enemy,
            status: StatusEffect::Vulnerable,
            stacks: 2,
        }));
    }

    #[test]
    fn strike_damage_boosted_against_vulnerable_enemy() {
        let mut state = combat_with_hand(vec![Card::Strike]);
        state.enemies[0].statuses.insert(StatusEffect::Vulnerable, 2);
        let (_, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(events.contains(&Event::PlayerAttacked { raw: 9, damage: 9 }));
    }

    // --- Clothesline ---

    #[test]
    fn clothesline_deals_12_damage_to_enemy() {
        let state = combat_with_hand(vec![Card::Clothesline]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(8));
    }

    #[test]
    fn clothesline_applies_2_weak_to_enemy() {
        let state = combat_with_hand(vec![Card::Clothesline]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Weak), Some(&2));
    }

    // --- Deadly Poison ---

    #[test]
    fn deadly_poison_applies_5_poison_to_enemy() {
        let state = combat_with_hand(vec![Card::DeadlyPoison]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Poison), Some(&5));
    }

    #[test]
    fn deadly_poison_deals_no_direct_damage() {
        let state = combat_with_hand(vec![Card::DeadlyPoison]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(20));
    }

    // --- CardType ---

    #[test]
    fn strike_card_type_is_attack() {
        assert_eq!(Card::Strike.card_type(), CardType::Attack);
    }

    #[test]
    fn bash_card_type_is_attack() {
        assert_eq!(Card::Bash.card_type(), CardType::Attack);
    }

    #[test]
    fn clothesline_card_type_is_attack() {
        assert_eq!(Card::Clothesline.card_type(), CardType::Attack);
    }

    #[test]
    fn defend_card_type_is_skill() {
        assert_eq!(Card::Defend.card_type(), CardType::Skill);
    }

    #[test]
    fn deadly_poison_card_type_is_skill() {
        assert_eq!(Card::DeadlyPoison.card_type(), CardType::Skill);
    }

    #[test]
    fn disarm_card_type_is_skill() {
        assert_eq!(Card::Disarm.card_type(), CardType::Skill);
    }

    #[test]
    fn inflame_card_type_is_power() {
        assert_eq!(Card::Inflame.card_type(), CardType::Power);
    }

    // --- Inflame ---

    #[test]
    fn inflame_grants_2_strength_to_player() {
        let state = combat_with_hand(vec![Card::Inflame]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Strength), Some(&2));
    }

    #[test]
    fn inflame_is_absorbed_not_discarded() {
        let state = combat_with_hand(vec![Card::Inflame]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.discard_pile.is_empty());
    }

    #[test]
    fn inflame_is_absorbed_not_exhausted() {
        let state = combat_with_hand(vec![Card::Inflame]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.is_empty());
    }

    // --- Upgraded effects ---

    #[test]
    fn strike_plus_deals_9_damage() {
        let state = combat_with_hand(vec![Card::StrikePlus]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(11));
    }

    #[test]
    fn defend_plus_grants_8_block() {
        let state = combat_with_hand(vec![Card::DefendPlus]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(8));
    }

    #[test]
    fn bash_plus_deals_10_damage() {
        let state = combat_with_hand(vec![Card::BashPlus]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(10));
    }

    #[test]
    fn bash_plus_applies_3_vulnerable() {
        let state = combat_with_hand(vec![Card::BashPlus]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Vulnerable), Some(&3));
    }

    #[test]
    fn clothesline_plus_deals_14_damage() {
        let state = combat_with_hand(vec![Card::ClotheslinePlus]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(6));
    }

    #[test]
    fn clothesline_plus_applies_3_weak() {
        let state = combat_with_hand(vec![Card::ClotheslinePlus]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Weak), Some(&3));
    }

    #[test]
    fn inflame_plus_grants_3_strength() {
        let state = combat_with_hand(vec![Card::InflamePlus]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Strength), Some(&3));
    }

    #[test]
    fn deadly_poison_plus_applies_7_poison() {
        let state = combat_with_hand(vec![Card::DeadlyPoisonPlus]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Poison), Some(&7));
    }

    // --- Card::upgrade() ---

    #[test]
    fn upgrading_strike_gives_strike_plus() {
        assert_eq!(Card::Strike.upgrade(), Some(Card::StrikePlus));
    }

    #[test]
    fn upgrading_defend_gives_defend_plus() {
        assert_eq!(Card::Defend.upgrade(), Some(Card::DefendPlus));
    }

    #[test]
    fn upgrading_bash_gives_bash_plus() {
        assert_eq!(Card::Bash.upgrade(), Some(Card::BashPlus));
    }

    #[test]
    fn upgrading_clothesline_gives_clothesline_plus() {
        assert_eq!(Card::Clothesline.upgrade(), Some(Card::ClotheslinePlus));
    }

    #[test]
    fn upgrading_inflame_gives_inflame_plus() {
        assert_eq!(Card::Inflame.upgrade(), Some(Card::InflamePlus));
    }

    #[test]
    fn upgrading_deadly_poison_gives_deadly_poison_plus() {
        assert_eq!(Card::DeadlyPoison.upgrade(), Some(Card::DeadlyPoisonPlus));
    }

    #[test]
    fn upgrading_plus_card_returns_none() {
        assert_eq!(Card::StrikePlus.upgrade(), None);
    }

    #[test]
    fn disarm_cannot_be_upgraded() {
        assert_eq!(Card::Disarm.upgrade(), None);
    }

    // --- Disarm ---

    #[test]
    fn disarm_applies_minus_2_strength_to_enemy() {
        let state = combat_with_hand(vec![Card::Disarm]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Strength), Some(&-2));
    }

    #[test]
    fn disarm_goes_to_exhaust_pile_not_discard() {
        let state = combat_with_hand(vec![Card::Disarm]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.exhaust_pile, vec![Card::Disarm]);
        assert!(state.player.discard_pile.is_empty());
    }

    #[test]
    fn disarm_emits_card_exhausted_event() {
        let state = combat_with_hand(vec![Card::Disarm]);
        let (_, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(events.contains(&Event::CardExhausted { card: Card::Disarm }));
    }

    // --- Iron Wave ---

    #[test]
    fn iron_wave_deals_5_damage_and_grants_5_block() {
        let state = combat_with_hand(vec![Card::IronWave]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(15));
        assert_eq!(state.player.block, Block(5));
    }

    #[test]
    fn iron_wave_plus_deals_7_damage_and_grants_7_block() {
        let state = combat_with_hand(vec![Card::IronWavePlus]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(13));
        assert_eq!(state.player.block, Block(7));
    }

    // --- Tremble ---

    #[test]
    fn tremble_applies_3_vulnerable_to_enemy() {
        let state = combat_with_hand(vec![Card::Tremble]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Vulnerable), Some(&3));
    }

    #[test]
    fn tremble_plus_applies_4_vulnerable() {
        let state = combat_with_hand(vec![Card::TremblePlus]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Vulnerable), Some(&4));
    }

    // --- Twin Strike ---

    #[test]
    fn twin_strike_deals_5_damage_twice() {
        let state = combat_with_hand(vec![Card::TwinStrike]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(10));
    }

    #[test]
    fn twin_strike_plus_deals_7_damage_twice() {
        let state = combat_with_hand(vec![Card::TwinStrikePlus]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(6));
    }

    // --- Bludgeon ---

    #[test]
    fn bludgeon_deals_32_damage() {
        let mut state = combat_with_hand(vec![Card::Bludgeon]);
        state.enemies[0].hp = Hp(50);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(18));
    }

    #[test]
    fn bludgeon_plus_deals_42_damage() {
        let mut state = combat_with_hand(vec![Card::BludgeonPlus]);
        state.enemies[0].hp = Hp(50);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(8));
    }

    // --- Impervious ---

    #[test]
    fn impervious_grants_30_block() {
        let state = combat_with_hand(vec![Card::Impervious]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(30));
    }

    #[test]
    fn impervious_goes_to_exhaust_pile() {
        let state = combat_with_hand(vec![Card::Impervious]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.exhaust_pile, vec![Card::Impervious]);
        assert!(state.player.discard_pile.is_empty());
    }

    // --- Not Yet ---

    #[test]
    fn not_yet_heals_10_hp() {
        let mut state = combat_with_hand(vec![Card::NotYet]);
        state.player.hp = Hp(50);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(60));
    }

    #[test]
    fn not_yet_cannot_overheal() {
        let state = combat_with_hand(vec![Card::NotYet]); // already at 80/80
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(80));
    }

    #[test]
    fn not_yet_plus_heals_13_hp() {
        let mut state = combat_with_hand(vec![Card::NotYetPlus]);
        state.player.hp = Hp(50);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(63));
    }

    // --- Mangle ---

    #[test]
    fn mangle_deals_15_damage() {
        let state = combat_with_hand(vec![Card::Mangle]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(5));
    }

    #[test]
    fn mangle_reduces_enemy_strength_by_10() {
        let state = combat_with_hand(vec![Card::Mangle]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Strength), Some(&-10));
    }

    #[test]
    fn mangle_plus_deals_20_damage() {
        let mut state = combat_with_hand(vec![Card::ManglePlus]);
        state.enemies[0].hp = Hp(50);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(30));
    }

    // --- Uppercut ---

    #[test]
    fn uppercut_deals_13_damage() {
        let state = combat_with_hand(vec![Card::Uppercut]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(7));
    }

    #[test]
    fn uppercut_applies_1_weak_and_1_vulnerable() {
        let state = combat_with_hand(vec![Card::Uppercut]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Weak), Some(&1));
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Vulnerable), Some(&1));
    }

    #[test]
    fn uppercut_plus_applies_2_weak_and_2_vulnerable() {
        let state = combat_with_hand(vec![Card::UppercutPlus]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Weak), Some(&2));
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Vulnerable), Some(&2));
    }

    // --- Taunt ---

    #[test]
    fn taunt_grants_7_block_and_applies_1_vulnerable() {
        let state = combat_with_hand(vec![Card::Taunt]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(7));
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Vulnerable), Some(&1));
    }

    #[test]
    fn taunt_plus_grants_8_block_and_2_vulnerable() {
        let state = combat_with_hand(vec![Card::TauntPlus]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(8));
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Vulnerable), Some(&2));
    }

    // --- Thunderclap ---

    #[test]
    fn thunderclap_deals_4_damage_and_applies_1_vulnerable_to_all_enemies() {
        let state = combat_with_two_enemies(vec![Card::Thunderclap]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(16));
        assert_eq!(state.enemies[1].hp, Hp(16));
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Vulnerable), Some(&1));
        assert_eq!(state.enemies[1].statuses.get(&StatusEffect::Vulnerable), Some(&1));
    }

    #[test]
    fn thunderclap_plus_deals_7_damage_to_all_enemies() {
        let state = combat_with_two_enemies(vec![Card::ThunderclapPlus]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(13));
        assert_eq!(state.enemies[1].hp, Hp(13));
    }

    // --- Cleave ---

    #[test]
    fn cleave_deals_8_damage_to_single_enemy() {
        let state = combat_with_hand(vec![Card::Cleave]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(12));
    }

    #[test]
    fn cleave_deals_8_damage_to_all_enemies() {
        let state = combat_with_two_enemies(vec![Card::Cleave]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(12));
        assert_eq!(state.enemies[1].hp, Hp(12));
    }

    #[test]
    fn cleave_plus_deals_11_damage_to_all_enemies() {
        let state = combat_with_two_enemies(vec![Card::CleavePlus]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(9));
        assert_eq!(state.enemies[1].hp, Hp(9));
    }

    #[test]
    fn cleave_card_type_is_attack() {
        assert_eq!(Card::Cleave.card_type(), CardType::Attack);
    }

    #[test]
    fn upgrading_cleave_gives_cleave_plus() {
        assert_eq!(Card::Cleave.upgrade(), Some(Card::CleavePlus));
    }

    // --- Breakthrough ---

    #[test]
    fn breakthrough_deals_9_damage_to_all_enemies() {
        let state = combat_with_two_enemies(vec![Card::Breakthrough]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(11));
        assert_eq!(state.enemies[1].hp, Hp(11));
    }

    #[test]
    fn breakthrough_costs_1_hp() {
        let state = combat_with_hand(vec![Card::Breakthrough]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(79));
    }

    #[test]
    fn breakthrough_emits_player_self_damaged_event() {
        let state = combat_with_hand(vec![Card::Breakthrough]);
        let (_, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(events.contains(&Event::PlayerSelfDamaged { amount: 1 }));
    }

    #[test]
    fn breakthrough_plus_deals_13_damage_to_all_enemies() {
        let state = combat_with_two_enemies(vec![Card::BreakthroughPlus]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(7));
        assert_eq!(state.enemies[1].hp, Hp(7));
    }

    // --- Blood Wall ---

    #[test]
    fn blood_wall_grants_16_block() {
        let state = combat_with_hand(vec![Card::BloodWall]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(16));
    }

    #[test]
    fn blood_wall_costs_2_hp() {
        let state = combat_with_hand(vec![Card::BloodWall]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(78));
    }

    #[test]
    fn blood_wall_plus_grants_20_block() {
        let state = combat_with_hand(vec![Card::BloodWallPlus]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(20));
    }

    // --- Bloodletting ---

    #[test]
    fn bloodletting_gains_2_energy() {
        let mut state = combat_with_hand(vec![Card::Bloodletting]);
        state.player.energy = Energy(0); // drain energy so we can measure the gain
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.energy, Energy(2));
    }

    #[test]
    fn bloodletting_costs_3_hp() {
        let state = combat_with_hand(vec![Card::Bloodletting]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(77));
    }

    #[test]
    fn bloodletting_emits_energy_gained_event() {
        let state = combat_with_hand(vec![Card::Bloodletting]);
        let (_, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(events.contains(&Event::EnergyGained { amount: 2 }));
    }

    #[test]
    fn bloodletting_plus_gains_3_energy() {
        let mut state = combat_with_hand(vec![Card::BloodlettingPlus]);
        state.player.energy = Energy(0);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.energy, Energy(3));
    }

    // --- Hemokinesis ---

    #[test]
    fn hemokinesis_deals_15_damage() {
        let state = combat_with_hand(vec![Card::Hemokinesis]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(5));
    }

    #[test]
    fn hemokinesis_costs_2_hp() {
        let state = combat_with_hand(vec![Card::Hemokinesis]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(78));
    }

    #[test]
    fn hemokinesis_plus_deals_20_damage() {
        let mut state = combat_with_hand(vec![Card::HemokinesisPlus]);
        state.enemies[0].hp = Hp(50);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(30));
    }

    // --- Self-damage defeat ---

    #[test]
    fn self_damage_killing_player_yields_defeat() {
        let mut state = combat_with_hand(vec![Card::Bloodletting]);
        state.player.hp = Hp(1);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.phase, CombatPhase::Defeat);
    }
}

pub fn apply(card: &Card, state: &mut crate::combat::CombatState, events: &mut Vec<crate::combat::Event>, target: usize) {
    match card {
        Card::Strike      => strike::apply(state, events, 6, target),
        Card::StrikePlus  => strike::apply(state, events, 9, target),
        Card::Defend      => defend::apply(state, events, 5, target),
        Card::DefendPlus  => defend::apply(state, events, 8, target),
        Card::Bash        => bash::apply(state, events, 8, 2, target),
        Card::BashPlus    => bash::apply(state, events, 10, 3, target),
        Card::Clothesline      => clothesline::apply(state, events, 12, 2, target),
        Card::ClotheslinePlus  => clothesline::apply(state, events, 14, 3, target),
        Card::Inflame     => inflame::apply(state, events, 2, target),
        Card::InflamePlus => inflame::apply(state, events, 3, target),
        Card::DeadlyPoison      => deadly_poison::apply(state, events, 5, target),
        Card::DeadlyPoisonPlus  => deadly_poison::apply(state, events, 7, target),
        Card::Disarm => disarm::apply(state, events, target),
        Card::Cleave     => cleave::apply(state, events, 8),
        Card::CleavePlus => cleave::apply(state, events, 11),
        Card::IronWave    => iron_wave::apply(state, events, 5, 5, target),
        Card::IronWavePlus => iron_wave::apply(state, events, 7, 7, target),
        Card::Tremble    => tremble::apply(state, events, 3, target),
        Card::TremblePlus => tremble::apply(state, events, 4, target),
        Card::TwinStrike    => twin_strike::apply(state, events, 5, target),
        Card::TwinStrikePlus => twin_strike::apply(state, events, 7, target),
        Card::Bludgeon    => strike::apply(state, events, 32, target),
        Card::BludgeonPlus => strike::apply(state, events, 42, target),
        Card::Impervious    => defend::apply(state, events, 30, target),
        Card::ImperviousPlus => defend::apply(state, events, 40, target),
        Card::NotYet    => not_yet::apply(state, events, 10, target),
        Card::NotYetPlus => not_yet::apply(state, events, 13, target),
        Card::Mangle    => mangle::apply(state, events, 15, 10, target),
        Card::ManglePlus => mangle::apply(state, events, 20, 15, target),
        Card::Uppercut    => uppercut::apply(state, events, 13, 1, 1, target),
        Card::UppercutPlus => uppercut::apply(state, events, 13, 2, 2, target),
        Card::Taunt    => taunt::apply(state, events, 7, 1, target),
        Card::TauntPlus => taunt::apply(state, events, 8, 2, target),
        Card::Thunderclap    => thunderclap::apply(state, events, 4, 1),
        Card::ThunderclapPlus => thunderclap::apply(state, events, 7, 1),
        Card::Breakthrough    => breakthrough::apply(state, events, 1, 9),
        Card::BreakthroughPlus => breakthrough::apply(state, events, 1, 13),
        Card::BloodWall    => blood_wall::apply(state, events, 2, 16),
        Card::BloodWallPlus => blood_wall::apply(state, events, 2, 20),
        Card::Bloodletting    => bloodletting::apply(state, events, 3, 2),
        Card::BloodlettingPlus => bloodletting::apply(state, events, 3, 3),
        Card::Hemokinesis    => hemokinesis::apply(state, events, 2, 15, target),
        Card::HemokinesisPlus => hemokinesis::apply(state, events, 2, 20, target),
    }
}

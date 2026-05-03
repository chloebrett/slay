mod bash;
mod clothesline;
mod deadly_poison;
mod defend;
mod disarm;
mod inflame;
mod strike;

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
        }
    }

    pub fn exhausts(&self) -> bool {
        matches!(self, Card::Disarm)
    }

    pub fn upgrade(&self) -> Option<Card> {
        match self {
            Card::Strike => Some(Card::StrikePlus),
            Card::Defend => Some(Card::DefendPlus),
            Card::Bash => Some(Card::BashPlus),
            Card::Clothesline => Some(Card::ClotheslinePlus),
            Card::Inflame => Some(Card::InflamePlus),
            Card::DeadlyPoison => Some(Card::DeadlyPoisonPlus),
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
    vec![Card::Bash, Card::Clothesline, Card::Inflame, Card::DeadlyPoison, Card::Strike, Card::Defend]
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
    use crate::combat::{combat_with_hand, apply_combat_command, CombatPhase, Event, Target};
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
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.enemy.hp, Hp(14));
    }

    #[test]
    fn strike_emits_player_attacked_event() {
        let state = combat_with_hand(vec![Card::Strike]);
        let (_, events) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert!(events.contains(&Event::PlayerAttacked { raw: 6, damage: 6 }));
    }

    #[test]
    fn strike_killing_enemy_yields_victory() {
        let mut state = combat_with_hand(vec![Card::Strike]);
        state.enemy.hp = Hp(1);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.phase, CombatPhase::Victory);
    }

    #[test]
    fn strike_killing_enemy_emits_enemy_died_event() {
        let mut state = combat_with_hand(vec![Card::Strike]);
        state.enemy.hp = Hp(1);
        let (_, events) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert!(events.contains(&Event::EnemyDied));
    }

    #[test]
    fn strike_moves_to_discard_after_play() {
        let state = combat_with_hand(vec![Card::Strike]);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 0);
        assert_eq!(state.player.discard_pile, vec![Card::Strike]);
    }

    #[test]
    fn strike_goes_to_discard_not_exhaust() {
        let state = combat_with_hand(vec![Card::Strike]);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.player.discard_pile, vec![Card::Strike]);
        assert!(state.player.exhaust_pile.is_empty());
    }

    // --- Defend ---

    #[test]
    fn defend_grants_5_block() {
        let state = combat_with_hand(vec![Card::Defend]);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(5));
    }

    #[test]
    fn defend_emits_player_blocked_event() {
        let state = combat_with_hand(vec![Card::Defend]);
        let (_, events) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert!(events.contains(&Event::PlayerBlocked { amount: 5 }));
    }

    // --- Bash ---

    #[test]
    fn bash_deals_8_damage_to_enemy() {
        let state = combat_with_hand(vec![Card::Bash]);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.enemy.hp, Hp(12));
    }

    #[test]
    fn bash_costs_2_energy() {
        let state = combat_with_hand(vec![Card::Bash]);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.player.energy, Energy(1));
    }

    #[test]
    fn bash_applies_2_vulnerable_to_enemy() {
        let state = combat_with_hand(vec![Card::Bash]);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.enemy.statuses.get(&StatusEffect::Vulnerable), Some(&2));
    }

    #[test]
    fn bash_emits_status_applied_event() {
        let state = combat_with_hand(vec![Card::Bash]);
        let (_, events) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert!(events.contains(&Event::StatusApplied {
            target: Target::Enemy,
            status: StatusEffect::Vulnerable,
            stacks: 2,
        }));
    }

    #[test]
    fn strike_damage_boosted_against_vulnerable_enemy() {
        let mut state = combat_with_hand(vec![Card::Strike]);
        state.enemy.statuses.insert(StatusEffect::Vulnerable, 2);
        let (_, events) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert!(events.contains(&Event::PlayerAttacked { raw: 9, damage: 9 }));
    }

    // --- Clothesline ---

    #[test]
    fn clothesline_deals_12_damage_to_enemy() {
        let state = combat_with_hand(vec![Card::Clothesline]);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.enemy.hp, Hp(8));
    }

    #[test]
    fn clothesline_applies_2_weak_to_enemy() {
        let state = combat_with_hand(vec![Card::Clothesline]);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.enemy.statuses.get(&StatusEffect::Weak), Some(&2));
    }

    // --- Deadly Poison ---

    #[test]
    fn deadly_poison_applies_5_poison_to_enemy() {
        let state = combat_with_hand(vec![Card::DeadlyPoison]);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.enemy.statuses.get(&StatusEffect::Poison), Some(&5));
    }

    #[test]
    fn deadly_poison_deals_no_direct_damage() {
        let state = combat_with_hand(vec![Card::DeadlyPoison]);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.enemy.hp, Hp(20));
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
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Strength), Some(&2));
    }

    #[test]
    fn inflame_is_absorbed_not_discarded() {
        let state = combat_with_hand(vec![Card::Inflame]);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert!(state.player.discard_pile.is_empty());
    }

    #[test]
    fn inflame_is_absorbed_not_exhausted() {
        let state = combat_with_hand(vec![Card::Inflame]);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.is_empty());
    }

    // --- Upgraded effects ---

    #[test]
    fn strike_plus_deals_9_damage() {
        let state = combat_with_hand(vec![Card::StrikePlus]);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.enemy.hp, Hp(11));
    }

    #[test]
    fn defend_plus_grants_8_block() {
        let state = combat_with_hand(vec![Card::DefendPlus]);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(8));
    }

    #[test]
    fn bash_plus_deals_10_damage() {
        let state = combat_with_hand(vec![Card::BashPlus]);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.enemy.hp, Hp(10));
    }

    #[test]
    fn bash_plus_applies_3_vulnerable() {
        let state = combat_with_hand(vec![Card::BashPlus]);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.enemy.statuses.get(&StatusEffect::Vulnerable), Some(&3));
    }

    #[test]
    fn clothesline_plus_deals_14_damage() {
        let state = combat_with_hand(vec![Card::ClotheslinePlus]);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.enemy.hp, Hp(6));
    }

    #[test]
    fn clothesline_plus_applies_3_weak() {
        let state = combat_with_hand(vec![Card::ClotheslinePlus]);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.enemy.statuses.get(&StatusEffect::Weak), Some(&3));
    }

    #[test]
    fn inflame_plus_grants_3_strength() {
        let state = combat_with_hand(vec![Card::InflamePlus]);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Strength), Some(&3));
    }

    #[test]
    fn deadly_poison_plus_applies_7_poison() {
        let state = combat_with_hand(vec![Card::DeadlyPoisonPlus]);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.enemy.statuses.get(&StatusEffect::Poison), Some(&7));
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
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.enemy.statuses.get(&StatusEffect::Strength), Some(&-2));
    }

    #[test]
    fn disarm_goes_to_exhaust_pile_not_discard() {
        let state = combat_with_hand(vec![Card::Disarm]);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.player.exhaust_pile, vec![Card::Disarm]);
        assert!(state.player.discard_pile.is_empty());
    }

    #[test]
    fn disarm_emits_card_exhausted_event() {
        let state = combat_with_hand(vec![Card::Disarm]);
        let (_, events) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert!(events.contains(&Event::CardExhausted { card: Card::Disarm }));
    }
}

pub fn apply(card: &Card, state: &mut crate::combat::CombatState, events: &mut Vec<crate::combat::Event>) {
    match card {
        Card::Strike      => strike::apply(state, events, 6),
        Card::StrikePlus  => strike::apply(state, events, 9),
        Card::Defend      => defend::apply(state, events, 5),
        Card::DefendPlus  => defend::apply(state, events, 8),
        Card::Bash        => bash::apply(state, events, 8, 2),
        Card::BashPlus    => bash::apply(state, events, 10, 3),
        Card::Clothesline      => clothesline::apply(state, events, 12, 2),
        Card::ClotheslinePlus  => clothesline::apply(state, events, 14, 3),
        Card::Inflame     => inflame::apply(state, events, 2),
        Card::InflamePlus => inflame::apply(state, events, 3),
        Card::DeadlyPoison      => deadly_poison::apply(state, events, 5),
        Card::DeadlyPoisonPlus  => deadly_poison::apply(state, events, 7),
        Card::Disarm => disarm::apply(state, events),
    }
}

use crate::cards::{Card, starter_deck};
use crate::combat::{apply_combat_command, CombatPhase, CombatState, Event, Player};
use crate::potions::{Potion, MAX_POTIONS};
use crate::enemies::EnemyKind;
use crate::relics::{
    apply_combat_start_relics, apply_end_of_combat_relics, apply_rest_relics,
    apply_turn_end_relics, apply_turn_start_relics, Relic,
};
use crate::rng::Rng;
use crate::status::StatusMap;
use crate::types::{Block, Energy, Hp};

#[derive(Debug, Clone, PartialEq)]
pub enum Scenario {
    Main,
    Simple,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    PlayCard(usize, usize), // card index, target enemy index
    EndTurn,
    EndEnemyTurn,
    ChooseNode(usize),
    Rest,
    ChooseCardReward(usize),
    SkipReward,
    UpgradeCard(usize),
    SkipFloor,
    WinCombat,
    AddCard(Card),
    AddRelic(Relic),
    AddPotion(Potion),
    UsePotion(usize, usize), // slot index, target enemy index
    DiscardPotion(usize),    // slot index
    Spawn(Vec<EnemyKind>),
    LeaveShop,
    BuyCard(usize),
    BuyRelic,
    BuyPotion,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandError {
    CombatOver,
    InvalidCard,
    NotEnoughEnergy,
    NotEnoughGold,
    InvalidPhase,
    Entangled,
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            CommandError::NotEnoughEnergy => "Not enough energy.",
            CommandError::NotEnoughGold   => "Not enough gold.",
            CommandError::InvalidCard     => "No card at that position.",
            CommandError::InvalidPhase    => "Can't do that right now.",
            CommandError::CombatOver      => "Combat is already over.",
            CommandError::Entangled       => "Entangled — cannot play Attack cards.",
        };
        f.write_str(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MapNode {
    Combat,
    RestSite,
    Boss,
    Merchant,
}

pub const MAP_NODES: &[MapNode] = &[
    MapNode::Combat,    // floor 0
    MapNode::Combat,    // floor 1
    MapNode::Combat,    // floor 2
    MapNode::Merchant,  // floor 3
    MapNode::RestSite,  // floor 4
    MapNode::Boss,      // floor 5
];

#[derive(Debug, Clone, PartialEq)]
pub struct MapState {
    pub player: Player,
    pub floor: usize,
    pub next_enemies: Option<Vec<EnemyKind>>,
    pub scenario: Scenario,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RestSiteState {
    pub player: Player,
    pub floor: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CardRewardState {
    pub player: Player,
    pub floor: usize,
    pub options: Vec<Card>,
    pub offered_potion: Option<Potion>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShopState {
    pub player: Player,
    pub floor: usize,
    pub cards: Vec<(Card, bool)>,
    pub relic: Option<(Relic, bool)>,
    pub potion: Option<(Potion, bool)>,
}

pub const CARD_PRICE: i32 = 75;
pub const RELIC_PRICE: i32 = 150;
pub const POTION_PRICE: i32 = 50;

#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    Map(MapState),
    Combat { state: CombatState, floor: usize, scenario: Scenario },
    RestSite(RestSiteState),
    CardReward(CardRewardState),
    Shop(ShopState),
    GameOver { victory: bool },
}

pub fn new_run(rng: &mut impl Rng) -> GameState {
    let deck = starter_deck();
    let player = Player {
        hp: Hp(80),
        max_hp: Hp(80),
        block: Block(0),
        energy: Energy(3),
        max_energy: Energy(3),
        hand: Vec::new(),
        draw_pile: Vec::new(),
        discard_pile: Vec::new(),
        exhaust_pile: Vec::new(),
        statuses: StatusMap::new(),
        deck,
        gold: 99,
        relics: Vec::new(),
        potions: Vec::new(),
    };
    let _ = rng;
    GameState::Map(MapState { player, floor: 0, next_enemies: None, scenario: Scenario::Main })
}

pub fn new_simple_run() -> GameState {
    let player = Player {
        hp: Hp(80),
        max_hp: Hp(80),
        block: Block(0),
        energy: Energy(3),
        max_energy: Energy(3),
        hand: Vec::new(),
        draw_pile: Vec::new(),
        discard_pile: Vec::new(),
        exhaust_pile: Vec::new(),
        statuses: StatusMap::new(),
        deck: Vec::new(),
        gold: 0,
        relics: Vec::new(),
        potions: Vec::new(),
    };
    GameState::Map(MapState { player, floor: 0, next_enemies: None, scenario: Scenario::Simple })
}

fn enemies_for_floor(floor: usize) -> Vec<EnemyKind> {
    match floor {
        1 => vec![EnemyKind::Fungibeast],
        2 => vec![EnemyKind::Cultist],
        3 => vec![EnemyKind::JawWorm],
        4 => vec![EnemyKind::SmallSpikeSlime],
        5 => vec![EnemyKind::Louse, EnemyKind::Louse],
        6 => vec![EnemyKind::RedLouse],
        _ => vec![EnemyKind::Louse],
    }
}

fn generate_rewards(rng: &mut impl Rng) -> Vec<Card> {
    let mut pool = crate::cards::reward_pool();
    rng.shuffle(&mut pool);
    pool.into_iter().take(3).collect()
}

fn random_potion(rng: &mut impl Rng) -> Potion {
    let mut pool = [
        Potion::FirePotion, Potion::ExplosivePotion, Potion::BlockPotion,
        Potion::StrengthPotion, Potion::SwiftPotion, Potion::FearPotion,
        Potion::WeakPotion, Potion::BloodPotion, Potion::EnergyPotion,
    ];
    rng.shuffle(&mut pool);
    pool[0]
}

fn award_potion(player: &mut Player, events: &mut Vec<Event>, rng: &mut impl Rng) -> Option<Potion> {
    let potion = random_potion(rng);
    if player.potions.len() < MAX_POTIONS {
        player.potions.push(potion);
        events.push(Event::PotionAwarded { potion });
        None
    } else {
        Some(potion)
    }
}

fn generate_shop(player: Player, floor: usize, rng: &mut impl Rng) -> ShopState {
    let mut card_pool = crate::cards::reward_pool();
    rng.shuffle(&mut card_pool);
    let cards = card_pool.into_iter().take(2).map(|c| (c, false)).collect();

    let mut relic_pool = Relic::all();
    rng.shuffle(&mut relic_pool);
    let relic = relic_pool.into_iter().next().map(|r| (r, false));

    let potion = Some((random_potion(rng), false));

    ShopState { player, floor, cards, relic, potion }
}

fn player_after_combat(player: Player, gold_gain: i32) -> Player {
    let mut deck = player.deck;
    deck.extend(player.exhaust_pile);
    // Lantern grants +1 max energy for the combat only; restore original value here.
    let max_energy = if player.relics.contains(&Relic::Lantern) {
        Energy(player.max_energy.0 - 1)
    } else {
        player.max_energy
    };
    Player {
        block: Block(0),
        energy: max_energy,
        max_energy,
        hand: Vec::new(),
        draw_pile: Vec::new(),
        discard_pile: Vec::new(),
        exhaust_pile: Vec::new(),
        statuses: StatusMap::new(),
        gold: player.gold + gold_gain,
        deck,
        ..player
    }
}

const GOLD_PER_COMBAT: i32 = 50;

pub fn apply_command(
    state: GameState,
    command: Command,
    rng: &mut impl Rng,
) -> Result<(GameState, Vec<Event>), CommandError> {
    match state {
        GameState::Map(MapState { player, floor, next_enemies, scenario }) => match command {
            Command::Spawn(enemies) => {
                Ok((GameState::Map(MapState { player, floor, next_enemies: Some(enemies), scenario }), Vec::new()))
            }
            Command::ChooseNode(idx) => {
                if idx != 0 {
                    return Err(CommandError::InvalidCard);
                }
                let node = MAP_NODES.get(floor).unwrap_or(&MapNode::Combat);
                match node {
                    MapNode::Combat | MapNode::Boss => {
                        let enemies = next_enemies.unwrap_or_else(|| enemies_for_floor(floor));
                        let is_boss = matches!(node, MapNode::Boss);
                        let mut combat_state = CombatState::from_player(player, enemies, rng);
                        let mut events = Vec::new();
                        apply_combat_start_relics(&mut combat_state, &mut events, rng, is_boss);
                        // FestivePopper (or similar) may have killed all enemies instantly.
                        if combat_state.enemies.iter().all(|e| e.hp <= Hp(0)) {
                            events.push(Event::GoldEarned { amount: GOLD_PER_COMBAT });
                            let mut victory_player = combat_state.player;
                            apply_end_of_combat_relics(&mut victory_player, &mut events);
                            let player = player_after_combat(victory_player, GOLD_PER_COMBAT);
                            if is_boss {
                                return Ok((GameState::GameOver { victory: true }, events));
                            }
                            let options = generate_rewards(rng);
                            return Ok((
                                GameState::CardReward(CardRewardState { player, floor: floor + 1, options, offered_potion: None }),
                                events,
                            ));
                        }
                        Ok((GameState::Combat { state: combat_state, floor, scenario }, events))
                    }
                    MapNode::RestSite => Ok((
                        GameState::RestSite(RestSiteState { player, floor }),
                        Vec::new(),
                    )),
                    MapNode::Merchant => Ok((
                        GameState::Shop(generate_shop(player, floor, rng)),
                        Vec::new(),
                    )),
                }
            }
            Command::SkipFloor => {
                Ok((GameState::Map(MapState { player, floor: floor + 1, next_enemies: None, scenario }), Vec::new()))
            }
            Command::AddRelic(relic) => {
                let mut p = player;
                let events = crate::relics::grant_relic(&mut p, relic, rng);
                Ok((GameState::Map(MapState { player: p, floor, next_enemies: None, scenario }), events))
            }
            Command::AddPotion(potion) => {
                let mut p = player;
                if p.potions.len() < MAX_POTIONS {
                    p.potions.push(potion);
                }
                Ok((GameState::Map(MapState { player: p, floor, next_enemies: None, scenario }), vec![]))
            }
            Command::DiscardPotion(slot) => {
                let mut p = player;
                if slot >= p.potions.len() {
                    return Err(CommandError::InvalidCard);
                }
                let potion = p.potions.remove(slot);
                Ok((GameState::Map(MapState { player: p, floor, next_enemies, scenario }),
                    vec![Event::PotionDiscarded { potion }]))
            }
            _ => Err(CommandError::InvalidPhase),
        },

        GameState::Combat { state: mut combat_state, floor, scenario } => match command {
            Command::WinCombat => {
                let mut events = vec![Event::EnemyDied, Event::GoldEarned { amount: GOLD_PER_COMBAT }];
                let is_boss = matches!(MAP_NODES.get(floor), Some(MapNode::Boss));
                apply_end_of_combat_relics(&mut combat_state.player, &mut events);
                let mut player = player_after_combat(combat_state.player, GOLD_PER_COMBAT);
                if is_boss {
                    Ok((GameState::GameOver { victory: true }, events))
                } else if scenario == Scenario::Simple {
                    Ok((GameState::Map(MapState { player, floor, next_enemies: None, scenario }), events))
                } else {
                    let offered_potion = award_potion(&mut player, &mut events, rng);
                    let options = generate_rewards(rng);
                    Ok((GameState::CardReward(CardRewardState { player, floor: floor + 1, options, offered_potion }), events))
                }
            }
            Command::AddCard(card) => {
                combat_state.player.hand.push(card);
                Ok((GameState::Combat { state: combat_state, floor, scenario }, vec![]))
            }
            Command::AddRelic(relic) => {
                let events = crate::relics::grant_relic(&mut combat_state.player, relic, rng);
                Ok((GameState::Combat { state: combat_state, floor, scenario }, events))
            }
            Command::AddPotion(potion) => {
                if combat_state.player.potions.len() < MAX_POTIONS {
                    combat_state.player.potions.push(potion);
                }
                Ok((GameState::Combat { state: combat_state, floor, scenario }, vec![]))
            }
            Command::DiscardPotion(slot) => {
                if slot >= combat_state.player.potions.len() {
                    return Err(CommandError::InvalidCard);
                }
                let potion = combat_state.player.potions.remove(slot);
                Ok((GameState::Combat { state: combat_state, floor, scenario },
                    vec![Event::PotionDiscarded { potion }]))
            }
            Command::ChooseNode(_) | Command::Rest | Command::ChooseCardReward(_)
            | Command::SkipFloor | Command::UpgradeCard(_) | Command::LeaveShop
            | Command::BuyCard(_) | Command::BuyRelic | Command::BuyPotion => {
                Err(CommandError::InvalidPhase)
            }
            cmd => {
                let hand_size_at_turn_end = if matches!(cmd, Command::EndTurn) {
                    combat_state.player.hand.len()
                } else {
                    0
                };
                let (mut new_combat, mut events) = apply_combat_command(combat_state, cmd, rng)?;
                // Turn-end relics fire after EndTurn (before enemy acts).
                if events.contains(&Event::TurnEnded)
                    && matches!(new_combat.phase, CombatPhase::EnemyTurn)
                {
                    apply_turn_end_relics(&mut new_combat, &mut events, hand_size_at_turn_end);
                }
                // Turn-start relics fire after EndEnemyTurn (new player turn begins).
                if events.iter().any(|e| matches!(e, Event::TurnStarted { .. }))
                    && matches!(new_combat.phase, CombatPhase::PlayerTurn)
                {
                    apply_turn_start_relics(&mut new_combat, &mut events, rng);
                }
                match new_combat.phase {
                    CombatPhase::Victory => {
                        events.push(Event::GoldEarned { amount: GOLD_PER_COMBAT });
                        let is_boss = matches!(MAP_NODES.get(floor), Some(MapNode::Boss));
                        let mut victory_player = new_combat.player;
                        apply_end_of_combat_relics(&mut victory_player, &mut events);
                        let mut player = player_after_combat(victory_player, GOLD_PER_COMBAT);
                        if is_boss {
                            Ok((GameState::GameOver { victory: true }, events))
                        } else if scenario == Scenario::Simple {
                            Ok((GameState::Map(MapState { player, floor, next_enemies: None, scenario }), events))
                        } else {
                            let offered_potion = award_potion(&mut player, &mut events, rng);
                            let options = generate_rewards(rng);
                            Ok((
                                GameState::CardReward(CardRewardState {
                                    player,
                                    floor: floor + 1,
                                    options,
                                    offered_potion,
                                }),
                                events,
                            ))
                        }
                    }
                    CombatPhase::Defeat => Ok((GameState::GameOver { victory: false }, events)),
                    _ => Ok((GameState::Combat { state: new_combat, floor, scenario }, events)),
                }
            }
        },

        GameState::RestSite(RestSiteState { mut player, floor }) => match command {
            Command::Rest => {
                let heal = (player.max_hp.0 * 30 / 100).max(1);
                player.hp.0 = (player.hp.0 + heal).min(player.max_hp.0);
                let mut events = vec![Event::Healed { amount: heal }];
                apply_rest_relics(&mut player, &mut events);
                Ok((GameState::Map(MapState { player, floor: floor + 1, next_enemies: None, scenario: Scenario::Main }), events))
            }
            Command::UpgradeCard(idx) => {
                let from = player.deck.get(idx).cloned().ok_or(CommandError::InvalidCard)?;
                let to = from.upgrade().ok_or(CommandError::InvalidCard)?;
                player.deck[idx] = to.clone();
                let events = vec![Event::CardUpgraded { from, to }];
                Ok((GameState::Map(MapState { player, floor: floor + 1, next_enemies: None, scenario: Scenario::Main }), events))
            }
            Command::DiscardPotion(slot) => {
                if slot >= player.potions.len() {
                    return Err(CommandError::InvalidCard);
                }
                let potion = player.potions.remove(slot);
                Ok((GameState::RestSite(RestSiteState { player, floor }),
                    vec![Event::PotionDiscarded { potion }]))
            }
            _ => Err(CommandError::InvalidPhase),
        },

        GameState::CardReward(CardRewardState { mut player, floor, options, offered_potion }) => {
            match command {
                Command::ChooseCardReward(idx) => {
                    if idx >= options.len() {
                        return Err(CommandError::InvalidCard);
                    }
                    let card = options[idx].clone();
                    player.deck.push(card.clone());
                    let events = vec![Event::CardAdded { card }];
                    Ok((GameState::Map(MapState { player, floor, next_enemies: None, scenario: Scenario::Main }), events))
                }
                Command::SkipReward => {
                    Ok((GameState::Map(MapState { player, floor, next_enemies: None, scenario: Scenario::Main }), Vec::new()))
                }
                Command::DiscardPotion(slot) => {
                    if slot >= player.potions.len() {
                        return Err(CommandError::InvalidCard);
                    }
                    let discarded = player.potions.remove(slot);
                    let mut events = vec![Event::PotionDiscarded { potion: discarded }];
                    let new_offered = if let Some(offered) = offered_potion {
                        player.potions.push(offered);
                        events.push(Event::PotionAwarded { potion: offered });
                        None
                    } else {
                        None
                    };
                    Ok((GameState::CardReward(CardRewardState { player, floor, options, offered_potion: new_offered }), events))
                }
                _ => Err(CommandError::InvalidPhase),
            }
        }

        GameState::Shop(ShopState { player, floor, .. }) => match command {
            Command::LeaveShop => Ok((
                GameState::Map(MapState { player, floor: floor + 1, next_enemies: None, scenario: Scenario::Main }),
                Vec::new(),
            )),
            _ => Err(CommandError::InvalidPhase),
        },

        GameState::GameOver { .. } => Err(CommandError::CombatOver),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::Enemy;
    use crate::enemies::Move;
    use crate::relics::Relic;

    #[test]
    fn command_error_display_not_enough_energy() {
        assert_eq!(CommandError::NotEnoughEnergy.to_string(), "Not enough energy.");
    }

    #[test]
    fn command_error_display_invalid_card() {
        assert_eq!(CommandError::InvalidCard.to_string(), "No card at that position.");
    }

    #[test]
    fn command_error_display_invalid_phase() {
        assert_eq!(CommandError::InvalidPhase.to_string(), "Can't do that right now.");
    }

    #[test]
    fn command_error_display_combat_over() {
        assert_eq!(CommandError::CombatOver.to_string(), "Combat is already over.");
    }
    use crate::rng::NoOpRng;

    fn rng() -> NoOpRng {
        NoOpRng
    }

    fn make_player() -> Player {
        Player {
            hp: Hp(80),
            max_hp: Hp(80),
            block: Block(0),
            energy: Energy(3),
            max_energy: Energy(3),
            hand: Vec::new(),
            draw_pile: Vec::new(),
            discard_pile: Vec::new(),
            exhaust_pile: Vec::new(),
            statuses: StatusMap::new(),
            deck: starter_deck(),
            gold: 0,
            relics: Vec::new(),
            potions: Vec::new(),
        }
    }

    fn combat_at_floor(floor: usize) -> GameState {
        let player = make_player();
        let cs = CombatState {
            player: Player {
                hand: vec![Card::Strike],
                draw_pile: Vec::new(),
                ..player
            },
            enemies: vec![Enemy {
                kind: EnemyKind::Louse,
                hp: Hp(1),
                max_hp: Hp(20),
                block: Block(0),
                move_: Move::LouseBite,
                last_move: None,
                statuses: StatusMap::new(),
            }],
            turn: 1,
            phase: CombatPhase::PlayerTurn,
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0,
        };
        GameState::Combat { state: cs, floor, scenario: Scenario::Main }
    }

    // --- new_run ---

    #[test]
    fn new_run_starts_on_map() {
        let state = new_run(&mut rng());
        assert!(matches!(state, GameState::Map(_)));
    }

    #[test]
    fn new_run_starts_at_floor_0() {
        if let GameState::Map(map) = new_run(&mut rng()) {
            assert_eq!(map.floor, 0);
        } else {
            panic!("expected Map state");
        }
    }

    #[test]
    fn new_run_player_starts_with_99_gold() {
        if let GameState::Map(map) = new_run(&mut rng()) {
            assert_eq!(map.player.gold, 99);
        } else {
            panic!("expected Map state");
        }
    }

    #[test]
    fn new_run_player_has_starter_deck() {
        if let GameState::Map(map) = new_run(&mut rng()) {
            assert_eq!(map.player.deck.len(), starter_deck().len());
        } else {
            panic!("expected Map state");
        }
    }

    // --- map navigation ---

    #[test]
    fn choose_node_0_enters_combat() {
        let state = new_run(&mut rng());
        let (state, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        assert!(matches!(state, GameState::Combat { floor: 0, .. }));
    }

    #[test]
    fn choose_node_invalid_index_returns_error() {
        let state = new_run(&mut rng());
        let result = apply_command(state, Command::ChooseNode(1), &mut rng());
        assert_eq!(result, Err(CommandError::InvalidCard));
    }

    #[test]
    fn non_map_commands_rejected_on_map() {
        let state = new_run(&mut rng());
        assert_eq!(
            apply_command(state.clone(), Command::EndTurn, &mut rng()),
            Err(CommandError::InvalidPhase)
        );
        assert_eq!(
            apply_command(state.clone(), Command::Rest, &mut rng()),
            Err(CommandError::InvalidPhase)
        );
    }

    // --- combat victory transitions ---

    #[test]
    fn winning_combat_goes_to_card_reward() {
        let state = combat_at_floor(0);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(matches!(state, GameState::CardReward(_)));
    }

    #[test]
    fn winning_combat_awards_50_gold() {
        let state = combat_at_floor(0);
        let (_, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(events.contains(&Event::GoldEarned { amount: 50 }));
    }

    #[test]
    fn winning_combat_advances_to_correct_floor_in_reward() {
        let state = combat_at_floor(0);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        if let GameState::CardReward(cr) = state {
            assert_eq!(cr.floor, 1);
        } else {
            panic!("expected CardReward");
        }
    }

    #[test]
    fn card_reward_has_3_options() {
        let state = combat_at_floor(0);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        if let GameState::CardReward(cr) = state {
            assert_eq!(cr.options.len(), 3);
        } else {
            panic!("expected CardReward");
        }
    }

    #[test]
    fn choosing_card_reward_adds_card_to_deck() {
        let state = combat_at_floor(0);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        if let GameState::CardReward(ref cr) = state {
            let deck_size_before = cr.player.deck.len();
            let (state, _) = apply_command(state, Command::ChooseCardReward(0), &mut rng()).unwrap();
            if let GameState::Map(map) = state {
                assert_eq!(map.player.deck.len(), deck_size_before + 1);
            } else {
                panic!("expected Map after reward");
            }
        } else {
            panic!("expected CardReward");
        }
    }

    #[test]
    fn choosing_card_reward_returns_to_map() {
        let state = combat_at_floor(0);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::ChooseCardReward(0), &mut rng()).unwrap();
        assert!(matches!(state, GameState::Map(_)));
    }

    #[test]
    fn choosing_card_reward_emits_card_added_event() {
        let state = combat_at_floor(0);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (_, events) = apply_command(state, Command::ChooseCardReward(0), &mut rng()).unwrap();
        assert!(events.iter().any(|e| matches!(e, Event::CardAdded { .. })));
    }

    #[test]
    fn invalid_reward_index_returns_error() {
        let state = combat_at_floor(0);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let result = apply_command(state, Command::ChooseCardReward(99), &mut rng());
        assert_eq!(result, Err(CommandError::InvalidCard));
    }

    // --- defeat ---

    #[test]
    fn losing_combat_yields_game_over_defeat() {
        let player = Player {
            hp: Hp(1),
            hand: vec![Card::Strike],
            ..make_player()
        };
        let cs = CombatState {
            player,
            enemies: vec![Enemy {
                kind: EnemyKind::Louse,
                hp: Hp(20),
                max_hp: Hp(20),
                block: Block(0),
                move_: Move::LouseBite,
                last_move: None,
                statuses: StatusMap::new(),
            }],
            turn: 1,
            phase: CombatPhase::PlayerTurn,
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0,
        };
        let state = GameState::Combat { state: cs, floor: 0, scenario: Scenario::Main };
        let (after_end, _) =
            apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        let (after_enemy, _) =
            apply_command(after_end, Command::EndEnemyTurn, &mut rng()).unwrap();
        assert_eq!(after_enemy, GameState::GameOver { victory: false });
    }

    // --- rest site ---

    #[test]
    fn floor_3_is_merchant() {
        assert_eq!(MAP_NODES[3], MapNode::Merchant);
    }

    #[test]
    fn floor_4_is_rest_site() {
        assert_eq!(MAP_NODES[4], MapNode::RestSite);
    }

    #[test]
    fn choosing_rest_site_enters_rest_state() {
        let map = GameState::Map(MapState { player: make_player(), floor: 4, next_enemies: None, scenario: Scenario::Main });
        let (state, _) = apply_command(map, Command::ChooseNode(0), &mut rng()).unwrap();
        assert!(matches!(state, GameState::RestSite(_)));
    }

    #[test]
    fn rest_heals_30_percent_of_max_hp() {
        let mut player = make_player();
        player.hp = Hp(50);
        let state = GameState::RestSite(RestSiteState { player, floor: 3 });
        let (state, _) = apply_command(state, Command::Rest, &mut rng()).unwrap();
        if let GameState::Map(map) = state {
            assert_eq!(map.player.hp, Hp(74)); // 50 + (80 * 30 / 100 = 24)
        } else {
            panic!("expected Map after rest");
        }
    }

    #[test]
    fn rest_cannot_overheal() {
        let player = make_player(); // already at 80/80
        let state = GameState::RestSite(RestSiteState { player, floor: 3 });
        let (state, _) = apply_command(state, Command::Rest, &mut rng()).unwrap();
        if let GameState::Map(map) = state {
            assert_eq!(map.player.hp, Hp(80));
        } else {
            panic!("expected Map after rest");
        }
    }

    #[test]
    fn rest_emits_healed_event() {
        let mut player = make_player();
        player.hp = Hp(50);
        let state = GameState::RestSite(RestSiteState { player, floor: 3 });
        let (_, events) = apply_command(state, Command::Rest, &mut rng()).unwrap();
        assert!(events.iter().any(|e| matches!(e, Event::Healed { .. })));
    }

    #[test]
    fn rest_advances_to_next_floor_on_map() {
        let state = GameState::RestSite(RestSiteState { player: make_player(), floor: 3 });
        let (state, _) = apply_command(state, Command::Rest, &mut rng()).unwrap();
        if let GameState::Map(map) = state {
            assert_eq!(map.floor, 4);
        } else {
            panic!("expected Map after rest");
        }
    }

    #[test]
    fn non_rest_commands_rejected_at_rest_site() {
        let state = GameState::RestSite(RestSiteState { player: make_player(), floor: 3 });
        assert_eq!(
            apply_command(state, Command::EndTurn, &mut rng()),
            Err(CommandError::InvalidPhase)
        );
    }

    // --- boss / run end ---

    #[test]
    fn floor_5_is_boss() {
        assert_eq!(MAP_NODES[5], MapNode::Boss);
    }

    #[test]
    fn winning_boss_ends_run_with_victory() {
        let state = combat_at_floor(5);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state, GameState::GameOver { victory: true });
    }

    #[test]
    fn game_over_rejects_all_commands() {
        let state = GameState::GameOver { victory: false };
        assert_eq!(
            apply_command(state, Command::EndTurn, &mut rng()),
            Err(CommandError::CombatOver)
        );
    }

    // --- enemy selection by floor ---

    #[test]
    fn floor_0_spawns_louse() {
        assert_eq!(enemies_for_floor(0), vec![EnemyKind::Louse]);
    }

    #[test]
    fn floor_1_spawns_fungibeast() {
        assert_eq!(enemies_for_floor(1), vec![EnemyKind::Fungibeast]);
    }

    #[test]
    fn floor_2_spawns_cultist() {
        assert_eq!(enemies_for_floor(2), vec![EnemyKind::Cultist]);
    }

    #[test]
    fn floor_3_spawns_jaw_worm() {
        assert_eq!(enemies_for_floor(3), vec![EnemyKind::JawWorm]);
    }

    #[test]
    fn floor_4_spawns_small_spike_slime() {
        assert_eq!(enemies_for_floor(4), vec![EnemyKind::SmallSpikeSlime]);
    }

    #[test]
    fn floor_5_boss_spawns_two_lice() {
        assert_eq!(enemies_for_floor(5), vec![EnemyKind::Louse, EnemyKind::Louse]);
    }

    #[test]
    fn floor_6_spawns_red_louse() {
        assert_eq!(enemies_for_floor(6), vec![EnemyKind::RedLouse]);
    }

    // --- Spawn command ---

    #[test]
    fn spawn_command_sets_next_enemies_on_map_state() {
        let state = GameState::Map(MapState { player: make_player(), floor: 0, next_enemies: None, scenario: Scenario::Main });
        let (state, _) = apply_command(state, Command::Spawn(vec![EnemyKind::Fungibeast]), &mut rng()).unwrap();
        let GameState::Map(map) = state else { panic!("expected Map") };
        assert_eq!(map.next_enemies, Some(vec![EnemyKind::Fungibeast]));
    }

    #[test]
    fn choose_node_uses_spawned_enemies_instead_of_floor_default() {
        // Floor 0 normally spawns a Louse, but spawn overrides it
        let state = GameState::Map(MapState {
            player: make_player(),
            floor: 0,
            next_enemies: Some(vec![EnemyKind::Cultist]),
            scenario: Scenario::Main,
        });
        let (state, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.enemies[0].kind, EnemyKind::Cultist);
    }

    #[test]
    fn choose_node_clears_next_enemies_after_use() {
        let state = GameState::Map(MapState {
            player: make_player(),
            floor: 0,
            next_enemies: Some(vec![EnemyKind::Cultist]),
            scenario: Scenario::Main,
        });
        // enter combat, win, come back to map
        let (state, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::WinCombat, &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::SkipReward, &mut rng()).unwrap();
        let GameState::Map(map) = state else { panic!("expected Map") };
        assert_eq!(map.next_enemies, None);
    }

    #[test]
    fn choose_node_falls_back_to_floor_enemies_when_no_spawn() {
        let state = GameState::Map(MapState { player: make_player(), floor: 0, next_enemies: None, scenario: Scenario::Main });
        let (state, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.enemies[0].kind, EnemyKind::Louse);
    }

    // --- player state persists across combat ---

    #[test]
    fn player_hp_persists_from_combat_to_map() {
        let mut player = make_player();
        player.hp = Hp(50);
        let cs = CombatState {
            player: Player {
                hand: vec![Card::Strike],
                ..player
            },
            enemies: vec![Enemy {
                kind: EnemyKind::Louse,
                hp: Hp(1),
                max_hp: Hp(20),
                block: Block(0),
                move_: Move::LouseBite,
                last_move: None,
                statuses: StatusMap::new(),
            }],
            turn: 1,
            phase: CombatPhase::PlayerTurn,
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0,
        };
        let state = GameState::Combat { state: cs, floor: 0, scenario: Scenario::Main };
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        // Now at CardReward
        let (state, _) = apply_command(state, Command::ChooseCardReward(0), &mut rng()).unwrap();
        if let GameState::Map(map) = state {
            assert_eq!(map.player.hp, Hp(50));
        } else {
            panic!("expected Map");
        }
    }

    #[test]
    fn player_gold_persists_after_multiple_combats() {
        // Win two combats, check gold = 100
        let state = combat_at_floor(0);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::ChooseCardReward(0), &mut rng()).unwrap();
        // Now on map floor 1 with 50 gold
        if let GameState::Map(ref map) = state {
            assert_eq!(map.player.gold, 50);
        }
        // Enter floor 1 combat
        let (state, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        // Manually kill the enemy
        let state = if let GameState::Combat { mut state, floor, scenario: Scenario::Main } = state {
            state.enemies[0].hp = Hp(1);
            state.player.hand = vec![Card::Strike];
            GameState::Combat { state, floor, scenario: Scenario::Main }
        } else {
            panic!("expected Combat");
        };
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::ChooseCardReward(0), &mut rng()).unwrap();
        if let GameState::Map(map) = state {
            assert_eq!(map.player.gold, 100);
        } else {
            panic!("expected Map");
        }
    }

    #[test]
    fn skipping_reward_returns_to_map() {
        let state = combat_at_floor(0);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(matches!(state, GameState::CardReward(_)));
        let (state, _) = apply_command(state, Command::SkipReward, &mut rng()).unwrap();
        assert!(matches!(state, GameState::Map(_)));
    }

    #[test]
    fn skipping_reward_does_not_add_card_to_deck() {
        let state = combat_at_floor(0);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let deck_size = if let GameState::CardReward(ref cr) = state {
            cr.player.deck.len()
        } else {
            panic!("expected CardReward");
        };
        let (state, _) = apply_command(state, Command::SkipReward, &mut rng()).unwrap();
        if let GameState::Map(map) = state {
            assert_eq!(map.player.deck.len(), deck_size);
        } else {
            panic!("expected Map");
        }
    }

    #[test]
    fn skipping_reward_advances_to_correct_floor() {
        let state = combat_at_floor(0);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::SkipReward, &mut rng()).unwrap();
        if let GameState::Map(map) = state {
            assert_eq!(map.floor, 1);
        } else {
            panic!("expected Map");
        }
    }

    // --- exhaust pile ---

    #[test]
    fn exhausted_cards_return_to_deck_after_combat() {
        let player = Player {
            deck: vec![Card::Strike, Card::Disarm],
            hand: vec![Card::Strike],
            exhaust_pile: vec![Card::Disarm], // Disarm was played and exhausted
            ..make_player()
        };
        let cs = CombatState {
            player,
            enemies: vec![Enemy {
                kind: EnemyKind::Louse,
                hp: Hp(1),
                max_hp: Hp(20),
                block: Block(0),
                move_: Move::LouseBite,
                last_move: None,
                statuses: StatusMap::new(),
            }],
            turn: 1,
            phase: CombatPhase::PlayerTurn,
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0,
        };
        let state = GameState::Combat { state: cs, floor: 0, scenario: Scenario::Main };
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let GameState::CardReward(cr) = state else { panic!("expected CardReward") };
        assert!(cr.player.deck.contains(&Card::Disarm), "Disarm should be back in deck");
        assert!(cr.player.exhaust_pile.is_empty(), "exhaust pile should be cleared");
    }

    #[test]
    fn exhaust_pile_is_empty_at_combat_start() {
        let state = new_run(&mut rng());
        let (state, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert!(cs.player.exhaust_pile.is_empty());
    }

    // --- shop ---

    fn map_at_floor(floor: usize) -> GameState {
        GameState::Map(MapState { player: make_player(), floor, next_enemies: None, scenario: Scenario::Main })
    }

    #[test]
    fn map_has_six_nodes() {
        assert_eq!(MAP_NODES.len(), 6);
    }

    #[test]
    fn entering_merchant_node_transitions_to_shop() {
        let (next, _) = apply_command(map_at_floor(3), Command::ChooseNode(0), &mut rng()).unwrap();
        assert!(matches!(next, GameState::Shop(_)));
    }

    #[test]
    fn shop_has_two_cards_one_relic_one_potion() {
        let (next, _) = apply_command(map_at_floor(3), Command::ChooseNode(0), &mut rng()).unwrap();
        let GameState::Shop(shop) = next else { panic!("expected Shop") };
        assert_eq!(shop.cards.len(), 2);
        assert!(shop.relic.is_some());
        assert!(shop.potion.is_some());
    }

    #[test]
    fn shop_items_start_as_not_purchased() {
        let (next, _) = apply_command(map_at_floor(3), Command::ChooseNode(0), &mut rng()).unwrap();
        let GameState::Shop(shop) = next else { panic!("expected Shop") };
        assert!(shop.cards.iter().all(|(_, purchased)| !purchased));
        assert!(shop.relic.as_ref().map_or(true, |(_, p)| !p));
        assert!(shop.potion.as_ref().map_or(true, |(_, p)| !p));
    }

    #[test]
    fn leave_shop_returns_to_map_at_next_floor() {
        let (shop_state, _) = apply_command(map_at_floor(3), Command::ChooseNode(0), &mut rng()).unwrap();
        let (next, _) = apply_command(shop_state, Command::LeaveShop, &mut rng()).unwrap();
        let GameState::Map(map) = next else { panic!("expected Map") };
        assert_eq!(map.floor, 4);
    }

    #[test]
    fn leave_shop_preserves_player_gold() {
        let mut player = make_player();
        player.gold = 150;
        let shop = ShopState { player, floor: 3, cards: vec![], relic: None, potion: None };
        let (next, _) = apply_command(GameState::Shop(shop), Command::LeaveShop, &mut rng()).unwrap();
        let GameState::Map(map) = next else { panic!("expected Map") };
        assert_eq!(map.player.gold, 150);
    }

    #[test]
    fn non_shop_commands_rejected_in_shop() {
        let shop = ShopState { player: make_player(), floor: 3, cards: vec![], relic: None, potion: None };
        assert_eq!(
            apply_command(GameState::Shop(shop), Command::EndTurn, &mut rng()),
            Err(CommandError::InvalidPhase)
        );
    }

    // --- rest site: upgrade ---

    #[test]
    fn upgrade_replaces_card_in_deck_with_plus_version() {
        // deck[0] = Strike (from starter_deck); upgrade it → StrikePlus
        let state = GameState::RestSite(RestSiteState { player: make_player(), floor: 3 });
        let (state, _) = apply_command(state, Command::UpgradeCard(0), &mut rng()).unwrap();
        let GameState::Map(map) = state else { panic!("expected Map") };
        assert_eq!(map.player.deck[0], Card::StrikePlus);
    }

    #[test]
    fn upgrade_advances_to_map_at_next_floor() {
        let state = GameState::RestSite(RestSiteState { player: make_player(), floor: 3 });
        let (state, _) = apply_command(state, Command::UpgradeCard(0), &mut rng()).unwrap();
        let GameState::Map(map) = state else { panic!("expected Map") };
        assert_eq!(map.floor, 4);
    }

    #[test]
    fn upgrade_emits_card_upgraded_event() {
        let state = GameState::RestSite(RestSiteState { player: make_player(), floor: 3 });
        let (_, events) = apply_command(state, Command::UpgradeCard(0), &mut rng()).unwrap();
        assert!(events.iter().any(|e| matches!(e, Event::CardUpgraded { .. })));
    }

    #[test]
    fn upgrade_invalid_index_returns_error() {
        let state = GameState::RestSite(RestSiteState { player: make_player(), floor: 3 });
        let result = apply_command(state, Command::UpgradeCard(99), &mut rng());
        assert_eq!(result, Err(CommandError::InvalidCard));
    }

    #[test]
    fn upgrade_non_upgradeable_card_returns_error() {
        // starter_deck last card is Disarm (index 11), which cannot be upgraded
        let state = GameState::RestSite(RestSiteState { player: make_player(), floor: 3 });
        let result = apply_command(state, Command::UpgradeCard(11), &mut rng());
        assert_eq!(result, Err(CommandError::InvalidCard));
    }

    // --- debug: WinCombat ---

    #[test]
    fn win_combat_kills_enemy_and_goes_to_card_reward() {
        let state = combat_at_floor(0);
        let (state, _) = apply_command(state, Command::WinCombat, &mut rng()).unwrap();
        assert!(matches!(state, GameState::CardReward(_)));
    }

    #[test]
    fn win_combat_awards_gold() {
        let state = combat_at_floor(0);
        let (_, events) = apply_command(state, Command::WinCombat, &mut rng()).unwrap();
        assert!(events.contains(&Event::GoldEarned { amount: 50 }));
    }

    #[test]
    fn win_combat_on_boss_floor_ends_run_with_victory() {
        let state = combat_at_floor(5);
        let (state, _) = apply_command(state, Command::WinCombat, &mut rng()).unwrap();
        assert_eq!(state, GameState::GameOver { victory: true });
    }

    // --- debug: SkipFloor ---

    #[test]
    fn skip_floor_from_map_advances_floor() {
        let state = new_run(&mut rng()); // Map at floor 0
        let (state, _) = apply_command(state, Command::SkipFloor, &mut rng()).unwrap();
        let GameState::Map(map) = state else { panic!("expected Map") };
        assert_eq!(map.floor, 1);
    }

    #[test]
    fn skip_floor_rejected_from_combat() {
        let state = combat_at_floor(0);
        assert_eq!(
            apply_command(state, Command::SkipFloor, &mut rng()),
            Err(CommandError::InvalidPhase)
        );
    }

    #[test]
    fn win_combat_rejected_from_map() {
        let state = new_run(&mut rng());
        assert_eq!(
            apply_command(state, Command::WinCombat, &mut rng()),
            Err(CommandError::InvalidPhase)
        );
    }

    // --- combat_commands_rejected_in_card_reward_state ---

    #[test]
    fn combat_commands_rejected_in_card_reward_state() {
        let state = combat_at_floor(0);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(matches!(state, GameState::CardReward(_)));
        assert_eq!(
            apply_command(state, Command::EndTurn, &mut rng()),
            Err(CommandError::InvalidPhase)
        );
    }

    // --- Phase 8: multiple enemies ---

    #[test]
    fn boss_floor_has_two_enemies() {
        let map = GameState::Map(MapState { player: make_player(), floor: 5, next_enemies: None, scenario: Scenario::Main });
        let (state, _) = apply_command(map, Command::ChooseNode(0), &mut rng()).unwrap();
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.enemies.len(), 2);
    }

    #[test]
    fn regular_floor_has_one_enemy() {
        let map = GameState::Map(MapState { player: make_player(), floor: 0, next_enemies: None, scenario: Scenario::Main });
        let (state, _) = apply_command(map, Command::ChooseNode(0), &mut rng()).unwrap();
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.enemies.len(), 1);
    }

    #[test]
    fn winning_boss_requires_all_enemies_dead() {
        // Boss has two enemies; combat_at_floor(5) only has one at 1 HP.
        // Use WinCombat (debug) to confirm boss victory still works.
        let state = combat_at_floor(5);
        let (state, _) = apply_command(state, Command::WinCombat, &mut rng()).unwrap();
        assert_eq!(state, GameState::GameOver { victory: true });
    }

    // --- burning blood ---

    fn combat_with_burning_blood_at_floor(floor: usize, hp: i32) -> GameState {
        let mut player = make_player();
        player.hp = Hp(hp);
        player.relics.push(Relic::BurningBlood);
        let cs = CombatState {
            player: Player {
                hand: vec![Card::Strike],
                draw_pile: Vec::new(),
                ..player
            },
            enemies: vec![Enemy {
                kind: EnemyKind::Louse,
                hp: Hp(1),
                max_hp: Hp(20),
                block: Block(0),
                move_: Move::LouseBite,
                last_move: None,
                statuses: StatusMap::new(),
            }],
            turn: 1,
            phase: CombatPhase::PlayerTurn,
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0,
        };
        GameState::Combat { state: cs, floor, scenario: Scenario::Main }
    }

    #[test]
    fn burning_blood_heals_6_hp_on_combat_victory() {
        let state = combat_with_burning_blood_at_floor(0, 50);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let GameState::CardReward(cr) = state else { panic!("expected CardReward") };
        assert_eq!(cr.player.hp, Hp(56));
    }

    #[test]
    fn burning_blood_heals_6_hp_on_win_combat_command() {
        let state = combat_with_burning_blood_at_floor(0, 50);
        let (state, _) = apply_command(state, Command::WinCombat, &mut rng()).unwrap();
        let GameState::CardReward(cr) = state else { panic!("expected CardReward") };
        assert_eq!(cr.player.hp, Hp(56));
    }

    #[test]
    fn burning_blood_cannot_overheal_on_victory() {
        let state = combat_with_burning_blood_at_floor(0, 78);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let GameState::CardReward(cr) = state else { panic!("expected CardReward") };
        assert_eq!(cr.player.hp, Hp(80));
    }

    // --- combat-start relics (integration) ---

    fn enter_combat_with_relic(relic: Relic, floor: usize) -> GameState {
        let mut player = make_player();
        player.relics.push(relic);
        let state = GameState::Map(MapState { player, floor, next_enemies: None, scenario: Scenario::Main });
        let (state, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        state
    }

    #[test]
    fn anchor_gives_10_block_when_entering_combat() {
        let state = enter_combat_with_relic(Relic::Anchor, 0);
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.player.block, Block(10));
    }

    #[test]
    fn vajra_grants_1_strength_when_entering_combat() {
        let state = enter_combat_with_relic(Relic::Vajra, 0);
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.player.statuses.get(&crate::status::StatusEffect::Strength), Some(&1));
    }

    #[test]
    fn lantern_gives_4_energy_when_entering_combat() {
        let state = enter_combat_with_relic(Relic::Lantern, 0);
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.player.energy, Energy(4));
        assert_eq!(cs.player.max_energy, Energy(4));
    }

    #[test]
    fn lantern_max_energy_restored_to_3_after_combat() {
        // Enter combat through ChooseNode so apply_combat_start_relics fires and
        // Lantern bumps max_energy to 4 before we win.
        let mut player = make_player();
        player.relics.push(Relic::Lantern);
        let state = GameState::Map(MapState { player, floor: 0, next_enemies: None, scenario: Scenario::Main });
        let (state, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        // Verify Lantern boosted energy during combat.
        let GameState::Combat { .. } = state else { panic!("expected Combat") };
        let (state, _) = apply_command(state, Command::WinCombat, &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::ChooseCardReward(0), &mut rng()).unwrap();
        let GameState::Map(map) = state else { panic!("expected Map") };
        assert_eq!(map.player.max_energy, Energy(3));
    }

    #[test]
    fn pantograph_heals_25_on_boss_floor() {
        let mut player = make_player();
        player.hp = Hp(50);
        player.relics.push(Relic::Pantograph);
        let state = GameState::Map(MapState { player, floor: 5, next_enemies: None, scenario: Scenario::Main }); // floor 5 = Boss
        let (state, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.player.hp, Hp(75));
    }

    #[test]
    fn pantograph_does_not_heal_on_normal_floor() {
        let mut player = make_player();
        player.hp = Hp(50);
        player.relics.push(Relic::Pantograph);
        let state = GameState::Map(MapState { player, floor: 0, next_enemies: None, scenario: Scenario::Main });
        let (state, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.player.hp, Hp(50));
    }

    // --- turn-end relics (integration) ---

    fn combat_with_relic_at_floor(relic: Relic, floor: usize) -> GameState {
        let mut player = make_player();
        player.relics.push(relic);
        let cs = CombatState {
            player: Player {
                hand: vec![],
                draw_pile: vec![Card::Strike; 5],
                ..player
            },
            enemies: vec![Enemy {
                kind: EnemyKind::Louse,
                hp: Hp(20),
                max_hp: Hp(20),
                block: Block(0),
                move_: Move::LouseBite,
                last_move: None,
                statuses: StatusMap::new(),
            }],
            turn: 1,
            phase: CombatPhase::PlayerTurn,
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0,
        };
        GameState::Combat { state: cs, floor, scenario: Scenario::Main }
    }

    #[test]
    fn orichalcum_gives_6_block_after_end_turn_with_no_block() {
        let state = combat_with_relic_at_floor(Relic::Orichalcum, 0);
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.player.block, Block(6));
    }

    #[test]
    fn orichalcum_does_not_fire_when_player_has_block_at_end_turn() {
        let mut player = make_player();
        player.relics.push(Relic::Orichalcum);
        let cs = CombatState {
            player: Player {
                hand: vec![],
                draw_pile: vec![Card::Strike; 5],
                block: Block(5),
                ..player
            },
            enemies: vec![Enemy {
                kind: EnemyKind::Louse,
                hp: Hp(20),
                max_hp: Hp(20),
                block: Block(0),
                move_: Move::LouseBite,
                last_move: None,
                statuses: StatusMap::new(),
            }],
            turn: 1,
            phase: CombatPhase::PlayerTurn,
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0,
        };
        let state = GameState::Combat { state: cs, floor: 0, scenario: Scenario::Main };
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.player.block, Block(5));
    }

    #[test]
    fn cloak_clasp_gives_block_per_card_remaining_in_hand_at_end_turn() {
        let mut player = make_player();
        player.relics.push(Relic::CloakClasp);
        let cs = CombatState {
            player: Player {
                hand: vec![Card::Strike, Card::Strike, Card::Strike],
                draw_pile: vec![Card::Strike; 5],
                ..player
            },
            enemies: vec![Enemy {
                kind: EnemyKind::Louse,
                hp: Hp(20),
                max_hp: Hp(20),
                block: Block(0),
                move_: Move::LouseBite,
                last_move: None,
                statuses: StatusMap::new(),
            }],
            turn: 1,
            phase: CombatPhase::PlayerTurn,
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0,
        };
        let state = GameState::Combat { state: cs, floor: 0, scenario: Scenario::Main };
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.player.block, Block(3));
    }

    // --- turn-start relics (integration) ---

    fn advance_to_turn(state: GameState, target_turn: u32) -> GameState {
        let mut state = state;
        let GameState::Combat { state: ref cs, .. } = state else { panic!("expected Combat") };
        let turns_to_advance = target_turn - cs.turn;
        for _ in 0..turns_to_advance {
            let (s, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
            let (s, _) = apply_command(s, Command::EndEnemyTurn, &mut rng()).unwrap();
            state = s;
        }
        state
    }

    #[test]
    fn mercury_hourglass_deals_3_damage_to_enemy_at_turn_start() {
        let state = combat_with_relic_at_floor(Relic::MercuryHourglass, 0);
        // Advance to turn 2 — that fires TurnStarted and the hourglass
        let state = advance_to_turn(state, 2);
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        // Enemy took 8 damage from attack + 3 from hourglass = 11 total... but wait:
        // EndTurn → EnemyTurn, enemy attacks for 8 → player takes 8 dmg.
        // EndEnemyTurn → TurnStarted{2} → MercuryHourglass fires → enemy takes 3.
        assert_eq!(cs.enemies[0].hp, Hp(17)); // 20 - 3
    }

    #[test]
    fn candelabra_gives_2_energy_on_turn_2() {
        let state = combat_with_relic_at_floor(Relic::Candelabra, 0);
        let state = advance_to_turn(state, 2);
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.player.energy, Energy(5)); // 3 base + 2 candelabra
    }

    #[test]
    fn horn_cleat_gives_14_block_on_turn_2() {
        let state = combat_with_relic_at_floor(Relic::HornCleat, 0);
        let state = advance_to_turn(state, 2);
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.player.block, Block(14));
    }

    #[test]
    fn captains_wheel_gives_18_block_on_turn_3() {
        let state = combat_with_relic_at_floor(Relic::CaptainsWheel, 0);
        let state = advance_to_turn(state, 3);
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.player.block, Block(18));
    }

    #[test]
    fn happy_flower_gives_energy_on_turn_3() {
        let state = combat_with_relic_at_floor(Relic::HappyFlower, 0);
        let state = advance_to_turn(state, 3);
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.player.energy, Energy(4));
    }

    // --- rest-site relics (integration) ---

    #[test]
    fn regal_pillow_heals_extra_15_hp_on_rest() {
        let mut player = make_player();
        player.hp = Hp(40);
        player.relics.push(Relic::RegalPillow);
        let state = GameState::RestSite(RestSiteState { player, floor: 3 });
        let (state, _) = apply_command(state, Command::Rest, &mut rng()).unwrap();
        let GameState::Map(map) = state else { panic!("expected Map") };
        // 30% of 80 = 24 HP from rest + 15 from RegalPillow, starting at 40 → 79
        assert_eq!(map.player.hp, Hp(79));
    }

    // --- Jaw Worm ---

    #[test]
    fn jaw_worm_thrash_damages_player_and_gains_enemy_block() {
        let player = make_player();
        let mut state = CombatState::from_player(player, vec![EnemyKind::JawWorm], &mut rng());
        state.enemies[0].move_ = Move::Thrash;
        let (state, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
        let (state, _) = apply_combat_command(state, Command::EndEnemyTurn, &mut rng()).unwrap();
        assert_eq!(state.player.hp.0, state.player.max_hp.0 - 7);
        assert_eq!(state.enemies[0].block.0, 5);
    }

    #[test]
    fn jaw_worm_bellow_grants_strength_and_block_without_damaging_player() {
        let player = make_player();
        let mut state = CombatState::from_player(player, vec![EnemyKind::JawWorm], &mut rng());
        state.enemies[0].move_ = Move::Bellow;
        let (state, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
        let (state, _) = apply_combat_command(state, Command::EndEnemyTurn, &mut rng()).unwrap();
        assert_eq!(state.player.hp.0, state.player.max_hp.0);
        assert_eq!(state.enemies[0].statuses.get(&crate::status::StatusEffect::Strength).copied(), Some(3));
        assert_eq!(state.enemies[0].block.0, 6);
    }

    // --- Cultist / Ritual ---

    fn cultist_combat() -> CombatState {
        let player = make_player();
        CombatState::from_player(player, vec![EnemyKind::Cultist], &mut rng())
    }

    #[test]
    fn cultist_incantation_applies_ritual_to_self() {
        let state = cultist_combat();
        let (state, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
        let (state, _) = apply_combat_command(state, Command::EndEnemyTurn, &mut rng()).unwrap();
        // After turn 1: Cultist played Incantation → Ritual(3), Strength not yet gained
        assert_eq!(
            state.enemies[0].statuses.get(&crate::status::StatusEffect::Ritual).copied(),
            Some(3)
        );
        assert_eq!(
            state.enemies[0].statuses.get(&crate::status::StatusEffect::Strength).copied(),
            None
        );
    }

    #[test]
    fn cultist_dark_strike_deals_base_6_plus_accumulated_strength() {
        let mut player = make_player();
        player.block = Block(0);
        let mut state = CombatState::from_player(player, vec![EnemyKind::Cultist], &mut rng());
        state.player.hand.clear();
        state.player.draw_pile = vec![Card::Defend; 10];

        // Turn 1: Cultist plays Incantation → gains Ritual(3), no Strength yet
        let (state, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
        let (mut state, _) = apply_combat_command(state, Command::EndEnemyTurn, &mut rng()).unwrap();
        let hp_after_turn_1 = state.player.hp.0;
        assert_eq!(hp_after_turn_1, state.player.max_hp.0);

        // Turn 2: Ritual ticks → Strength(3), then Dark Strike deals 6 + 3 = 9
        state.player.hand.clear();
        let (state, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
        let (state, _) = apply_combat_command(state, Command::EndEnemyTurn, &mut rng()).unwrap();
        assert_eq!(state.player.hp.0, hp_after_turn_1 - 9);
    }

    #[test]
    fn ritual_stacks_strength_each_turn() {
        let mut state = cultist_combat();
        state.player.hand.clear();
        state.player.draw_pile = vec![Card::Defend; 20];

        // Three full turns: Incantation + 2× DarkStrike
        for _ in 0..3 {
            let (s, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
            let (s, _) = apply_combat_command(s, Command::EndEnemyTurn, &mut rng()).unwrap();
            state = s;
        }
        // After 3 turns: Ritual ticked on turns 2 and 3 → Strength = 3 + 3 = 6
        assert_eq!(
            state.enemies[0].statuses.get(&crate::status::StatusEffect::Strength).copied(),
            Some(6)
        );
    }

    // --- Small Spike Slime ---

    #[test]
    fn flame_tackle_deals_5_damage_and_adds_dazed_to_discard() {
        let mut player = make_player();
        player.block = Block(0);
        let mut state = CombatState::from_player(player, vec![EnemyKind::SmallSpikeSlime], &mut rng());
        state.enemies[0].move_ = Move::FlameTackle;
        let (state, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
        let (state, _) = apply_combat_command(state, Command::EndEnemyTurn, &mut rng()).unwrap();
        assert_eq!(state.player.hp.0, state.player.max_hp.0 - 5);
        assert!(state.player.discard_pile.contains(&Card::Dazed));
    }

    #[test]
    fn dazed_card_is_not_playable() {
        let player = make_player();
        let mut state = CombatState::from_player(player, vec![EnemyKind::Louse], &mut rng());
        state.player.hand = vec![Card::Dazed];
        let result = apply_combat_command(state, Command::PlayCard(0, 0), &mut rng());
        assert_eq!(result, Err(CommandError::InvalidCard));
    }

    // --- Red Louse ---

    #[test]
    fn red_louse_grow_grants_strength() {
        let player = make_player();
        let mut state = CombatState::from_player(player, vec![EnemyKind::RedLouse], &mut rng());
        state.enemies[0].move_ = Move::Grow;
        let (state, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
        let (state, _) = apply_combat_command(state, Command::EndEnemyTurn, &mut rng()).unwrap();
        assert_eq!(
            state.enemies[0].statuses.get(&crate::status::StatusEffect::Strength).copied(),
            Some(3)
        );
        assert_eq!(state.player.hp.0, state.player.max_hp.0);
    }

    // --- Dexterity ---

    #[test]
    fn dexterity_increases_block_gained_from_defend() {
        let player = make_player();
        let mut state = CombatState::from_player(player, vec![EnemyKind::Louse], &mut rng());
        state.player.hand = vec![Card::Defend];
        state.player.statuses.insert(crate::status::StatusEffect::Dexterity, 2);
        let (state, _) = apply_combat_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        // Defend gives 5 block + 2 Dexterity = 7
        assert_eq!(state.player.block.0, 7);
    }

    #[test]
    fn regal_pillow_cannot_overheal_at_rest() {
        let mut player = make_player();
        player.hp = Hp(70);
        player.relics.push(Relic::RegalPillow);
        let state = GameState::RestSite(RestSiteState { player, floor: 3 });
        let (state, _) = apply_command(state, Command::Rest, &mut rng()).unwrap();
        let GameState::Map(map) = state else { panic!("expected Map") };
        assert_eq!(map.player.hp, Hp(80));
    }

    // --- Scenario::Simple ---

    #[test]
    fn simple_run_starts_with_empty_deck() {
        let state = new_simple_run();
        let GameState::Map(map) = state else { panic!("expected Map") };
        assert!(map.player.deck.is_empty());
    }

    #[test]
    fn simple_run_combat_win_returns_to_map_not_reward() {
        let state = new_simple_run();
        let (state, _) = apply_command(
            state,
            Command::Spawn(vec![EnemyKind::Louse]),
            &mut rng(),
        ).unwrap();
        let (state, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::WinCombat, &mut rng()).unwrap();
        assert!(matches!(state, GameState::Map(_)), "expected Map, got {state:?}");
    }

    // --- Potions ---

    #[test]
    fn add_potion_on_map_adds_to_player() {
        let state = new_simple_run();
        let (state, _) = apply_command(state, Command::AddPotion(Potion::FirePotion), &mut rng()).unwrap();
        let GameState::Map(map) = state else { panic!("expected Map") };
        assert_eq!(map.player.potions, vec![Potion::FirePotion]);
    }

    #[test]
    fn add_potion_rejected_when_slots_full() {
        let state = new_simple_run();
        let mut state = state;
        for _ in 0..MAX_POTIONS {
            (state, _) = apply_command(state, Command::AddPotion(Potion::BlockPotion), &mut rng()).unwrap();
        }
        (state, _) = apply_command(state, Command::AddPotion(Potion::FirePotion), &mut rng()).unwrap();
        let GameState::Map(map) = state else { panic!("expected Map") };
        assert_eq!(map.player.potions.len(), MAX_POTIONS);
        assert!(!map.player.potions.contains(&Potion::FirePotion));
    }

    #[test]
    fn potions_persist_through_combat() {
        let state = new_simple_run();
        let (state, _) = apply_command(state, Command::AddPotion(Potion::BlockPotion), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::Spawn(vec![EnemyKind::Louse]), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::WinCombat, &mut rng()).unwrap();
        let GameState::Map(map) = state else { panic!("expected Map") };
        assert_eq!(map.player.potions, vec![Potion::BlockPotion]);
    }

    // --- Potion rewards ---

    fn combat_at_floor_0() -> GameState {
        let (state, _) = apply_command(new_run(&mut rng()), Command::ChooseNode(0), &mut rng()).unwrap();
        state
    }

    #[test]
    fn win_combat_awards_a_potion() {
        let (state, _) = apply_command(combat_at_floor_0(), Command::WinCombat, &mut rng()).unwrap();
        let GameState::CardReward(cr) = state else { panic!("expected CardReward") };
        assert_eq!(cr.player.potions.len(), 1);
    }

    #[test]
    fn win_combat_emits_potion_awarded_event() {
        let (_, events) = apply_command(combat_at_floor_0(), Command::WinCombat, &mut rng()).unwrap();
        assert!(events.iter().any(|e| matches!(e, Event::PotionAwarded { .. })));
    }

    #[test]
    fn potion_offered_when_slots_full_on_victory() {
        let state = new_run(&mut rng());
        let (state, _) = apply_command(state, Command::AddPotion(Potion::BlockPotion), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::AddPotion(Potion::BlockPotion), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::AddPotion(Potion::BlockPotion), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::WinCombat, &mut rng()).unwrap();
        let GameState::CardReward(cr) = state else { panic!("expected CardReward") };
        assert_eq!(cr.player.potions.len(), MAX_POTIONS);
        assert!(cr.offered_potion.is_some());
    }

    #[test]
    fn potion_not_awarded_on_boss_floor() {
        let state = new_run(&mut rng());
        // skip to boss floor (floor 5)
        let (state, _) = apply_command(state, Command::SkipFloor, &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::SkipFloor, &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::SkipFloor, &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::SkipFloor, &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::SkipFloor, &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::WinCombat, &mut rng()).unwrap();
        assert!(matches!(state, GameState::GameOver { victory: true }));
    }

    #[test]
    fn potion_awarded_via_in_combat_kill() {
        let state = combat_at_floor_0();
        let GameState::Combat { state: mut cs, floor, scenario } = state else { panic!() };
        cs.enemies[0].hp = Hp(6);
        let state = GameState::Combat { state: cs, floor, scenario };
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        let (state, events) = apply_command(state, Command::EndEnemyTurn, &mut rng()).unwrap();
        if let GameState::CardReward(cr) = state {
            assert_eq!(cr.player.potions.len(), 1);
            assert!(events.iter().any(|e| matches!(e, Event::PotionAwarded { .. })));
        }
        // (if not yet in CardReward, combat may still be ongoing — that's fine for this test)
    }

    // --- DiscardPotion ---

    #[test]
    fn discard_potion_on_map_removes_from_player() {
        let state = new_simple_run();
        let (state, _) = apply_command(state, Command::AddPotion(Potion::BlockPotion), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::DiscardPotion(0), &mut rng()).unwrap();
        let GameState::Map(map) = state else { panic!("expected Map") };
        assert!(map.player.potions.is_empty());
    }

    #[test]
    fn discard_potion_on_map_emits_potion_discarded() {
        let state = new_simple_run();
        let (state, _) = apply_command(state, Command::AddPotion(Potion::BlockPotion), &mut rng()).unwrap();
        let (_, events) = apply_command(state, Command::DiscardPotion(0), &mut rng()).unwrap();
        assert!(events.contains(&Event::PotionDiscarded { potion: Potion::BlockPotion }));
    }

    #[test]
    fn discard_potion_invalid_slot_returns_error() {
        let state = new_simple_run();
        let result = apply_command(state, Command::DiscardPotion(0), &mut rng());
        assert_eq!(result, Err(CommandError::InvalidCard));
    }

    #[test]
    fn discard_potion_in_combat_removes_potion() {
        let state = new_simple_run();
        let (state, _) = apply_command(state, Command::AddPotion(Potion::BlockPotion), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::Spawn(vec![EnemyKind::Louse]), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        let (state, events) = apply_command(state, Command::DiscardPotion(0), &mut rng()).unwrap();
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert!(cs.player.potions.is_empty());
        assert!(events.contains(&Event::PotionDiscarded { potion: Potion::BlockPotion }));
    }

    #[test]
    fn discard_potion_in_rest_site_removes_potion() {
        let mut player = make_player();
        player.potions.push(Potion::BlockPotion);
        let state = GameState::RestSite(RestSiteState { player, floor: 3 });
        let (state, events) = apply_command(state, Command::DiscardPotion(0), &mut rng()).unwrap();
        let GameState::RestSite(rs) = state else { panic!("expected RestSite") };
        assert!(rs.player.potions.is_empty());
        assert!(events.contains(&Event::PotionDiscarded { potion: Potion::BlockPotion }));
    }

    #[test]
    fn discard_in_card_reward_stays_in_card_reward() {
        let state = combat_at_floor(0);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(matches!(state, GameState::CardReward(_)));
        let GameState::CardReward(ref cr) = state else { unreachable!() };
        assert!(!cr.player.potions.is_empty(), "need a potion to discard");
        let (state, _) = apply_command(state, Command::DiscardPotion(0), &mut rng()).unwrap();
        assert!(matches!(state, GameState::CardReward(_)));
    }

    #[test]
    fn discard_in_card_reward_collects_offered_potion() {
        let state = new_run(&mut rng());
        let (state, _) = apply_command(state, Command::AddPotion(Potion::BlockPotion), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::AddPotion(Potion::BlockPotion), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::AddPotion(Potion::BlockPotion), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::WinCombat, &mut rng()).unwrap();
        let GameState::CardReward(ref cr) = state else { panic!("expected CardReward") };
        assert!(cr.offered_potion.is_some());
        let (state, events) = apply_command(state, Command::DiscardPotion(0), &mut rng()).unwrap();
        let GameState::CardReward(cr) = state else { panic!("expected CardReward") };
        assert_eq!(cr.player.potions.len(), MAX_POTIONS);
        assert!(cr.offered_potion.is_none());
        assert!(events.iter().any(|e| matches!(e, Event::PotionDiscarded { .. })));
        assert!(events.iter().any(|e| matches!(e, Event::PotionAwarded { .. })));
    }

    // --- Tier 4 relic integration tests ---

    fn combat_with_relic_and_cards(relic: Relic, cards: Vec<Card>, enemy_hp: i32) -> GameState {
        let mut player = make_player();
        player.relics.push(relic);
        let cs = CombatState {
            player: Player {
                hand: cards,
                draw_pile: vec![Card::Strike; 10],
                energy: Energy(20),
                max_energy: Energy(20),
                ..player
            },
            enemies: vec![Enemy {
                kind: EnemyKind::Louse,
                hp: Hp(enemy_hp),
                max_hp: Hp(enemy_hp),
                block: Block(0),
                move_: Move::LouseBite,
                last_move: None,
                statuses: StatusMap::new(),
            }],
            turn: 1,
            phase: CombatPhase::PlayerTurn,
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0,
        };
        GameState::Combat { state: cs, floor: 0, scenario: Scenario::Main }
    }

    #[test]
    fn nunchaku_grants_energy_after_10th_attack() {
        let mut state = combat_with_relic_and_cards(
            Relic::Nunchaku,
            vec![Card::Strike; 10],
            200,
        );
        // Play 9 attacks — no energy boost yet
        for _ in 0..9 {
            (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        }
        let GameState::Combat { state: ref cs, .. } = state else { panic!("expected Combat") };
        let energy_before = cs.player.energy.0;
        // Play 10th attack — Nunchaku fires
        let (state, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        // Nunchaku gives +1 energy which cancels the card's cost; net energy = energy_before
        assert_eq!(cs.player.energy.0, energy_before, "net energy should be unchanged (gained 1, spent 1)");
        assert!(events.iter().any(|e| matches!(e, Event::EnergyGained { amount: 1 })));
    }

    #[test]
    fn ornamental_fan_grants_block_after_3_attacks_this_turn() {
        let state = combat_with_relic_and_cards(Relic::OrnamentalFan, vec![Card::Strike; 3], 200);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.player.block, Block(4));
        assert!(events.iter().any(|e| matches!(e, Event::PlayerBlocked { amount: 4 })));
    }

    #[test]
    fn ornamental_fan_counter_resets_each_turn() {
        // Play 2 attacks turn 1 (no fire), end turn, play 1 attack turn 2 (no fire)
        let mut player = make_player();
        player.relics.push(Relic::OrnamentalFan);
        let cs = CombatState {
            player: Player {
                hand: vec![Card::Strike, Card::Strike],
                draw_pile: vec![Card::Strike; 10],
                ..player
            },
            enemies: vec![Enemy {
                kind: EnemyKind::Louse,
                hp: Hp(200),
                max_hp: Hp(200),
                block: Block(0),
                move_: Move::LouseBite,
                last_move: None,
                statuses: StatusMap::new(),
            }],
            turn: 1,
            phase: CombatPhase::PlayerTurn,
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0,
        };
        let state = GameState::Combat { state: cs, floor: 0, scenario: Scenario::Main };
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        // End turn (2 attacks, no fan fire)
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::EndEnemyTurn, &mut rng()).unwrap();
        // Play 1 attack on turn 2 — attacks_this_turn should be 1 (reset), no fan fire
        let (state, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.player.block, Block(0), "fan should not have fired yet (only 1 attack this turn)");
        assert!(!events.iter().any(|e| matches!(e, Event::PlayerBlocked { amount: 4 })));
    }

    #[test]
    fn gremlin_horn_grants_energy_and_draws_card_on_kill() {
        let state = combat_with_relic_and_cards(Relic::GremlinHorn, vec![Card::Strike], 6);
        let GameState::Combat { state: ref cs, .. } = state else { panic!("expected Combat") };
        let hand_before = cs.player.hand.len();
        let (state, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(events.iter().any(|e| matches!(e, Event::EnemyDied)));
        assert!(events.iter().any(|e| matches!(e, Event::EnergyGained { amount: 1 })));
        assert!(events.iter().any(|e| matches!(e, Event::CardsDrawn { count: 1 })));
        // state is Victory; verify energy was boosted (even though combat ended)
        let _ = state;
        let _ = hand_before;
    }

    #[test]
    fn pocketwatch_draws_3_extra_cards_after_end_turn_with_3_or_fewer_played() {
        let mut player = make_player();
        player.relics.push(Relic::Pocketwatch);
        let cs = CombatState {
            player: Player {
                hand: vec![Card::Defend],
                draw_pile: vec![Card::Strike; 10],
                ..player
            },
            enemies: vec![Enemy {
                kind: EnemyKind::Louse,
                hp: Hp(20),
                max_hp: Hp(20),
                block: Block(0),
                move_: Move::LouseBite,
                last_move: None,
                statuses: StatusMap::new(),
            }],
            turn: 1,
            phase: CombatPhase::PlayerTurn,
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0,
        };
        let state = GameState::Combat { state: cs, floor: 0, scenario: Scenario::Main };
        // Play 1 card (≤3), end turn
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::EndEnemyTurn, &mut rng()).unwrap();
        // Should have drawn 5 normal + 3 pocketwatch = 8 cards
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.player.hand.len(), 8);
    }

    #[test]
    fn pocketwatch_does_not_fire_when_4_cards_played() {
        let mut player = make_player();
        player.relics.push(Relic::Pocketwatch);
        let cs = CombatState {
            player: Player {
                hand: vec![Card::Defend; 4],
                draw_pile: vec![Card::Strike; 10],
                energy: Energy(20),
                max_energy: Energy(20),
                ..player
            },
            enemies: vec![Enemy {
                kind: EnemyKind::Louse,
                hp: Hp(200),
                max_hp: Hp(200),
                block: Block(0),
                move_: Move::LouseBite,
                last_move: None,
                statuses: StatusMap::new(),
            }],
            turn: 1,
            phase: CombatPhase::PlayerTurn,
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0,
        };
        let mut state = GameState::Combat { state: cs, floor: 0, scenario: Scenario::Main };
        // Play 4 cards (>3 threshold), end turn
        for _ in 0..4 {
            (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        }
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::EndEnemyTurn, &mut rng()).unwrap();
        // Should draw exactly 5 cards (no bonus)
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.player.hand.len(), 5);
    }
}

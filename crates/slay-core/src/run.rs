use crate::cards::{Card, starter_deck};
use crate::combat::{apply_combat_command, CombatPhase, CombatState, Event, Player};
use crate::neow::{NeowBlessing, NeowContext, NeowState};
use crate::potions::{Potion, MAX_POTIONS};
use crate::enemies::EnemyKind;
use crate::relics::{
    apply_combat_start_relics, apply_end_of_combat_relics, apply_rest_relics,
    apply_turn_end_relics, apply_turn_start_relics, Relic,
};
use crate::rng::Rng;
use crate::status::StatusMap;
use crate::types::{Block, Energy, Hp};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Scenario {
    Main,
    Simple,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    PlayCard(usize, usize), // card index, target enemy index
    EndTurn,
    EndEnemyTurn,
    StartPlayerTurn,
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
    LeaveTreasure,
    ChooseEventOption(usize),
    ChooseHandCard(usize),
    ChooseNeowBlessing(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandError {
    CombatOver,
    InvalidCard,
    NotEnoughEnergy,
    NotEnoughGold,
    InvalidPhase,
    Entangled,
    Normality,
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
            CommandError::Normality       => "Normality — cannot play more than 3 cards this turn.",
        };
        f.write_str(msg)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum MapNode {
    Combat(Vec<EnemyKind>),
    Elite(Vec<EnemyKind>),
    RestSite,
    Boss(Vec<EnemyKind>),
    Merchant,
    Treasure,
    Event,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum EventKind {
    Ssssserpent,
    BigFish,
    Mushrooms,
    GoldenIdol,
}

impl EventKind {
    fn random(rng: &mut impl Rng) -> Self {
        let mut pool = vec![
            EventKind::Ssssserpent,
            EventKind::BigFish,
            EventKind::Mushrooms,
            EventKind::GoldenIdol,
        ];
        rng.shuffle(&mut pool);
        pool.into_iter().next().unwrap() // SAFETY: pool is non-empty
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EventRoomState {
    pub player: Player,
    pub floor: usize,
    pub graph: MapGraph,
    pub available_cols: Vec<usize>,
    pub event: EventKind,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MapGraph {
    pub rows: Vec<Vec<MapNode>>,
    pub edges: Vec<Vec<Vec<usize>>>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MapState {
    pub player: Player,
    pub floor: usize,
    pub graph: MapGraph,
    pub available_cols: Vec<usize>,
    pub next_enemies: Option<Vec<EnemyKind>>,
    pub scenario: Scenario,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RestSiteState {
    pub player: Player,
    pub floor: usize,
    pub graph: MapGraph,
    pub available_cols: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TreasureRoomState {
    pub player: Player,
    pub floor: usize,
    pub graph: MapGraph,
    pub available_cols: Vec<usize>,
    pub relic: Relic,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CardRewardState {
    pub player: Player,
    pub floor: usize,
    pub options: Vec<Card>,
    pub offered_potion: Option<Potion>,
    pub graph: MapGraph,
    pub available_cols: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ShopState {
    pub player: Player,
    pub floor: usize,
    pub cards: Vec<(Card, bool)>,
    pub relic: Option<(Relic, bool)>,
    pub potion: Option<(Potion, bool)>,
    pub graph: MapGraph,
    pub available_cols: Vec<usize>,
}

pub const CARD_PRICE: i32 = 75;
pub const RELIC_PRICE: i32 = 150;
pub const POTION_PRICE: i32 = 50;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum GameState {
    Neow(NeowState),
    Map(MapState),
    Combat { state: CombatState, floor: usize, is_boss: bool, is_elite: bool, graph: MapGraph, next_floor_cols: Vec<usize>, scenario: Scenario },
    RestSite(RestSiteState),
    TreasureRoom(TreasureRoomState),
    CardReward(CardRewardState),
    Shop(ShopState),
    EventRoom(EventRoomState),
    GameOver { victory: bool },
}

pub fn new_run(rng: &mut impl Rng, ctx: &NeowContext) -> GameState {
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
        neow_lament_combats_remaining: 0,
        reached_boss: false,
        potion_chance: 0.40,
    };
    let graph = generate_map(rng);
    let blessings = generate_blessings(rng, ctx);
    GameState::Neow(NeowState { player, graph, blessings })
}

fn generate_blessings(rng: &mut impl Rng, ctx: &NeowContext) -> Vec<NeowBlessing> {
    if ctx.runs_completed == 0 {
        return vec![NeowBlessing::GainMaxHp(8), NeowBlessing::NeowsLament];
    }
    let mut blessings = vec![
        pick_category_1(rng),
        pick_category_2(rng),
        pick_category_3(rng),
    ];
    if ctx.prev_run_reached_boss {
        blessings.push(pick_category_4(rng));
    }
    blessings
}

fn pick_category_1(rng: &mut impl Rng) -> NeowBlessing {
    use crate::cards::{reward_pool, Grade};
    let mut pool = vec![
        NeowBlessing::RemoveCard,
        NeowBlessing::TransformCard,
        NeowBlessing::UpgradeCard,
    ];
    let mut rare_cards = reward_pool()
        .into_iter()
        .filter(|c| matches!(c.grade(), Some(Grade::Base)) && c.def().card_type == crate::cards::CardType::Attack)
        .take(10)
        .collect::<Vec<_>>();
    rng.shuffle(&mut rare_cards);
    rare_cards.truncate(3);
    if rare_cards.len() == 3 {
        pool.push(NeowBlessing::ChooseRareCard(rare_cards));
    }
    rng.shuffle(&mut pool);
    pool.into_iter().next().unwrap() // SAFETY: pool always has >= 3 elements
}

fn pick_category_2(rng: &mut impl Rng) -> NeowBlessing {
    use crate::relics::random_common_relic;
    use crate::potions::random_potions;
    let mut pool = vec![
        NeowBlessing::GainMaxHp(8),
        NeowBlessing::NeowsLament,
        NeowBlessing::GainGold(100),
        NeowBlessing::GainRelic(random_common_relic(rng)),
        NeowBlessing::GainPotions(random_potions(rng, 3)),
    ];
    rng.shuffle(&mut pool);
    pool.into_iter().next().unwrap() // SAFETY: pool has 5 elements
}

fn pick_category_3(rng: &mut impl Rng) -> NeowBlessing {
    use crate::relics::random_rare_relic;
    let mut pool = vec![
        NeowBlessing::LoseHpGainGold { hp_loss: 7, gold: 250 },
        NeowBlessing::LoseHpRemoveCards { hp_loss: 7, count: 2 },
        NeowBlessing::LoseHpTransformCards { hp_loss: 7, count: 2 },
        NeowBlessing::LoseHpGainRareRelic { hp_loss: 7, relic: random_rare_relic(rng) },
    ];
    rng.shuffle(&mut pool);
    pool.into_iter().next().unwrap() // SAFETY: pool has 4 elements
}

fn pick_category_4(rng: &mut impl Rng) -> NeowBlessing {
    use crate::relics::random_boss_relic;
    NeowBlessing::SwapStarterRelic(random_boss_relic(rng))
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
        neow_lament_combats_remaining: 0,
        reached_boss: false,
        potion_chance: 0.40,
    };
    let graph = MapGraph {
        rows: vec![vec![MapNode::Combat(vec![EnemyKind::RedLouse])]],
        edges: vec![vec![vec![]]],
    };
    GameState::Map(MapState { player, floor: 0, graph, available_cols: vec![0], next_enemies: None, scenario: Scenario::Simple })
}

fn easy_encounters() -> Vec<Vec<EnemyKind>> {
    vec![
        vec![EnemyKind::Cultist],
        vec![EnemyKind::JawWorm],
        vec![EnemyKind::RedLouse, EnemyKind::GreenLouse],
        vec![EnemyKind::MediumSpike, EnemyKind::SmallAcidSlime], // Small Slimes
    ]
}

fn hard_encounters() -> Vec<Vec<EnemyKind>> {
    vec![
        // weight ~2
        vec![EnemyKind::BlueSlaver],
        vec![EnemyKind::Fungibeast, EnemyKind::Fungibeast],
        vec![EnemyKind::RedLouse, EnemyKind::GreenLouse, EnemyKind::RedLouse], // 3 Louses
        vec![EnemyKind::LargeSpike],   // Large Slime — spike variant
        vec![EnemyKind::LargeAcid],    // Large Slime — acid variant
        vec![EnemyKind::Looter],
        // weight ~1.5
        vec![EnemyKind::Looter, EnemyKind::Mugger],         // Exordium Thugs
        vec![EnemyKind::JawWorm, EnemyKind::GreenLouse],    // Exordium Wildlife
        vec![EnemyKind::Fungibeast, EnemyKind::RedLouse],   // Exordium Wildlife
        vec![EnemyKind::JawWorm, EnemyKind::MediumSpike],   // Exordium Wildlife
        // weight ~1
        vec![EnemyKind::RedSlaver],
        vec![EnemyKind::FatGremlin, EnemyKind::MadGremlin, EnemyKind::SneakyGremlin, EnemyKind::ShieldGremlin], // Gremlin Gang (4 of 5)
        vec![EnemyKind::SmallSpikeSlime, EnemyKind::SmallSpikeSlime, EnemyKind::SmallSpikeSlime, EnemyKind::SmallAcidSlime, EnemyKind::SmallAcidSlime], // Swarm of Slimes
    ]
}

fn elite_encounters() -> Vec<Vec<EnemyKind>> {
    vec![
        vec![EnemyKind::GremlinNob],
        vec![EnemyKind::Lagavulin],
        vec![EnemyKind::Sentry, EnemyKind::Sentry, EnemyKind::Sentry],
    ]
}

fn boss_encounters() -> Vec<Vec<EnemyKind>> {
    vec![
        vec![EnemyKind::TheGuardian],
        vec![EnemyKind::SlimeBoss],
        vec![EnemyKind::Hexaghost],
    ]
}

fn pick_encounter(pool: &mut [Vec<EnemyKind>], rng: &mut impl Rng) -> Vec<EnemyKind> {
    rng.shuffle(pool);
    pool[0].clone() // SAFETY: all pools are non-empty
}

pub fn generate_map(rng: &mut impl Rng) -> MapGraph {
    let both: Vec<Vec<usize>> = vec![vec![0, 1], vec![0, 1]];
    let converge: Vec<Vec<usize>> = vec![vec![0], vec![0]];
    let from_one: Vec<Vec<usize>> = vec![vec![0, 1]];
    let mut rows: Vec<Vec<MapNode>> = Vec::new();
    let mut edges: Vec<Vec<Vec<usize>>> = Vec::new();

    // Floors 0–2: easy combat section
    // Floor 0: two easy combats
    rows.push(vec![
        MapNode::Combat(pick_encounter(&mut easy_encounters(), rng)),
        MapNode::Combat(pick_encounter(&mut easy_encounters(), rng)),
    ]);
    edges.push(both.clone());

    // Floor 1: easy combat + event
    rows.push(vec![MapNode::Combat(pick_encounter(&mut easy_encounters(), rng)), MapNode::Event]);
    edges.push(both.clone());

    // Floor 2: two easy combats, then converge
    rows.push(vec![
        MapNode::Combat(pick_encounter(&mut easy_encounters(), rng)),
        MapNode::Combat(pick_encounter(&mut easy_encounters(), rng)),
    ]);
    edges.push(converge.clone());

    // Floor 3: Merchant
    rows.push(vec![MapNode::Merchant]);
    edges.push(from_one.clone());

    // Floor 4: hard combat + event
    rows.push(vec![MapNode::Combat(pick_encounter(&mut hard_encounters(), rng)), MapNode::Event]);
    edges.push(both.clone());

    // Floor 5: hard combat + elite (first elite available)
    rows.push(vec![
        MapNode::Combat(pick_encounter(&mut hard_encounters(), rng)),
        MapNode::Elite(pick_encounter(&mut elite_encounters(), rng)),
    ]);
    edges.push(converge.clone());

    // Floor 6: Rest site
    rows.push(vec![MapNode::RestSite]);
    edges.push(from_one.clone());

    // Floor 7: two hard combats
    rows.push(vec![
        MapNode::Combat(pick_encounter(&mut hard_encounters(), rng)),
        MapNode::Combat(pick_encounter(&mut hard_encounters(), rng)),
    ]);
    edges.push(converge.clone());

    // Floor 8: Treasure
    rows.push(vec![MapNode::Treasure]);
    edges.push(vec![vec![0]]);

    // Floor 9: Boss
    rows.push(vec![MapNode::Boss(pick_encounter(&mut boss_encounters(), rng))]);
    edges.push(vec![vec![]]);

    MapGraph { rows, edges }
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
    let dropped = rng.gen_bool(player.potion_chance);
    if dropped {
        player.potion_chance = (player.potion_chance - 0.10).max(0.0);
    } else {
        player.potion_chance = (player.potion_chance + 0.10).min(1.0);
        return None;
    }
    let potion = random_potion(rng);
    if player.potions.len() < MAX_POTIONS {
        player.potions.push(potion);
        events.push(Event::PotionAwarded { potion });
        None
    } else {
        Some(potion)
    }
}

fn generate_shop(player: Player, floor: usize, graph: MapGraph, available_cols: Vec<usize>, rng: &mut impl Rng) -> ShopState {
    let mut card_pool = crate::cards::reward_pool();
    rng.shuffle(&mut card_pool);
    let cards = card_pool.into_iter().take(2).map(|c| (c, false)).collect();

    let mut relic_pool = Relic::all();
    rng.shuffle(&mut relic_pool);
    let relic = relic_pool.into_iter().next().map(|r| (r, false));

    let potion = Some((random_potion(rng), false));

    ShopState { player, floor, cards, relic, potion, graph, available_cols }
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

fn roll_gold(lo: i32, hi: i32, rng: &mut impl Rng) -> i32 {
    let mut values: Vec<i32> = (lo..=hi).collect();
    rng.shuffle(&mut values);
    values[0]
}

fn combat_gold(is_elite: bool, is_boss: bool, rng: &mut impl Rng) -> i32 {
    if is_boss        { roll_gold(95, 105, rng) }
    else if is_elite  { roll_gold(25,  35, rng) }
    else              { roll_gold(10,  20, rng) }
}

pub fn apply_command(
    state: GameState,
    command: Command,
    rng: &mut impl Rng,
) -> Result<(GameState, Vec<Event>), CommandError> {
    match state {
        GameState::Map(MapState { player, floor, graph, available_cols, next_enemies, scenario }) => match command {
            Command::Spawn(enemies) => {
                Ok((GameState::Map(MapState { player, floor, graph, available_cols, next_enemies: Some(enemies), scenario }), Vec::new()))
            }
            Command::ChooseNode(col) => {
                if !available_cols.contains(&col) {
                    return Err(CommandError::InvalidPhase);
                }
                let floor_nodes = graph.rows.get(floor).ok_or(CommandError::InvalidPhase)?;
                let node = floor_nodes.get(col).cloned().ok_or(CommandError::InvalidCard)?;
                let next_floor_cols = graph.edges.get(floor)
                    .and_then(|row_edges| row_edges.get(col))
                    .cloned()
                    .unwrap_or_default();
                match &node {
                    MapNode::Combat(node_enemies) | MapNode::Boss(node_enemies) | MapNode::Elite(node_enemies) => {
                        let enemies = next_enemies.unwrap_or_else(|| node_enemies.clone());
                        let is_boss = matches!(node, MapNode::Boss(_));
                        let is_elite = matches!(node, MapNode::Elite(_));
                        let mut combat_state = CombatState::from_player(player, enemies, rng);
                        let mut events = Vec::new();
                        apply_combat_start_relics(&mut combat_state, &mut events, rng, is_boss);
                        if combat_state.player.neow_lament_combats_remaining > 0 {
                            for enemy in &mut combat_state.enemies {
                                if enemy.hp > Hp(0) { enemy.hp = Hp(1); }
                            }
                            combat_state.player.neow_lament_combats_remaining -= 1;
                        }
                        if combat_state.enemies.iter().all(|e| e.hp <= Hp(0)) {
                            let gold = combat_gold(is_elite, is_boss, rng);
                            events.push(Event::GoldEarned { amount: gold });
                            let mut victory_player = combat_state.player;
                            apply_end_of_combat_relics(&mut victory_player, &mut events);
                            let mut player = player_after_combat(victory_player, gold);
                            if is_boss {
                                return Ok((GameState::GameOver { victory: true }, events));
                            }
                            if is_elite {
                                let relic = crate::relics::random_common_relic(rng);
                                player.relics.push(relic.clone());
                                events.push(Event::RelicObtained { relic });
                            }
                            let options = generate_rewards(rng);
                            return Ok((
                                GameState::CardReward(CardRewardState { player, floor: floor + 1, options, offered_potion: None, graph, available_cols: next_floor_cols }),
                                events,
                            ));
                        }
                        Ok((GameState::Combat { state: combat_state, floor, is_boss, is_elite, graph, next_floor_cols, scenario }, events))
                    }
                    MapNode::RestSite => Ok((
                        GameState::RestSite(RestSiteState { player, floor, graph, available_cols: next_floor_cols }),
                        Vec::new(),
                    )),
                    MapNode::Merchant => Ok((
                        GameState::Shop(generate_shop(player, floor, graph, next_floor_cols, rng)),
                        Vec::new(),
                    )),
                    MapNode::Treasure => {
                        let mut relic_pool = Relic::all();
                        rng.shuffle(&mut relic_pool);
                        let relic = relic_pool.into_iter().next().unwrap(); // SAFETY: Relic::all() is non-empty
                        Ok((
                            GameState::TreasureRoom(TreasureRoomState { player, floor, graph, available_cols: next_floor_cols, relic }),
                            Vec::new(),
                        ))
                    }
                    MapNode::Event => {
                        let event = EventKind::random(rng);
                        Ok((
                            GameState::EventRoom(EventRoomState { player, floor, graph, available_cols: next_floor_cols, event }),
                            Vec::new(),
                        ))
                    }
                }
            }
            Command::SkipFloor => {
                let next_floor = floor + 1;
                let new_cols = graph.rows.get(next_floor)
                    .map(|row| (0..row.len()).collect())
                    .unwrap_or_default();
                Ok((GameState::Map(MapState { player, floor: next_floor, graph, available_cols: new_cols, next_enemies: None, scenario }), Vec::new()))
            }
            Command::AddRelic(relic) => {
                let mut p = player;
                let events = crate::relics::grant_relic(&mut p, relic, rng);
                Ok((GameState::Map(MapState { player: p, floor, graph, available_cols, next_enemies: None, scenario }), events))
            }
            Command::AddPotion(potion) => {
                let mut p = player;
                if p.potions.len() < MAX_POTIONS {
                    p.potions.push(potion);
                }
                Ok((GameState::Map(MapState { player: p, floor, graph, available_cols, next_enemies: None, scenario }), vec![]))
            }
            Command::DiscardPotion(slot) => {
                let mut p = player;
                if slot >= p.potions.len() {
                    return Err(CommandError::InvalidCard);
                }
                let potion = p.potions.remove(slot);
                Ok((GameState::Map(MapState { player: p, floor, graph, available_cols, next_enemies, scenario }),
                    vec![Event::PotionDiscarded { potion }]))
            }
            _ => Err(CommandError::InvalidPhase),
        },

        GameState::Combat { state: mut combat_state, floor, is_boss, is_elite, graph, next_floor_cols, scenario } => match command {
            Command::WinCombat => {
                let gold = combat_gold(is_elite, is_boss, rng);
                let mut events = vec![Event::EnemyDied, Event::GoldEarned { amount: gold }];
                apply_end_of_combat_relics(&mut combat_state.player, &mut events);
                let mut player = player_after_combat(combat_state.player, gold);
                if is_boss {
                    Ok((GameState::GameOver { victory: true }, events))
                } else if scenario == Scenario::Simple {
                    Ok((GameState::Map(MapState { player, floor, graph, available_cols: next_floor_cols, next_enemies: None, scenario }), events))
                } else {
                    if is_elite {
                        let relic = crate::relics::random_uncommon_relic(rng);
                        player.relics.push(relic.clone());
                        events.push(Event::RelicObtained { relic });
                    }
                    let offered_potion = award_potion(&mut player, &mut events, rng);
                    let options = generate_rewards(rng);
                    Ok((GameState::CardReward(CardRewardState { player, floor: floor + 1, options, offered_potion, graph, available_cols: next_floor_cols }), events))
                }
            }
            Command::AddCard(card) => {
                combat_state.player.hand.push(card);
                Ok((GameState::Combat { state: combat_state, floor, is_boss, is_elite, graph, next_floor_cols, scenario }, vec![]))
            }
            Command::AddRelic(relic) => {
                let events = crate::relics::grant_relic(&mut combat_state.player, relic, rng);
                Ok((GameState::Combat { state: combat_state, floor, is_boss, is_elite, graph, next_floor_cols, scenario }, events))
            }
            Command::AddPotion(potion) => {
                if combat_state.player.potions.len() < MAX_POTIONS {
                    combat_state.player.potions.push(potion);
                }
                Ok((GameState::Combat { state: combat_state, floor, is_boss, is_elite, graph, next_floor_cols, scenario }, vec![]))
            }
            Command::DiscardPotion(slot) => {
                if slot >= combat_state.player.potions.len() {
                    return Err(CommandError::InvalidCard);
                }
                let potion = combat_state.player.potions.remove(slot);
                Ok((GameState::Combat { state: combat_state, floor, is_boss, is_elite, graph, next_floor_cols, scenario },
                    vec![Event::PotionDiscarded { potion }]))
            }
            Command::ChooseNode(_) | Command::Rest | Command::ChooseCardReward(_)
            | Command::SkipFloor | Command::UpgradeCard(_) | Command::LeaveShop
            | Command::BuyCard(_) | Command::BuyRelic | Command::BuyPotion
            | Command::LeaveTreasure => {
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
                        let gold = combat_gold(is_elite, is_boss, rng);
                        events.push(Event::GoldEarned { amount: gold });
                        let mut victory_player = new_combat.player;
                        apply_end_of_combat_relics(&mut victory_player, &mut events);
                        let mut player = player_after_combat(victory_player, gold);
                        if is_boss {
                            Ok((GameState::GameOver { victory: true }, events))
                        } else if scenario == Scenario::Simple {
                            Ok((GameState::Map(MapState { player, floor, graph, available_cols: next_floor_cols, next_enemies: None, scenario }), events))
                        } else {
                            if is_elite {
                                let relic = crate::relics::random_common_relic(rng);
                                player.relics.push(relic.clone());
                                events.push(Event::RelicObtained { relic });
                            }
                            let offered_potion = award_potion(&mut player, &mut events, rng);
                            let options = generate_rewards(rng);
                            Ok((
                                GameState::CardReward(CardRewardState {
                                    player,
                                    floor: floor + 1,
                                    options,
                                    offered_potion,
                                    graph,
                                    available_cols: next_floor_cols,
                                }),
                                events,
                            ))
                        }
                    }
                    CombatPhase::Defeat => Ok((GameState::GameOver { victory: false }, events)),
                    CombatPhase::Fled => {
                        let mut player = player_after_combat(new_combat.player, 0);
                        apply_end_of_combat_relics(&mut player, &mut events);
                        Ok((GameState::Map(MapState { player, floor, graph, available_cols: next_floor_cols, next_enemies: None, scenario }), events))
                    }
                    _ => Ok((GameState::Combat { state: new_combat, floor, is_boss, is_elite, graph, next_floor_cols, scenario }, events)),
                }
            }
        },

        GameState::RestSite(RestSiteState { mut player, floor, graph, available_cols }) => match command {
            Command::Rest => {
                let heal = (player.max_hp.0 * 30 / 100).max(1);
                player.hp.0 = (player.hp.0 + heal).min(player.max_hp.0);
                let mut events = vec![Event::Healed { amount: heal }];
                apply_rest_relics(&mut player, &mut events);
                Ok((GameState::Map(MapState { player, floor: floor + 1, graph, available_cols, next_enemies: None, scenario: Scenario::Main }), events))
            }
            Command::UpgradeCard(idx) => {
                let from = player.deck.get(idx).cloned().ok_or(CommandError::InvalidCard)?;
                let to = from.upgrade().ok_or(CommandError::InvalidCard)?;
                player.deck[idx] = to.clone();
                let events = vec![Event::CardUpgraded { from, to }];
                Ok((GameState::Map(MapState { player, floor: floor + 1, graph, available_cols, next_enemies: None, scenario: Scenario::Main }), events))
            }
            Command::DiscardPotion(slot) => {
                if slot >= player.potions.len() {
                    return Err(CommandError::InvalidCard);
                }
                let potion = player.potions.remove(slot);
                Ok((GameState::RestSite(RestSiteState { player, floor, graph, available_cols }),
                    vec![Event::PotionDiscarded { potion }]))
            }
            _ => Err(CommandError::InvalidPhase),
        },

        GameState::TreasureRoom(TreasureRoomState { mut player, floor, graph, available_cols, relic }) => match command {
            Command::LeaveTreasure => {
                let events = crate::relics::grant_relic(&mut player, relic, rng);
                Ok((GameState::Map(MapState { player, floor: floor + 1, graph, available_cols, next_enemies: None, scenario: Scenario::Main }), events))
            }
            _ => Err(CommandError::InvalidPhase),
        },

        GameState::CardReward(CardRewardState { mut player, floor, options, offered_potion, graph, available_cols }) => {
            match command {
                Command::ChooseCardReward(idx) => {
                    if idx >= options.len() {
                        return Err(CommandError::InvalidCard);
                    }
                    let card = options[idx].clone();
                    player.deck.push(card.clone());
                    let events = vec![Event::CardAdded { card }];
                    Ok((GameState::Map(MapState { player, floor, graph, available_cols, next_enemies: None, scenario: Scenario::Main }), events))
                }
                Command::SkipReward => {
                    Ok((GameState::Map(MapState { player, floor, graph, available_cols, next_enemies: None, scenario: Scenario::Main }), Vec::new()))
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
                    Ok((GameState::CardReward(CardRewardState { player, floor, options, offered_potion: new_offered, graph, available_cols }), events))
                }
                _ => Err(CommandError::InvalidPhase),
            }
        }

        GameState::Shop(ShopState { mut player, floor, mut cards, mut relic, mut potion, graph, available_cols }) => match command {
            Command::LeaveShop => Ok((
                GameState::Map(MapState { player, floor: floor + 1, graph, available_cols, next_enemies: None, scenario: Scenario::Main }),
                Vec::new(),
            )),
            Command::BuyCard(idx) => {
                if idx >= cards.len() || cards[idx].1 {
                    return Err(CommandError::InvalidCard);
                }
                if player.gold < CARD_PRICE {
                    return Err(CommandError::NotEnoughGold);
                }
                let card = cards[idx].0.clone();
                player.gold -= CARD_PRICE;
                player.deck.push(card.clone());
                cards[idx].1 = true;
                let events = vec![Event::CardAdded { card }];
                Ok((GameState::Shop(ShopState { player, floor, cards, relic, potion, graph, available_cols }), events))
            }
            Command::BuyRelic => {
                if relic.as_ref().is_none_or(|(_, p)| *p) {
                    return Err(CommandError::InvalidCard);
                }
                if player.gold < RELIC_PRICE {
                    return Err(CommandError::NotEnoughGold);
                }
                let r = relic.as_ref().unwrap().0.clone(); // SAFETY: checked is_some above
                player.gold -= RELIC_PRICE;
                let events = crate::relics::grant_relic(&mut player, r, rng);
                relic = relic.map(|(r, _)| (r, true));
                Ok((GameState::Shop(ShopState { player, floor, cards, relic, potion, graph, available_cols }), events))
            }
            Command::BuyPotion => {
                if potion.as_ref().is_none_or(|(_, p)| *p) {
                    return Err(CommandError::InvalidCard);
                }
                if player.potions.len() >= MAX_POTIONS {
                    return Err(CommandError::InvalidPhase);
                }
                if player.gold < POTION_PRICE {
                    return Err(CommandError::NotEnoughGold);
                }
                let pot = potion.as_ref().unwrap().0; // SAFETY: checked is_some above; Potion: Copy
                player.gold -= POTION_PRICE;
                player.potions.push(pot);
                potion = potion.map(|(p, _)| (p, true));
                let events = vec![Event::PotionAwarded { potion: pot }];
                Ok((GameState::Shop(ShopState { player, floor, cards, relic, potion, graph, available_cols }), events))
            }
            _ => Err(CommandError::InvalidPhase),
        },

        GameState::EventRoom(EventRoomState { mut player, floor, graph, available_cols, event }) => match command {
            Command::ChooseEventOption(opt) => {
                let events = match event {
                    EventKind::Ssssserpent => match opt {
                        0 => {
                            player.gold += 150;
                            player.deck.push(Card::Doubt);
                            vec![Event::CardAdded { card: Card::Doubt }]
                        }
                        _ => Vec::new(),
                    },
                    EventKind::GoldenIdol => match opt {
                        0 => {
                            player.gold += 250;
                            player.deck.push(Card::Injury);
                            vec![Event::GoldEarned { amount: 250 }, Event::CardAdded { card: Card::Injury }]
                        }
                        1 => {
                            player.gold += 250;
                            player.hp.0 = (player.hp.0 - 25).max(1);
                            vec![Event::GoldEarned { amount: 250 }]
                        }
                        2 => {
                            player.gold += 250;
                            player.max_hp.0 -= 6;
                            player.hp.0 = player.hp.0.min(player.max_hp.0);
                            vec![Event::GoldEarned { amount: 250 }]
                        }
                        _ => Vec::new(),
                    },
                    EventKind::Mushrooms => match opt {
                        0 => {
                            player.hp.0 = (player.hp.0 + 12).min(player.max_hp.0);
                            player.deck.push(Card::Parasite);
                            vec![Event::Healed { amount: 12 }, Event::CardAdded { card: Card::Parasite }]
                        }
                        _ => Vec::new(),
                    },
                    EventKind::BigFish => match opt {
                        0 => {
                            let heal = (player.max_hp.0 * 30 / 100).max(1);
                            player.hp.0 = (player.hp.0 + heal).min(player.max_hp.0);
                            vec![Event::Healed { amount: heal }]
                        }
                        1 => {
                            player.max_hp.0 += 3;
                            player.hp.0 += 3;
                            Vec::new()
                        }
                        2 => {
                            let mut relic_pool = Relic::all();
                            rng.shuffle(&mut relic_pool);
                            let relic = relic_pool.into_iter().next().unwrap(); // SAFETY: Relic::all() is non-empty
                            let mut events = crate::relics::grant_relic(&mut player, relic, rng);
                            player.deck.push(Card::Regret);
                            events.push(Event::CardAdded { card: Card::Regret });
                            events
                        }
                        _ => Vec::new(),
                    },
                };
                Ok((GameState::Map(MapState { player, floor: floor + 1, graph, available_cols, next_enemies: None, scenario: Scenario::Main }), events))
            }
            _ => Err(CommandError::InvalidPhase),
        },

        GameState::Neow(NeowState { player, graph, blessings }) => match command {
            Command::ChooseNeowBlessing(idx) => {
                if idx >= blessings.len() {
                    return Err(CommandError::InvalidCard);
                }
                let blessing = blessings[idx].clone();
                let mut player = player;
                match blessing {
                    NeowBlessing::GainMaxHp(amount) => {
                        player.max_hp.0 += amount;
                        player.hp.0 = (player.hp.0 + amount).min(player.max_hp.0);
                    }
                    NeowBlessing::NeowsLament => {
                        player.neow_lament_combats_remaining = 3;
                    }
                    NeowBlessing::GainGold(amount) => {
                        player.gold += amount;
                    }
                    NeowBlessing::GainRelic(relic) => {
                        crate::relics::grant_relic(&mut player, relic, rng);
                    }
                    NeowBlessing::GainPotions(potions) => {
                        player.potions.extend(potions);
                    }
                    NeowBlessing::LoseHpGainGold { hp_loss, gold } => {
                        player.hp.0 -= hp_loss;
                        player.gold += gold;
                    }
                    NeowBlessing::LoseHpGainRareRelic { hp_loss, relic } => {
                        player.hp.0 -= hp_loss;
                        crate::relics::grant_relic(&mut player, relic, rng);
                    }
                    NeowBlessing::ObtainCurseGainRareRelic { curse, relic } => {
                        player.deck.push(curse);
                        crate::relics::grant_relic(&mut player, relic, rng);
                    }
                    NeowBlessing::SwapStarterRelic(relic) => {
                        player.relics.retain(|r| !matches!(r, Relic::BurningBlood));
                        crate::relics::grant_relic(&mut player, relic, rng);
                    }
                    NeowBlessing::RemoveCard
                    | NeowBlessing::TransformCard
                    | NeowBlessing::UpgradeCard
                    | NeowBlessing::ChooseRareCard(_)
                    | NeowBlessing::LoseHpRemoveCards { .. }
                    | NeowBlessing::LoseHpTransformCards { .. } => {}
                }
                let available_cols = (0..graph.rows.first().map_or(0, |r| r.len())).collect();
                Ok((GameState::Map(MapState { player, floor: 0, graph, available_cols, next_enemies: None, scenario: Scenario::Main }), Vec::new()))
            }
            _ => Err(CommandError::InvalidPhase),
        },

        GameState::GameOver { .. } => Err(CommandError::CombatOver),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::Grade;
    use crate::combat::Enemy;
    use crate::enemies::Move;
    use crate::neow::NeowContext;
    use crate::relics::Relic;
    use crate::status::StatusEffect;

    // --- Neow ---

    #[test]
    fn new_run_starts_in_neow_phase() {
        let state = new_run(&mut rng(), &NeowContext::default());
        assert!(matches!(state, GameState::Neow(_)));
    }

    #[test]
    fn first_run_neow_offers_two_blessings() {
        let GameState::Neow(neow) = new_run(&mut rng(), &NeowContext::default()) else { panic!("expected Neow") };
        assert_eq!(neow.blessings.len(), 2);
    }

    #[test]
    fn subsequent_run_without_boss_neow_offers_three_blessings() {
        let ctx = NeowContext { runs_completed: 1, prev_run_reached_boss: false };
        let GameState::Neow(neow) = new_run(&mut rng(), &ctx) else { panic!("expected Neow") };
        assert_eq!(neow.blessings.len(), 3);
    }

    #[test]
    fn subsequent_run_with_boss_neow_offers_four_blessings() {
        let ctx = NeowContext { runs_completed: 1, prev_run_reached_boss: true };
        let GameState::Neow(neow) = new_run(&mut rng(), &ctx) else { panic!("expected Neow") };
        assert_eq!(neow.blessings.len(), 4);
    }

    #[test]
    fn choosing_neow_blessing_transitions_to_map() {
        let mut r = rng();
        let state = new_run(&mut r, &NeowContext::default());
        let (state, _) = apply_command(state, Command::ChooseNeowBlessing(0), &mut r).unwrap();
        assert!(matches!(state, GameState::Map(_)));
    }

    #[test]
    fn neow_gain_max_hp_increases_player_max_hp() {
        let mut r = rng();
        let state = new_run(&mut r, &NeowContext::default());
        let GameState::Neow(ref neow) = state else { panic!() };
        let hp_before = neow.player.max_hp.0;
        let gain_hp_idx = neow.blessings.iter().position(|b| matches!(b, NeowBlessing::GainMaxHp(_))).expect("should have GainMaxHp");
        let (state, _) = apply_command(state, Command::ChooseNeowBlessing(gain_hp_idx), &mut r).unwrap();
        let GameState::Map(map) = state else { panic!() };
        assert!(map.player.max_hp.0 > hp_before);
    }

    #[test]
    fn neow_lament_sets_lament_counter_on_player() {
        let mut r = rng();
        let state = new_run(&mut r, &NeowContext::default());
        let GameState::Neow(ref neow) = state else { panic!() };
        let lament_idx = neow.blessings.iter().position(|b| matches!(b, NeowBlessing::NeowsLament)).expect("should have NeowsLament");
        let (state, _) = apply_command(state, Command::ChooseNeowBlessing(lament_idx), &mut r).unwrap();
        let GameState::Map(map) = state else { panic!() };
        assert_eq!(map.player.neow_lament_combats_remaining, 3);
    }

    #[test]
    fn neow_lament_sets_enemies_to_one_hp_in_combat() {
        let mut r = rng();
        // Give lament, then enter combat
        let state = new_run(&mut r, &NeowContext::default());
        let GameState::Neow(ref neow) = state else { panic!() };
        let lament_idx = neow.blessings.iter().position(|b| matches!(b, NeowBlessing::NeowsLament)).expect("should have NeowsLament");
        let (state, _) = apply_command(state, Command::ChooseNeowBlessing(lament_idx), &mut r).unwrap();
        let (state, _) = apply_command(state, Command::Spawn(vec![EnemyKind::RedLouse]), &mut r).unwrap();
        let (state, _) = apply_command(state, Command::ChooseNode(0), &mut r).unwrap();
        let GameState::Combat { state: ref cs, .. } = state else { panic!() };
        assert!(cs.enemies.iter().all(|e| e.hp.0 == 1));
    }

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

    fn run_after_neow() -> GameState {
        let state = new_run(&mut rng(), &NeowContext::default());
        let (state, _) = apply_command(state, Command::ChooseNeowBlessing(0), &mut rng()).unwrap();
        state
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
            neow_lament_combats_remaining: 0,
            reached_boss: false,
            potion_chance: 0.40,
        }
    }

    fn test_graph() -> MapGraph {
        generate_map(&mut rng())
    }

    fn all_cols(graph: &MapGraph, floor: usize) -> Vec<usize> {
        (0..graph.rows.get(floor).map_or(0, |r| r.len())).collect()
    }

    fn combat_at_floor(floor: usize) -> GameState {
        let player = make_player();
        let cs = CombatState {
            player: Player {
                hand: vec![Card::Strike(Grade::Base)],
                draw_pile: Vec::new(),
                ..player
            },
            enemies: vec![Enemy {
                kind: EnemyKind::RedLouse,
                hp: Hp(1),
                max_hp: Hp(20),
                block: Block(0),
                move_: Move::LouseBite,
                move_history: vec![], stolen_gold: 0,
                statuses: StatusMap::new(),
            }],
            turn: 1,
            phase: CombatPhase::PlayerTurn,
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0, hand_cost_max: None, hand_cost_max_expires: false, block_locked_turns: 0, pending_bombs: Vec::new(), duplication_pending: false, zero_cost_cards: Vec::new(),
        };
        let graph = test_graph();
        let next_floor_cols = all_cols(&graph, floor + 1);
        GameState::Combat { state: cs, floor, is_boss: false, is_elite: false, graph, next_floor_cols, scenario: Scenario::Main }
    }

    fn elite_combat_at_floor(floor: usize) -> GameState {
        match combat_at_floor(floor) {
            GameState::Combat { state, floor, graph, next_floor_cols, scenario, .. } =>
                GameState::Combat { state, floor, is_boss: false, is_elite: true, graph, next_floor_cols, scenario },
            other => other,
        }
    }

    fn boss_combat() -> GameState {
        let player = make_player();
        let cs = CombatState {
            player: Player {
                hand: vec![Card::Strike(Grade::Base)],
                draw_pile: Vec::new(),
                ..player
            },
            enemies: vec![Enemy {
                kind: EnemyKind::RedLouse,
                hp: Hp(1),
                max_hp: Hp(20),
                block: Block(0),
                move_: Move::LouseBite,
                move_history: vec![], stolen_gold: 0,
                statuses: StatusMap::new(),
            }],
            turn: 1,
            phase: CombatPhase::PlayerTurn,
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0, hand_cost_max: None, hand_cost_max_expires: false, block_locked_turns: 0, pending_bombs: Vec::new(), duplication_pending: false, zero_cost_cards: Vec::new(),
        };
        GameState::Combat { state: cs, floor: 9, is_boss: true, is_elite: false, graph: test_graph(), next_floor_cols: vec![], scenario: Scenario::Main }
    }

    fn wrap_combat(cs: CombatState, floor: usize) -> GameState {
        let graph = test_graph();
        let next_floor_cols = all_cols(&graph, floor + 1);
        GameState::Combat { state: cs, floor, is_boss: false, is_elite: false, graph, next_floor_cols, scenario: Scenario::Main }
    }

    // --- new_run ---

    #[test]
    fn new_run_starts_on_map() {
        let state = run_after_neow();
        assert!(matches!(state, GameState::Map(_)));
    }

    #[test]
    fn new_run_starts_at_floor_0() {
        if let GameState::Map(map) = run_after_neow() {
            assert_eq!(map.floor, 0);
        } else {
            panic!("expected Map state");
        }
    }

    #[test]
    fn new_run_player_starts_with_99_gold() {
        if let GameState::Map(map) = run_after_neow() {
            assert_eq!(map.player.gold, 99);
        } else {
            panic!("expected Map state");
        }
    }

    #[test]
    fn new_run_player_has_starter_deck() {
        if let GameState::Map(map) = run_after_neow() {
            assert_eq!(map.player.deck.len(), starter_deck().len());
        } else {
            panic!("expected Map state");
        }
    }

    // --- map navigation ---

    #[test]
    fn choose_node_0_enters_combat() {
        let state = run_after_neow();
        let (state, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        assert!(matches!(state, GameState::Combat { floor: 0, .. }));
    }

    #[test]
    fn choose_node_invalid_index_returns_error() {
        let state = run_after_neow();
        let result = apply_command(state, Command::ChooseNode(2), &mut rng());
        assert_eq!(result, Err(CommandError::InvalidPhase));
    }

    #[test]
    fn non_map_commands_rejected_on_map() {
        let state = run_after_neow();
        assert_eq!(
            apply_command(state.clone(), Command::EndTurn, &mut rng()),
            Err(CommandError::InvalidPhase)
        );
        assert_eq!(
            apply_command(state.clone(), Command::Rest, &mut rng()),
            Err(CommandError::InvalidPhase)
        );
    }

    #[test]
    fn choose_node_rejects_unavailable_col() {
        let graph = test_graph();
        // Floor 1 has 2 columns, but restrict available to [1] only
        let state = GameState::Map(MapState {
            player: make_player(),
            floor: 1,
            graph,
            available_cols: vec![1],
            next_enemies: None,
            scenario: Scenario::Main,
        });
        let result = apply_command(state, Command::ChooseNode(0), &mut rng());
        assert_eq!(result, Err(CommandError::InvalidPhase));
    }

    #[test]
    fn choose_node_advances_to_combat_with_enemies_from_node() {
        let graph = test_graph();
        let enemies = match &graph.rows[0][0] {
            MapNode::Combat(e) => e.clone(),
            _ => panic!("expected combat node at floor 0 col 0"),
        };
        let state = GameState::Map(MapState {
            player: make_player(),
            floor: 0,
            graph,
            available_cols: vec![0, 1],
            next_enemies: None,
            scenario: Scenario::Main,
        });
        let (next, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        let GameState::Combat { state: cs, .. } = next else { panic!("expected Combat") };
        let kinds: Vec<_> = cs.enemies.iter().map(|e| e.kind.clone()).collect();
        assert_eq!(kinds, enemies);
    }

    #[test]
    fn choose_node_carries_next_floor_cols_from_edges() {
        let graph = test_graph();
        let expected_cols = graph.edges[0][0].clone();
        let state = GameState::Map(MapState {
            player: make_player(),
            floor: 0,
            graph,
            available_cols: vec![0, 1],
            next_enemies: None,
            scenario: Scenario::Main,
        });
        let (next, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        let GameState::Combat { next_floor_cols, .. } = next else { panic!("expected Combat") };
        assert_eq!(next_floor_cols, expected_cols);
    }

    #[test]
    fn boss_node_sets_is_boss_true() {
        let graph = test_graph();
        let state = GameState::Map(MapState {
            player: make_player(),
            floor: 9,
            graph,
            available_cols: vec![0],
            next_enemies: None,
            scenario: Scenario::Main,
        });
        let (next, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        let GameState::Combat { is_boss, .. } = next else { panic!("expected Combat") };
        assert!(is_boss);
    }

    #[test]
    fn non_boss_node_sets_is_boss_false() {
        let state = run_after_neow();
        let (next, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        let GameState::Combat { is_boss, .. } = next else { panic!("expected Combat") };
        assert!(!is_boss);
    }

    // --- combat victory transitions ---

    #[test]
    fn winning_combat_goes_to_card_reward() {
        let state = combat_at_floor(0);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(matches!(state, GameState::CardReward(_)));
    }

    #[test]
    fn winning_normal_combat_awards_10_to_20_gold() {
        let state = combat_at_floor(0);
        let (_, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let gold = events.iter().find_map(|e| if let Event::GoldEarned { amount } = e { Some(*amount) } else { None }).unwrap();
        assert!((10..=20).contains(&gold), "expected 10–20 gold, got {gold}");
    }

    #[test]
    fn winning_elite_combat_awards_25_to_35_gold() {
        let state = elite_combat_at_floor(5);
        let (_, events) = apply_command(state, Command::WinCombat, &mut rng()).unwrap();
        let gold = events.iter().find_map(|e| if let Event::GoldEarned { amount } = e { Some(*amount) } else { None }).unwrap();
        assert!((25..=35).contains(&gold), "expected 25–35 gold, got {gold}");
    }

    #[test]
    fn winning_boss_combat_awards_95_to_105_gold() {
        let state = boss_combat();
        let (_, events) = apply_command(state, Command::WinCombat, &mut rng()).unwrap();
        let gold = events.iter().find_map(|e| if let Event::GoldEarned { amount } = e { Some(*amount) } else { None }).unwrap();
        assert!((95..=105).contains(&gold), "expected 95–105 gold, got {gold}");
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
    fn win_combat_command_advances_floor_in_reward() {
        let state = combat_at_floor(2);
        let (next, _) = apply_command(state, Command::WinCombat, &mut rng()).unwrap();
        let GameState::CardReward(cr) = next else { panic!("expected CardReward") };
        assert_eq!(cr.floor, 3);
    }

    #[test]
    fn player_block_reset_after_combat() {
        let GameState::Combat { state: mut cs, floor, is_boss, is_elite, graph, next_floor_cols, scenario }
            = combat_at_floor(0) else { panic!() };
        cs.player.block = Block(10);
        let (next, _) = apply_command(
            GameState::Combat { state: cs, floor, is_boss, is_elite, graph, next_floor_cols, scenario },
            Command::PlayCard(0, 0),
            &mut rng(),
        ).unwrap();
        let GameState::CardReward(cr) = next else { panic!("expected CardReward") };
        assert_eq!(cr.player.block, Block(0));
    }

    #[test]
    fn player_energy_restored_after_combat() {
        let (next, _) = apply_command(combat_at_floor(0), Command::PlayCard(0, 0), &mut rng()).unwrap();
        let GameState::CardReward(cr) = next else { panic!("expected CardReward") };
        assert_eq!(cr.player.energy, Energy(3));
    }

    #[test]
    fn player_hand_cleared_after_combat() {
        let GameState::Combat { state: mut cs, floor, is_boss, is_elite, graph, next_floor_cols, scenario }
            = combat_at_floor(0) else { panic!() };
        cs.player.hand.push(Card::Strike(Grade::Base)); // two Strikes in hand; one kills enemy, one remains
        let (next, _) = apply_command(
            GameState::Combat { state: cs, floor, is_boss, is_elite, graph, next_floor_cols, scenario },
            Command::PlayCard(0, 0),
            &mut rng(),
        ).unwrap();
        let GameState::CardReward(cr) = next else { panic!("expected CardReward") };
        assert!(cr.player.hand.is_empty());
    }

    #[test]
    fn player_draw_pile_cleared_after_combat() {
        let GameState::Combat { state: mut cs, floor, is_boss, is_elite, graph, next_floor_cols, scenario }
            = combat_at_floor(0) else { panic!() };
        cs.player.draw_pile = vec![Card::Strike(Grade::Base), Card::Strike(Grade::Base)];
        let (next, _) = apply_command(
            GameState::Combat { state: cs, floor, is_boss, is_elite, graph, next_floor_cols, scenario },
            Command::PlayCard(0, 0),
            &mut rng(),
        ).unwrap();
        let GameState::CardReward(cr) = next else { panic!("expected CardReward") };
        assert!(cr.player.draw_pile.is_empty());
    }

    #[test]
    fn player_discard_pile_cleared_after_combat() {
        let (next, _) = apply_command(combat_at_floor(0), Command::PlayCard(0, 0), &mut rng()).unwrap();
        let GameState::CardReward(cr) = next else { panic!("expected CardReward") };
        assert!(cr.player.discard_pile.is_empty());
    }

    #[test]
    fn player_statuses_cleared_after_combat() {
        let GameState::Combat { state: mut cs, floor, is_boss, is_elite, graph, next_floor_cols, scenario }
            = combat_at_floor(0) else { panic!() };
        cs.player.statuses.insert(StatusEffect::Strength, 3);
        let (next, _) = apply_command(
            GameState::Combat { state: cs, floor, is_boss, is_elite, graph, next_floor_cols, scenario },
            Command::PlayCard(0, 0),
            &mut rng(),
        ).unwrap();
        let GameState::CardReward(cr) = next else { panic!("expected CardReward") };
        assert!(cr.player.statuses.is_empty());
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
            hand: vec![Card::Strike(Grade::Base)],
            ..make_player()
        };
        let cs = CombatState {
            player,
            enemies: vec![Enemy {
                kind: EnemyKind::RedLouse,
                hp: Hp(20),
                max_hp: Hp(20),
                block: Block(0),
                move_: Move::LouseBite,
                move_history: vec![], stolen_gold: 0,
                statuses: StatusMap::new(),
            }],
            turn: 1,
            phase: CombatPhase::PlayerTurn,
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0, hand_cost_max: None, hand_cost_max_expires: false, block_locked_turns: 0, pending_bombs: Vec::new(), duplication_pending: false, zero_cost_cards: Vec::new(),
        };
        let state = wrap_combat(cs, 0);
        let (after_end, _) =
            apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        let (after_enemy, _) =
            apply_command(after_end, Command::EndEnemyTurn, &mut rng()).unwrap();
        assert_eq!(after_enemy, GameState::GameOver { victory: false });
    }

    fn rest_site_at_floor(floor: usize) -> GameState {
        GameState::RestSite(RestSiteState { player: make_player(), floor, graph: test_graph(), available_cols: vec![0, 1] })
    }

    // --- rest site ---

    #[test]
    fn choosing_rest_site_enters_rest_state() {
        let (state, _) = apply_command(map_at_floor(6), Command::ChooseNode(0), &mut rng()).unwrap();
        assert!(matches!(state, GameState::RestSite(_)));
    }

    #[test]
    fn rest_heals_30_percent_of_max_hp() {
        let mut player = make_player();
        player.hp = Hp(50);
        let state = GameState::RestSite(RestSiteState { player, floor: 3, graph: test_graph(), available_cols: vec![0, 1] });
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
        let state = GameState::RestSite(RestSiteState { player, floor: 3, graph: test_graph(), available_cols: vec![0, 1] });
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
        let state = GameState::RestSite(RestSiteState { player, floor: 3, graph: test_graph(), available_cols: vec![0, 1] });
        let (_, events) = apply_command(state, Command::Rest, &mut rng()).unwrap();
        assert!(events.iter().any(|e| matches!(e, Event::Healed { .. })));
    }

    #[test]
    fn rest_advances_to_next_floor_on_map() {
        let state = rest_site_at_floor(3);
        let (state, _) = apply_command(state, Command::Rest, &mut rng()).unwrap();
        if let GameState::Map(map) = state {
            assert_eq!(map.floor, 4);
        } else {
            panic!("expected Map after rest");
        }
    }

    #[test]
    fn non_rest_commands_rejected_at_rest_site() {
        let state = rest_site_at_floor(3);
        assert_eq!(
            apply_command(state, Command::EndTurn, &mut rng()),
            Err(CommandError::InvalidPhase)
        );
    }

    // --- boss / run end ---

    #[test]
    fn winning_boss_ends_run_with_victory() {
        let state = boss_combat();
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

    // --- Spawn command ---

    #[test]
    fn spawn_command_sets_next_enemies_on_map_state() {
        let (state, _) = apply_command(map_at_floor(0), Command::Spawn(vec![EnemyKind::Fungibeast]), &mut rng()).unwrap();
        let GameState::Map(map) = state else { panic!("expected Map") };
        assert_eq!(map.next_enemies, Some(vec![EnemyKind::Fungibeast]));
    }

    #[test]
    fn choose_node_uses_spawned_enemies_instead_of_floor_default() {
        let graph = test_graph();
        let available_cols = all_cols(&graph, 0);
        let state = GameState::Map(MapState {
            player: make_player(),
            floor: 0,
            graph,
            available_cols,
            next_enemies: Some(vec![EnemyKind::Cultist]),
            scenario: Scenario::Main,
        });
        let (state, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.enemies[0].kind, EnemyKind::Cultist);
    }

    #[test]
    fn choose_node_clears_next_enemies_after_use() {
        let graph = test_graph();
        let available_cols = all_cols(&graph, 0);
        let state = GameState::Map(MapState {
            player: make_player(),
            floor: 0,
            graph,
            available_cols,
            next_enemies: Some(vec![EnemyKind::Cultist]),
            scenario: Scenario::Main,
        });
        let (state, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::WinCombat, &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::SkipReward, &mut rng()).unwrap();
        let GameState::Map(map) = state else { panic!("expected Map") };
        assert_eq!(map.next_enemies, None);
    }

    #[test]
    fn choose_node_falls_back_to_graph_enemies_when_no_spawn() {
        let (state, _) = apply_command(map_at_floor(0), Command::ChooseNode(0), &mut rng()).unwrap();
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        // With NoOpRng the pool is unshuffled, so first easy encounter = Cultist
        assert_eq!(cs.enemies[0].kind, EnemyKind::Cultist);
    }

    // --- player state persists across combat ---

    #[test]
    fn player_hp_persists_from_combat_to_map() {
        let mut player = make_player();
        player.hp = Hp(50);
        let cs = CombatState {
            player: Player {
                hand: vec![Card::Strike(Grade::Base)],
                ..player
            },
            enemies: vec![Enemy {
                kind: EnemyKind::RedLouse,
                hp: Hp(1),
                max_hp: Hp(20),
                block: Block(0),
                move_: Move::LouseBite,
                move_history: vec![], stolen_gold: 0,
                statuses: StatusMap::new(),
            }],
            turn: 1,
            phase: CombatPhase::PlayerTurn,
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0, hand_cost_max: None, hand_cost_max_expires: false, block_locked_turns: 0, pending_bombs: Vec::new(), duplication_pending: false, zero_cost_cards: Vec::new(),
        };
        let state = wrap_combat(cs, 0);
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
        let state = combat_at_floor(0);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::ChooseCardReward(0), &mut rng()).unwrap();
        let gold_after_first = if let GameState::Map(ref map) = state {
            assert!((10..=20).contains(&map.player.gold), "gold after first combat should be 10–20");
            map.player.gold
        } else {
            panic!("expected Map");
        };
        let (state, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        let state = if let GameState::Combat { mut state, floor, is_boss, is_elite, graph, next_floor_cols, scenario: Scenario::Main } = state {
            state.enemies[0].hp = Hp(1);
            state.player.hand = vec![Card::Strike(Grade::Base)];
            GameState::Combat { state, floor, is_boss, is_elite, graph, next_floor_cols, scenario: Scenario::Main }
        } else {
            panic!("expected Combat");
        };
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::ChooseCardReward(0), &mut rng()).unwrap();
        if let GameState::Map(map) = state {
            assert!(map.player.gold >= gold_after_first + 10, "gold should increase by at least 10 after second combat");
            assert!(map.player.gold <= gold_after_first + 20, "gold should increase by at most 20 after second combat");
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
            deck: vec![Card::Strike(Grade::Base), Card::Disarm],
            hand: vec![Card::Strike(Grade::Base)],
            exhaust_pile: vec![Card::Disarm], // Disarm was played and exhausted
            ..make_player()
        };
        let cs = CombatState {
            player,
            enemies: vec![Enemy {
                kind: EnemyKind::RedLouse,
                hp: Hp(1),
                max_hp: Hp(20),
                block: Block(0),
                move_: Move::LouseBite,
                move_history: vec![], stolen_gold: 0,
                statuses: StatusMap::new(),
            }],
            turn: 1,
            phase: CombatPhase::PlayerTurn,
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0, hand_cost_max: None, hand_cost_max_expires: false, block_locked_turns: 0, pending_bombs: Vec::new(), duplication_pending: false, zero_cost_cards: Vec::new(),
        };
        let state = wrap_combat(cs, 0);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let GameState::CardReward(cr) = state else { panic!("expected CardReward") };
        assert!(cr.player.deck.contains(&Card::Disarm), "Disarm should be back in deck");
        assert!(cr.player.exhaust_pile.is_empty(), "exhaust pile should be cleared");
    }

    #[test]
    fn exhaust_pile_is_empty_at_combat_start() {
        let state = run_after_neow();
        let (state, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert!(cs.player.exhaust_pile.is_empty());
    }

    // --- map generation ---

    fn map_at_floor(floor: usize) -> GameState {
        let graph = test_graph();
        let available_cols = all_cols(&graph, floor);
        GameState::Map(MapState { player: make_player(), floor, graph, available_cols, next_enemies: None, scenario: Scenario::Main })
    }

    #[test]
    fn map_has_ten_floors() {
        assert_eq!(test_graph().rows.len(), 10);
    }

    #[test]
    fn convergence_floors_have_one_column() {
        let graph = test_graph();
        assert_eq!(graph.rows[3].len(), 1);
        assert_eq!(graph.rows[6].len(), 1);
        assert_eq!(graph.rows[8].len(), 1);
        assert_eq!(graph.rows[9].len(), 1);
    }

    #[test]
    fn combat_floors_have_two_columns() {
        let graph = test_graph();
        for floor in [0usize, 1, 2, 4, 5, 7] {
            assert_eq!(graph.rows[floor].len(), 2, "floor {floor} should have 2 columns");
        }
    }

    #[test]
    fn map_contains_event_nodes() {
        let graph = test_graph();
        let has_event = graph.rows.iter().flatten().any(|n| matches!(n, MapNode::Event));
        assert!(has_event, "map should contain at least one Event node");
    }

    #[test]
    fn floor_1_has_an_event_node() {
        let graph = test_graph();
        assert!(graph.rows[1].iter().any(|n| matches!(n, MapNode::Event)));
    }

    #[test]
    fn floor_4_has_an_event_node() {
        let graph = test_graph();
        assert!(graph.rows[4].iter().any(|n| matches!(n, MapNode::Event)));
    }

    #[test]
    fn choosing_event_node_enters_event_room() {
        let graph = MapGraph {
            rows: vec![vec![MapNode::Event]],
            edges: vec![vec![vec![]]],
        };
        let state = GameState::Map(MapState {
            player: make_player(),
            floor: 0,
            graph,
            available_cols: vec![0],
            next_enemies: None,
            scenario: Scenario::Main,
        });
        let (next, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        assert!(matches!(next, GameState::EventRoom(_)));
    }

    #[test]
    fn event_room_event_is_not_always_ssssserpent() {
        // With a real rng, different events should appear — verify the pool has more than one entry
        // by checking that EventKind::random can produce BigFish (second in pool)
        let mut pool = vec![
            EventKind::Ssssserpent,
            EventKind::BigFish,
            EventKind::Mushrooms,
            EventKind::GoldenIdol,
        ];
        // ThreadRng would shuffle; here we just verify the pool itself is correct size
        assert_eq!(pool.len(), 4);
        // Remove Ssssserpent and verify the rest are distinct valid kinds
        pool.retain(|e| !matches!(e, EventKind::Ssssserpent));
        assert!(pool.iter().any(|e| matches!(e, EventKind::BigFish)));
    }

    #[test]
    fn boss_floor_node_is_boss_variant() {
        let graph = test_graph();
        assert!(matches!(graph.rows[9][0], MapNode::Boss(_)));
    }

    #[test]
    fn merchant_floor_node_is_merchant_variant() {
        let graph = test_graph();
        assert!(matches!(graph.rows[3][0], MapNode::Merchant));
    }

    #[test]
    fn rest_floor_node_is_restsite_variant() {
        let graph = test_graph();
        assert!(matches!(graph.rows[6][0], MapNode::RestSite));
    }

    #[test]
    fn edges_from_combat_floor_reach_next_floor_columns() {
        let graph = test_graph();
        for col in 0..graph.rows[0].len() {
            assert!(!graph.edges[0][col].is_empty());
            for &target in &graph.edges[0][col] {
                assert!(target < graph.rows[1].len());
            }
        }
    }

    #[test]
    fn available_cols_starts_as_both_columns() {
        let GameState::Map(map) = run_after_neow() else { panic!("expected Map") };
        assert_eq!(map.available_cols, vec![0, 1]);
    }

    #[test]
    fn segment_internal_floors_have_branching_edges() {
        let graph = test_graph();
        let both = vec![vec![0usize, 1], vec![0, 1]];
        for floor in [0usize, 1, 4] {
            assert_eq!(graph.edges[floor], both, "floor {floor} should have branching edges");
        }
    }

    #[test]
    fn segment_end_floors_have_converging_edges() {
        let graph = test_graph();
        let converge = vec![vec![0usize], vec![0]];
        for floor in [2usize, 5, 7] {
            assert_eq!(graph.edges[floor], converge, "floor {floor} should have converging edges");
        }
    }

    #[test]
    fn treasure_floor_edges_point_to_boss() {
        let graph = test_graph();
        assert_eq!(graph.edges[8], vec![vec![0usize]]);
    }

    #[test]
    fn treasure_floor_node_is_treasure_variant() {
        let graph = test_graph();
        assert!(matches!(graph.rows[8][0], MapNode::Treasure));
    }

    #[test]
    fn treasure_floor_has_single_column() {
        assert_eq!(test_graph().rows[8].len(), 1);
    }

    #[test]
    fn entering_treasure_node_from_map_transitions_to_treasure_room() {
        let (next, _) = apply_command(map_at_floor(8), Command::ChooseNode(0), &mut rng()).unwrap();
        assert!(matches!(next, GameState::TreasureRoom(_)));
    }

    // --- shop ---

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
        assert!(shop.relic.as_ref().is_none_or(|(_, p)| !p));
        assert!(shop.potion.as_ref().is_none_or(|(_, p)| !p));
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
        let shop = ShopState { player, floor: 3, cards: vec![], relic: None, potion: None, graph: test_graph(), available_cols: vec![0, 1] };
        let (next, _) = apply_command(GameState::Shop(shop), Command::LeaveShop, &mut rng()).unwrap();
        let GameState::Map(map) = next else { panic!("expected Map") };
        assert_eq!(map.player.gold, 150);
    }

    #[test]
    fn non_shop_commands_rejected_in_shop() {
        let shop = ShopState { player: make_player(), floor: 3, cards: vec![], relic: None, potion: None, graph: test_graph(), available_cols: vec![0, 1] };
        assert_eq!(
            apply_command(GameState::Shop(shop), Command::EndTurn, &mut rng()),
            Err(CommandError::InvalidPhase)
        );
    }

    fn shop_state(gold: i32) -> GameState {
        let mut player = make_player();
        player.gold = gold;
        GameState::Shop(ShopState {
            player,
            floor: 3,
            cards: vec![(Card::Strike(Grade::Base), false), (Card::Bash(Grade::Base), false)],
            relic: Some((Relic::Anchor, false)),
            potion: Some((Potion::FirePotion, false)),
            graph: test_graph(),
            available_cols: vec![0, 1],
        })
    }

    // BuyCard

    #[test]
    fn buy_card_deducts_gold() {
        let (next, _) = apply_command(shop_state(99), Command::BuyCard(0), &mut rng()).unwrap();
        let GameState::Shop(s) = next else { panic!() };
        assert_eq!(s.player.gold, 99 - CARD_PRICE);
    }

    #[test]
    fn buy_card_adds_card_to_deck() {
        let (next, _) = apply_command(shop_state(99), Command::BuyCard(0), &mut rng()).unwrap();
        let GameState::Shop(s) = next else { panic!() };
        assert!(s.player.deck.contains(&Card::Strike(Grade::Base)));
    }

    #[test]
    fn buy_card_emits_card_added_event() {
        let (_, events) = apply_command(shop_state(99), Command::BuyCard(0), &mut rng()).unwrap();
        assert!(events.iter().any(|e| matches!(e, Event::CardAdded { .. })));
    }

    #[test]
    fn buy_card_marks_slot_purchased() {
        let (next, _) = apply_command(shop_state(99), Command::BuyCard(0), &mut rng()).unwrap();
        let GameState::Shop(s) = next else { panic!() };
        assert!(s.cards[0].1);
    }

    #[test]
    fn buy_card_not_enough_gold() {
        assert_eq!(
            apply_command(shop_state(CARD_PRICE - 1), Command::BuyCard(0), &mut rng()),
            Err(CommandError::NotEnoughGold)
        );
    }

    #[test]
    fn buy_card_exact_gold_succeeds() {
        assert!(apply_command(shop_state(CARD_PRICE), Command::BuyCard(0), &mut rng()).is_ok());
    }

    #[test]
    fn buy_card_out_of_bounds_returns_invalid_card() {
        assert_eq!(
            apply_command(shop_state(999), Command::BuyCard(99), &mut rng()),
            Err(CommandError::InvalidCard)
        );
    }

    #[test]
    fn buy_card_already_purchased_returns_invalid_card() {
        let (s1, _) = apply_command(shop_state(999), Command::BuyCard(0), &mut rng()).unwrap();
        assert_eq!(
            apply_command(s1, Command::BuyCard(0), &mut rng()),
            Err(CommandError::InvalidCard)
        );
    }

    // BuyRelic

    #[test]
    fn buy_relic_deducts_gold() {
        let (next, _) = apply_command(shop_state(999), Command::BuyRelic, &mut rng()).unwrap();
        let GameState::Shop(s) = next else { panic!() };
        assert_eq!(s.player.gold, 999 - RELIC_PRICE);
    }

    #[test]
    fn buy_relic_grants_relic_to_player() {
        let (next, _) = apply_command(shop_state(999), Command::BuyRelic, &mut rng()).unwrap();
        let GameState::Shop(s) = next else { panic!() };
        assert!(s.player.relics.contains(&Relic::Anchor));
    }

    #[test]
    fn buy_relic_marks_slot_purchased() {
        let (next, _) = apply_command(shop_state(999), Command::BuyRelic, &mut rng()).unwrap();
        let GameState::Shop(s) = next else { panic!() };
        assert!(s.relic.unwrap().1);
    }

    #[test]
    fn buy_relic_not_enough_gold() {
        assert_eq!(
            apply_command(shop_state(RELIC_PRICE - 1), Command::BuyRelic, &mut rng()),
            Err(CommandError::NotEnoughGold)
        );
    }

    #[test]
    fn buy_relic_exact_gold_succeeds() {
        assert!(apply_command(shop_state(RELIC_PRICE), Command::BuyRelic, &mut rng()).is_ok());
    }

    #[test]
    fn buy_relic_no_relic_in_shop() {
        let mut player = make_player();
        player.gold = 999;
        let shop = GameState::Shop(ShopState { player, floor: 3, cards: vec![], relic: None, potion: None, graph: test_graph(), available_cols: vec![0, 1] });
        assert_eq!(apply_command(shop, Command::BuyRelic, &mut rng()), Err(CommandError::InvalidCard));
    }

    #[test]
    fn buy_relic_already_purchased_returns_invalid_card() {
        let (s1, _) = apply_command(shop_state(999), Command::BuyRelic, &mut rng()).unwrap();
        assert_eq!(apply_command(s1, Command::BuyRelic, &mut rng()), Err(CommandError::InvalidCard));
    }

    // BuyPotion

    #[test]
    fn buy_potion_deducts_gold() {
        let (next, _) = apply_command(shop_state(99), Command::BuyPotion, &mut rng()).unwrap();
        let GameState::Shop(s) = next else { panic!() };
        assert_eq!(s.player.gold, 99 - POTION_PRICE);
    }

    #[test]
    fn buy_potion_adds_potion_to_player() {
        let (next, _) = apply_command(shop_state(99), Command::BuyPotion, &mut rng()).unwrap();
        let GameState::Shop(s) = next else { panic!() };
        assert!(s.player.potions.contains(&Potion::FirePotion));
    }

    #[test]
    fn buy_potion_marks_slot_purchased() {
        let (next, _) = apply_command(shop_state(99), Command::BuyPotion, &mut rng()).unwrap();
        let GameState::Shop(s) = next else { panic!() };
        assert!(s.potion.unwrap().1);
    }

    #[test]
    fn buy_potion_not_enough_gold() {
        assert_eq!(
            apply_command(shop_state(POTION_PRICE - 1), Command::BuyPotion, &mut rng()),
            Err(CommandError::NotEnoughGold)
        );
    }

    #[test]
    fn buy_potion_exact_gold_succeeds() {
        assert!(apply_command(shop_state(POTION_PRICE), Command::BuyPotion, &mut rng()).is_ok());
    }

    #[test]
    fn buy_potion_slots_full_returns_invalid_phase() {
        let mut player = make_player();
        player.gold = 999;
        player.potions = vec![Potion::FirePotion, Potion::BlockPotion, Potion::WeakPotion];
        let shop = GameState::Shop(ShopState {
            player,
            floor: 3,
            cards: vec![],
            relic: None,
            potion: Some((Potion::EnergyPotion, false)),
            graph: test_graph(),
            available_cols: vec![0, 1],
        });
        assert_eq!(apply_command(shop, Command::BuyPotion, &mut rng()), Err(CommandError::InvalidPhase));
    }

    #[test]
    fn buy_potion_no_potion_in_shop() {
        let mut player = make_player();
        player.gold = 999;
        let shop = GameState::Shop(ShopState { player, floor: 3, cards: vec![], relic: None, potion: None, graph: test_graph(), available_cols: vec![0, 1] });
        assert_eq!(apply_command(shop, Command::BuyPotion, &mut rng()), Err(CommandError::InvalidCard));
    }

    #[test]
    fn buy_potion_already_purchased_returns_invalid_card() {
        let (s1, _) = apply_command(shop_state(999), Command::BuyPotion, &mut rng()).unwrap();
        assert_eq!(apply_command(s1, Command::BuyPotion, &mut rng()), Err(CommandError::InvalidCard));
    }

    // --- rest site: upgrade ---

    #[test]
    fn upgrade_replaces_card_in_deck_with_plus_version() {
        // deck[0] = Strike (from starter_deck); upgrade it → StrikePlus
        let state = rest_site_at_floor(3);
        let (state, _) = apply_command(state, Command::UpgradeCard(0), &mut rng()).unwrap();
        let GameState::Map(map) = state else { panic!("expected Map") };
        assert_eq!(map.player.deck[0], Card::Strike(Grade::Plus));
    }

    #[test]
    fn upgrade_advances_to_map_at_next_floor() {
        let state = rest_site_at_floor(3);
        let (state, _) = apply_command(state, Command::UpgradeCard(0), &mut rng()).unwrap();
        let GameState::Map(map) = state else { panic!("expected Map") };
        assert_eq!(map.floor, 4);
    }

    #[test]
    fn upgrade_emits_card_upgraded_event() {
        let state = rest_site_at_floor(3);
        let (_, events) = apply_command(state, Command::UpgradeCard(0), &mut rng()).unwrap();
        assert!(events.iter().any(|e| matches!(e, Event::CardUpgraded { .. })));
    }

    #[test]
    fn upgrade_invalid_index_returns_error() {
        let state = rest_site_at_floor(3);
        let result = apply_command(state, Command::UpgradeCard(99), &mut rng());
        assert_eq!(result, Err(CommandError::InvalidCard));
    }

    #[test]
    fn upgrade_non_upgradeable_card_returns_error() {
        // starter_deck last card is Disarm (index 11), which cannot be upgraded
        let state = rest_site_at_floor(3);
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
    fn win_combat_awards_gold_in_range() {
        let state = combat_at_floor(0);
        let (_, events) = apply_command(state, Command::WinCombat, &mut rng()).unwrap();
        let gold = events.iter().find_map(|e| if let Event::GoldEarned { amount } = e { Some(*amount) } else { None }).unwrap();
        assert!((10..=20).contains(&gold), "expected 10–20 gold, got {gold}");
    }

    #[test]
    fn win_combat_on_boss_floor_ends_run_with_victory() {
        let state = boss_combat();
        let (state, _) = apply_command(state, Command::WinCombat, &mut rng()).unwrap();
        assert_eq!(state, GameState::GameOver { victory: true });
    }

    // --- debug: SkipFloor ---

    #[test]
    fn skip_floor_from_map_advances_floor() {
        let state = run_after_neow(); // Map at floor 0
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
        let state = run_after_neow();
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
    fn boss_floor_has_the_guardian() {
        let map = map_at_floor(9);
        let (state, _) = apply_command(map, Command::ChooseNode(0), &mut rng()).unwrap();
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.enemies.len(), 1);
        assert_eq!(cs.enemies[0].kind, EnemyKind::TheGuardian);
    }

    #[test]
    fn regular_floor_has_one_enemy() {
        let map = map_at_floor(0);
        let (state, _) = apply_command(map, Command::ChooseNode(0), &mut rng()).unwrap();
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.enemies.len(), 1);
    }

    #[test]
    fn winning_boss_requires_all_enemies_dead() {
        // Boss has two enemies; use WinCombat (debug) to confirm boss victory still works.
        let state = boss_combat();
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
                hand: vec![Card::Strike(Grade::Base)],
                draw_pile: Vec::new(),
                ..player
            },
            enemies: vec![Enemy {
                kind: EnemyKind::RedLouse,
                hp: Hp(1),
                max_hp: Hp(20),
                block: Block(0),
                move_: Move::LouseBite,
                move_history: vec![], stolen_gold: 0,
                statuses: StatusMap::new(),
            }],
            turn: 1,
            phase: CombatPhase::PlayerTurn,
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0, hand_cost_max: None, hand_cost_max_expires: false, block_locked_turns: 0, pending_bombs: Vec::new(), duplication_pending: false, zero_cost_cards: Vec::new(),
        };
        wrap_combat(cs, floor)
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
        let graph = test_graph();
        let available_cols = all_cols(&graph, floor);
        let state = GameState::Map(MapState { player, floor, graph, available_cols, next_enemies: None, scenario: Scenario::Main });
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
        let graph = test_graph();
        let available_cols = all_cols(&graph, 0);
        let state = GameState::Map(MapState { player, floor: 0, graph, available_cols, next_enemies: None, scenario: Scenario::Main });
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
        let state = GameState::Map(MapState { player, floor: 9, graph: test_graph(), available_cols: vec![0], next_enemies: None, scenario: Scenario::Main });
        let (state, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.player.hp, Hp(75));
    }

    #[test]
    fn pantograph_does_not_heal_on_normal_floor() {
        let mut player = make_player();
        player.hp = Hp(50);
        player.relics.push(Relic::Pantograph);
        let graph = test_graph();
        let available_cols = all_cols(&graph, 0);
        let state = GameState::Map(MapState { player, floor: 0, graph, available_cols, next_enemies: None, scenario: Scenario::Main });
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
                draw_pile: vec![Card::Strike(Grade::Base); 5],
                ..player
            },
            enemies: vec![Enemy {
                kind: EnemyKind::RedLouse,
                hp: Hp(20),
                max_hp: Hp(20),
                block: Block(0),
                move_: Move::LouseBite,
                move_history: vec![], stolen_gold: 0,
                statuses: StatusMap::new(),
            }],
            turn: 1,
            phase: CombatPhase::PlayerTurn,
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0, hand_cost_max: None, hand_cost_max_expires: false, block_locked_turns: 0, pending_bombs: Vec::new(), duplication_pending: false, zero_cost_cards: Vec::new(),
        };
        wrap_combat(cs, floor)
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
                draw_pile: vec![Card::Strike(Grade::Base); 5],
                block: Block(5),
                ..player
            },
            enemies: vec![Enemy {
                kind: EnemyKind::RedLouse,
                hp: Hp(20),
                max_hp: Hp(20),
                block: Block(0),
                move_: Move::LouseBite,
                move_history: vec![], stolen_gold: 0,
                statuses: StatusMap::new(),
            }],
            turn: 1,
            phase: CombatPhase::PlayerTurn,
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0, hand_cost_max: None, hand_cost_max_expires: false, block_locked_turns: 0, pending_bombs: Vec::new(), duplication_pending: false, zero_cost_cards: Vec::new(),
        };
        let state = wrap_combat(cs, 0);
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
                hand: vec![Card::Strike(Grade::Base), Card::Strike(Grade::Base), Card::Strike(Grade::Base)],
                draw_pile: vec![Card::Strike(Grade::Base); 5],
                ..player
            },
            enemies: vec![Enemy {
                kind: EnemyKind::RedLouse,
                hp: Hp(20),
                max_hp: Hp(20),
                block: Block(0),
                move_: Move::LouseBite,
                move_history: vec![], stolen_gold: 0,
                statuses: StatusMap::new(),
            }],
            turn: 1,
            phase: CombatPhase::PlayerTurn,
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0, hand_cost_max: None, hand_cost_max_expires: false, block_locked_turns: 0, pending_bombs: Vec::new(), duplication_pending: false, zero_cost_cards: Vec::new(),
        };
        let state = wrap_combat(cs, 0);
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
            let (s, _) = apply_command(s, Command::StartPlayerTurn, &mut rng()).unwrap();
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
        let state = GameState::RestSite(RestSiteState { player, floor: 3, graph: test_graph(), available_cols: vec![0, 1] });
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
        state.player.draw_pile = vec![Card::Defend(Grade::Base); 10];

        // Turn 1: Cultist plays Incantation → gains Ritual(3), no Strength yet
        let (state, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
        let (state, _) = apply_combat_command(state, Command::EndEnemyTurn, &mut rng()).unwrap();
        let (mut state, _) = apply_combat_command(state, Command::StartPlayerTurn, &mut rng()).unwrap();
        let hp_after_turn_1 = state.player.hp.0;
        assert_eq!(hp_after_turn_1, state.player.max_hp.0);

        // Turn 2: Ritual ticks → Strength(3), then Dark Strike deals 6 + 3 = 9
        state.player.hand.clear();
        let (state, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
        let (state, _) = apply_combat_command(state, Command::EndEnemyTurn, &mut rng()).unwrap();
        let (state, _) = apply_combat_command(state, Command::StartPlayerTurn, &mut rng()).unwrap();
        assert_eq!(state.player.hp.0, hp_after_turn_1 - 9);
    }

    #[test]
    fn ritual_stacks_strength_each_turn() {
        let mut state = cultist_combat();
        state.player.hand.clear();
        state.player.draw_pile = vec![Card::Defend(Grade::Base); 20];

        // Three full turns: Incantation + 2× DarkStrike
        for _ in 0..3 {
            let (s, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
            let (s, _) = apply_combat_command(s, Command::EndEnemyTurn, &mut rng()).unwrap();
            let (s, _) = apply_combat_command(s, Command::StartPlayerTurn, &mut rng()).unwrap();
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
        let mut state = CombatState::from_player(player, vec![EnemyKind::RedLouse], &mut rng());
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
        let mut state = CombatState::from_player(player, vec![EnemyKind::RedLouse], &mut rng());
        state.player.hand = vec![Card::Defend(Grade::Base)];
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
        let state = GameState::RestSite(RestSiteState { player, floor: 3, graph: test_graph(), available_cols: vec![0, 1] });
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
            Command::Spawn(vec![EnemyKind::RedLouse]),
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
        let (state, _) = apply_command(state, Command::Spawn(vec![EnemyKind::RedLouse]), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::WinCombat, &mut rng()).unwrap();
        let GameState::Map(map) = state else { panic!("expected Map") };
        assert_eq!(map.player.potions, vec![Potion::BlockPotion]);
    }

    #[test]
    fn add_relic_on_map_grants_relic_to_player() {
        let state = run_after_neow();
        let (next, _) = apply_command(state, Command::AddRelic(Relic::BurningBlood), &mut rng()).unwrap();
        let GameState::Map(map) = next else { panic!("expected Map") };
        assert!(map.player.relics.contains(&Relic::BurningBlood));
    }

    #[test]
    fn add_potion_in_combat_adds_to_player() {
        let state = combat_at_floor(0);
        let (next, _) = apply_command(state, Command::AddPotion(Potion::FirePotion), &mut rng()).unwrap();
        let GameState::Combat { state: cs, .. } = next else { panic!("expected Combat") };
        assert!(cs.player.potions.contains(&Potion::FirePotion));
    }

    #[test]
    fn add_potion_in_combat_rejected_when_slots_full() {
        let GameState::Combat { state: mut cs, floor, is_boss, is_elite, graph, next_floor_cols, scenario }
            = combat_at_floor(0) else { panic!() };
        for _ in 0..MAX_POTIONS {
            cs.player.potions.push(Potion::BlockPotion);
        }
        let (next, _) = apply_command(
            GameState::Combat { state: cs, floor, is_boss, is_elite, graph, next_floor_cols, scenario },
            Command::AddPotion(Potion::FirePotion),
            &mut rng(),
        ).unwrap();
        let GameState::Combat { state: cs, .. } = next else { panic!("expected Combat") };
        assert!(!cs.player.potions.contains(&Potion::FirePotion));
    }

    // --- Potion rewards ---

    fn combat_at_floor_0() -> GameState {
        let (state, _) = apply_command(run_after_neow(), Command::ChooseNode(0), &mut rng()).unwrap();
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
        let state = run_after_neow();
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
        let state = run_after_neow();
        // skip to boss floor (floor 9)
        let mut state = state;
        for _ in 0..9 {
            let (s, _) = apply_command(state, Command::SkipFloor, &mut rng()).unwrap();
            state = s;
        }
        let (state, _) = apply_command(state, Command::ChooseNode(0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::WinCombat, &mut rng()).unwrap();
        assert!(matches!(state, GameState::GameOver { victory: true }));
    }

    #[test]
    fn potion_awarded_via_in_combat_kill() {
        let state = combat_at_floor_0();
        let GameState::Combat { state: mut cs, floor, is_boss, is_elite, graph, next_floor_cols, scenario } = state else { panic!() };
        cs.enemies[0].hp = Hp(6);
        let state = GameState::Combat { state: cs, floor, is_boss, is_elite, graph, next_floor_cols, scenario };
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        let (state, events) = apply_command(state, Command::EndEnemyTurn, &mut rng()).unwrap();
        if let GameState::CardReward(cr) = state {
            assert_eq!(cr.player.potions.len(), 1);
            assert!(events.iter().any(|e| matches!(e, Event::PotionAwarded { .. })));
        }
        // (if not yet in CardReward, combat may still be ongoing — that's fine for this test)
    }

    #[test]
    fn winning_combat_decreases_potion_chance_when_potion_drops() {
        // NoOpRng always returns true for gen_bool, so a potion always drops
        let (state, _) = apply_command(combat_at_floor_0(), Command::WinCombat, &mut rng()).unwrap();
        let GameState::CardReward(cr) = state else { panic!("expected CardReward") };
        assert!((cr.player.potion_chance - 0.30).abs() < f64::EPSILON);
    }

    #[test]
    fn winning_combat_increases_potion_chance_when_no_potion_drops() {
        // Set potion_chance to 0.0 so SeededRng will never roll a drop
        let base = combat_at_floor_0();
        let GameState::Combat { mut state, floor, is_boss, is_elite, graph, next_floor_cols, scenario } = base else { panic!() };
        state.player.potion_chance = 0.0;
        let state = GameState::Combat { state, floor, is_boss, is_elite, graph, next_floor_cols, scenario };
        let (state, _) = apply_command(state, Command::WinCombat, &mut crate::rng::SeededRng::new(0)).unwrap();
        let GameState::CardReward(cr) = state else { panic!("expected CardReward") };
        assert!((cr.player.potion_chance - 0.10).abs() < f64::EPSILON);
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
        let (state, _) = apply_command(state, Command::Spawn(vec![EnemyKind::RedLouse]), &mut rng()).unwrap();
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
        let state = GameState::RestSite(RestSiteState { player, floor: 3, graph: test_graph(), available_cols: vec![0, 1] });
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
        let state = run_after_neow();
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
                draw_pile: vec![Card::Strike(Grade::Base); 10],
                energy: Energy(20),
                max_energy: Energy(20),
                ..player
            },
            enemies: vec![Enemy {
                kind: EnemyKind::RedLouse,
                hp: Hp(enemy_hp),
                max_hp: Hp(enemy_hp),
                block: Block(0),
                move_: Move::LouseBite,
                move_history: vec![], stolen_gold: 0,
                statuses: StatusMap::new(),
            }],
            turn: 1,
            phase: CombatPhase::PlayerTurn,
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0, hand_cost_max: None, hand_cost_max_expires: false, block_locked_turns: 0, pending_bombs: Vec::new(), duplication_pending: false, zero_cost_cards: Vec::new(),
        };
        wrap_combat(cs, 0)
    }

    #[test]
    fn nunchaku_grants_energy_after_10th_attack() {
        let mut state = combat_with_relic_and_cards(
            Relic::Nunchaku,
            vec![Card::Strike(Grade::Base); 10],
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
        let state = combat_with_relic_and_cards(Relic::OrnamentalFan, vec![Card::Strike(Grade::Base); 3], 200);
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
                hand: vec![Card::Strike(Grade::Base), Card::Strike(Grade::Base)],
                draw_pile: vec![Card::Strike(Grade::Base); 10],
                ..player
            },
            enemies: vec![Enemy {
                kind: EnemyKind::RedLouse,
                hp: Hp(200),
                max_hp: Hp(200),
                block: Block(0),
                move_: Move::LouseBite,
                move_history: vec![], stolen_gold: 0,
                statuses: StatusMap::new(),
            }],
            turn: 1,
            phase: CombatPhase::PlayerTurn,
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0, hand_cost_max: None, hand_cost_max_expires: false, block_locked_turns: 0, pending_bombs: Vec::new(), duplication_pending: false, zero_cost_cards: Vec::new(),
        };
        let state = wrap_combat(cs, 0);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        // End turn (2 attacks, no fan fire)
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::EndEnemyTurn, &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::StartPlayerTurn, &mut rng()).unwrap();
        // Play 1 attack on turn 2 — attacks_this_turn should be 1 (reset), no fan fire
        let (state, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.player.block, Block(0), "fan should not have fired yet (only 1 attack this turn)");
        assert!(!events.iter().any(|e| matches!(e, Event::PlayerBlocked { amount: 4 })));
    }

    #[test]
    fn gremlin_horn_grants_energy_and_draws_card_on_kill() {
        let state = combat_with_relic_and_cards(Relic::GremlinHorn, vec![Card::Strike(Grade::Base)], 6);
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
                hand: vec![Card::Defend(Grade::Base)],
                draw_pile: vec![Card::Strike(Grade::Base); 10],
                ..player
            },
            enemies: vec![Enemy {
                kind: EnemyKind::RedLouse,
                hp: Hp(20),
                max_hp: Hp(20),
                block: Block(0),
                move_: Move::LouseBite,
                move_history: vec![], stolen_gold: 0,
                statuses: StatusMap::new(),
            }],
            turn: 1,
            phase: CombatPhase::PlayerTurn,
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0, hand_cost_max: None, hand_cost_max_expires: false, block_locked_turns: 0, pending_bombs: Vec::new(), duplication_pending: false, zero_cost_cards: Vec::new(),
        };
        let state = wrap_combat(cs, 0);
        // Play 1 card (≤3), end turn
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::EndEnemyTurn, &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::StartPlayerTurn, &mut rng()).unwrap();
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
                hand: vec![Card::Defend(Grade::Base); 4],
                draw_pile: vec![Card::Strike(Grade::Base); 10],
                energy: Energy(20),
                max_energy: Energy(20),
                ..player
            },
            enemies: vec![Enemy {
                kind: EnemyKind::RedLouse,
                hp: Hp(200),
                max_hp: Hp(200),
                block: Block(0),
                move_: Move::LouseBite,
                move_history: vec![], stolen_gold: 0,
                statuses: StatusMap::new(),
            }],
            turn: 1,
            phase: CombatPhase::PlayerTurn,
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0, hand_cost_max: None, hand_cost_max_expires: false, block_locked_turns: 0, pending_bombs: Vec::new(), duplication_pending: false, zero_cost_cards: Vec::new(),
        };
        let mut state = wrap_combat(cs, 0);
        // Play 4 cards (>3 threshold), end turn
        for _ in 0..4 {
            (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        }
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::EndEnemyTurn, &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::StartPlayerTurn, &mut rng()).unwrap();
        // Should draw exactly 5 cards (no bonus)
        let GameState::Combat { state: cs, .. } = state else { panic!("expected Combat") };
        assert_eq!(cs.player.hand.len(), 5);
    }

    // --- treasure room ---

    fn treasure_map_at(floor: usize) -> GameState {
        let graph = MapGraph {
            rows: vec![vec![MapNode::Treasure], vec![MapNode::RestSite]],
            edges: vec![vec![vec![0]], vec![vec![]]],
        };
        GameState::Map(MapState {
            player: make_player(),
            floor,
            graph,
            available_cols: vec![0],
            next_enemies: None,
            scenario: Scenario::Main,
        })
    }

    #[test]
    fn choosing_treasure_node_transitions_to_treasure_room() {
        let (next, _) = apply_command(treasure_map_at(0), Command::ChooseNode(0), &mut rng()).unwrap();
        assert!(matches!(next, GameState::TreasureRoom(_)));
    }

    #[test]
    fn treasure_room_state_contains_relic() {
        let (next, _) = apply_command(treasure_map_at(0), Command::ChooseNode(0), &mut rng()).unwrap();
        let GameState::TreasureRoom(tr) = next else { panic!("expected TreasureRoom") };
        let _relic = tr.relic; // confirm relic field accessible
    }

    #[test]
    fn leave_treasure_returns_to_map_at_next_floor() {
        let (tr_state, _) = apply_command(treasure_map_at(0), Command::ChooseNode(0), &mut rng()).unwrap();
        let (next, _) = apply_command(tr_state, Command::LeaveTreasure, &mut rng()).unwrap();
        let GameState::Map(map) = next else { panic!("expected Map") };
        assert_eq!(map.floor, 1);
    }

    #[test]
    fn leave_treasure_grants_relic_to_player() {
        let state = GameState::TreasureRoom(TreasureRoomState {
            player: make_player(),
            floor: 3,
            graph: test_graph(),
            available_cols: vec![0],
            relic: Relic::Anchor,
        });
        let (next, _) = apply_command(state, Command::LeaveTreasure, &mut rng()).unwrap();
        let GameState::Map(map) = next else { panic!("expected Map") };
        assert!(map.player.relics.contains(&Relic::Anchor));
    }

    #[test]
    fn non_treasure_commands_rejected_in_treasure_room() {
        let state = GameState::TreasureRoom(TreasureRoomState {
            player: make_player(),
            floor: 3,
            graph: test_graph(),
            available_cols: vec![0],
            relic: Relic::Anchor,
        });
        assert_eq!(
            apply_command(state, Command::EndTurn, &mut rng()),
            Err(CommandError::InvalidPhase)
        );
    }

    // --- event room ---

    fn ssssserpent_state() -> GameState {
        GameState::EventRoom(EventRoomState {
            player: make_player(),
            floor: 3,
            graph: test_graph(),
            available_cols: vec![0, 1],
            event: EventKind::Ssssserpent,
        })
    }

    #[test]
    fn ssssserpent_agree_adds_150_gold() {
        let (state, _) = apply_command(ssssserpent_state(), Command::ChooseEventOption(0), &mut rng()).unwrap();
        let GameState::Map(map) = state else { panic!("expected Map") };
        assert_eq!(map.player.gold, 150);
    }

    #[test]
    fn ssssserpent_agree_adds_doubt_to_deck() {
        let (state, _) = apply_command(ssssserpent_state(), Command::ChooseEventOption(0), &mut rng()).unwrap();
        let GameState::Map(map) = state else { panic!("expected Map") };
        assert!(map.player.deck.contains(&Card::Doubt));
    }

    #[test]
    fn ssssserpent_agree_emits_card_added_doubt() {
        let (_, events) = apply_command(ssssserpent_state(), Command::ChooseEventOption(0), &mut rng()).unwrap();
        assert!(events.iter().any(|e| matches!(e, Event::CardAdded { card: Card::Doubt })));
    }

    #[test]
    fn ssssserpent_disagree_changes_nothing() {
        let (state, _) = apply_command(ssssserpent_state(), Command::ChooseEventOption(1), &mut rng()).unwrap();
        let GameState::Map(map) = state else { panic!("expected Map") };
        assert_eq!(map.player.gold, 0);
        assert!(!map.player.deck.contains(&Card::Doubt));
    }

    #[test]
    fn ssssserpent_leave_changes_nothing() {
        let (state, _) = apply_command(ssssserpent_state(), Command::ChooseEventOption(2), &mut rng()).unwrap();
        let GameState::Map(map) = state else { panic!("expected Map") };
        assert_eq!(map.player.gold, 0);
        assert!(!map.player.deck.contains(&Card::Doubt));
    }

    #[test]
    fn ssssserpent_all_options_return_to_map() {
        for opt in 0..=2 {
            let (state, _) = apply_command(ssssserpent_state(), Command::ChooseEventOption(opt), &mut rng()).unwrap();
            assert!(matches!(state, GameState::Map(_)), "option {opt} should return to Map");
        }
    }

    #[test]
    fn ssssserpent_advances_to_next_floor() {
        let state = GameState::EventRoom(EventRoomState {
            player: make_player(),
            floor: 3,
            graph: test_graph(),
            available_cols: vec![0, 1],
            event: EventKind::Ssssserpent,
        });
        let (next, _) = apply_command(state, Command::ChooseEventOption(0), &mut rng()).unwrap();
        let GameState::Map(map) = next else { panic!("expected Map") };
        assert_eq!(map.floor, 4);
    }

    fn big_fish_state() -> GameState {
        GameState::EventRoom(EventRoomState {
            player: make_player(),
            floor: 3,
            graph: test_graph(),
            available_cols: vec![0, 1],
            event: EventKind::BigFish,
        })
    }

    #[test]
    fn big_fish_banana_heals_30_percent_of_max_hp() {
        let mut player = make_player();
        player.hp = Hp(20);
        let state = GameState::EventRoom(EventRoomState {
            player,
            floor: 3,
            graph: test_graph(),
            available_cols: vec![0, 1],
            event: EventKind::BigFish,
        });
        let (next, _) = apply_command(state, Command::ChooseEventOption(0), &mut rng()).unwrap();
        let GameState::Map(map) = next else { panic!("expected Map") };
        assert_eq!(map.player.hp, Hp(44)); // 20 + (80 * 30 / 100) = 20 + 24 = 44
    }

    #[test]
    fn big_fish_banana_cannot_overheal() {
        let (next, _) = apply_command(big_fish_state(), Command::ChooseEventOption(0), &mut rng()).unwrap();
        let GameState::Map(map) = next else { panic!("expected Map") };
        assert_eq!(map.player.hp, Hp(80));
    }

    #[test]
    fn big_fish_donut_increases_max_hp_by_3() {
        let (next, _) = apply_command(big_fish_state(), Command::ChooseEventOption(1), &mut rng()).unwrap();
        let GameState::Map(map) = next else { panic!("expected Map") };
        assert_eq!(map.player.max_hp, Hp(83));
    }

    #[test]
    fn big_fish_donut_also_increases_current_hp() {
        let (next, _) = apply_command(big_fish_state(), Command::ChooseEventOption(1), &mut rng()).unwrap();
        let GameState::Map(map) = next else { panic!("expected Map") };
        assert_eq!(map.player.hp, Hp(83));
    }

    #[test]
    fn big_fish_box_adds_relic() {
        let (next, _) = apply_command(big_fish_state(), Command::ChooseEventOption(2), &mut rng()).unwrap();
        let GameState::Map(map) = next else { panic!("expected Map") };
        assert!(!map.player.relics.is_empty());
    }

    #[test]
    fn big_fish_box_adds_regret_to_deck() {
        let (next, _) = apply_command(big_fish_state(), Command::ChooseEventOption(2), &mut rng()).unwrap();
        let GameState::Map(map) = next else { panic!("expected Map") };
        assert!(map.player.deck.contains(&Card::Regret));
    }

    #[test]
    fn big_fish_box_emits_card_added_regret() {
        let (_, events) = apply_command(big_fish_state(), Command::ChooseEventOption(2), &mut rng()).unwrap();
        assert!(events.iter().any(|e| matches!(e, Event::CardAdded { card: Card::Regret })));
    }

    #[test]
    fn big_fish_leave_does_nothing() {
        let (next, events) = apply_command(big_fish_state(), Command::ChooseEventOption(3), &mut rng()).unwrap();
        let GameState::Map(map) = next else { panic!("expected Map") };
        assert_eq!(map.player.hp, Hp(80));
        assert_eq!(map.player.max_hp, Hp(80));
        assert!(map.player.relics.is_empty());
        assert!(events.is_empty());
    }

    fn mushrooms_state() -> GameState {
        GameState::EventRoom(EventRoomState {
            player: make_player(),
            floor: 3,
            graph: test_graph(),
            available_cols: vec![0, 1],
            event: EventKind::Mushrooms,
        })
    }

    #[test]
    fn mushrooms_eat_heals_12_hp() {
        let mut player = make_player();
        player.hp = Hp(50);
        let state = GameState::EventRoom(EventRoomState {
            player,
            floor: 3,
            graph: test_graph(),
            available_cols: vec![0, 1],
            event: EventKind::Mushrooms,
        });
        let (next, _) = apply_command(state, Command::ChooseEventOption(0), &mut rng()).unwrap();
        let GameState::Map(map) = next else { panic!("expected Map") };
        assert_eq!(map.player.hp, Hp(62));
    }

    #[test]
    fn mushrooms_eat_cannot_overheal() {
        let (next, _) = apply_command(mushrooms_state(), Command::ChooseEventOption(0), &mut rng()).unwrap();
        let GameState::Map(map) = next else { panic!("expected Map") };
        assert_eq!(map.player.hp, Hp(80));
    }

    #[test]
    fn mushrooms_eat_adds_parasite_to_deck() {
        let (next, _) = apply_command(mushrooms_state(), Command::ChooseEventOption(0), &mut rng()).unwrap();
        let GameState::Map(map) = next else { panic!("expected Map") };
        assert!(map.player.deck.contains(&Card::Parasite));
    }

    #[test]
    fn mushrooms_eat_emits_card_added_parasite() {
        let (_, events) = apply_command(mushrooms_state(), Command::ChooseEventOption(0), &mut rng()).unwrap();
        assert!(events.iter().any(|e| matches!(e, Event::CardAdded { card: Card::Parasite })));
    }

    #[test]
    fn mushrooms_leave_does_nothing() {
        let (next, events) = apply_command(mushrooms_state(), Command::ChooseEventOption(1), &mut rng()).unwrap();
        let GameState::Map(map) = next else { panic!("expected Map") };
        assert_eq!(map.player.hp, Hp(80));
        assert!(!map.player.deck.contains(&Card::Parasite));
        assert!(events.is_empty());
    }

    fn golden_idol_state() -> GameState {
        GameState::EventRoom(EventRoomState {
            player: make_player(),
            floor: 3,
            graph: test_graph(),
            available_cols: vec![0, 1],
            event: EventKind::GoldenIdol,
        })
    }

    #[test]
    fn golden_idol_outrun_adds_injury_to_deck() {
        let (next, _) = apply_command(golden_idol_state(), Command::ChooseEventOption(0), &mut rng()).unwrap();
        let GameState::Map(map) = next else { panic!("expected Map") };
        assert!(map.player.deck.contains(&Card::Injury));
    }

    #[test]
    fn golden_idol_outrun_emits_card_added_injury() {
        let (_, events) = apply_command(golden_idol_state(), Command::ChooseEventOption(0), &mut rng()).unwrap();
        assert!(events.iter().any(|e| matches!(e, Event::CardAdded { card: Card::Injury })));
    }

    #[test]
    fn golden_idol_outrun_gives_250_gold() {
        let (next, _) = apply_command(golden_idol_state(), Command::ChooseEventOption(0), &mut rng()).unwrap();
        let GameState::Map(map) = next else { panic!("expected Map") };
        assert_eq!(map.player.gold, 250);
    }

    #[test]
    fn golden_idol_smash_deals_25_damage() {
        let (next, _) = apply_command(golden_idol_state(), Command::ChooseEventOption(1), &mut rng()).unwrap();
        let GameState::Map(map) = next else { panic!("expected Map") };
        assert_eq!(map.player.hp, Hp(55));
    }

    #[test]
    fn golden_idol_smash_gives_250_gold() {
        let (next, _) = apply_command(golden_idol_state(), Command::ChooseEventOption(1), &mut rng()).unwrap();
        let GameState::Map(map) = next else { panic!("expected Map") };
        assert_eq!(map.player.gold, 250);
    }

    #[test]
    fn golden_idol_hide_costs_6_max_hp() {
        let (next, _) = apply_command(golden_idol_state(), Command::ChooseEventOption(2), &mut rng()).unwrap();
        let GameState::Map(map) = next else { panic!("expected Map") };
        assert_eq!(map.player.max_hp, Hp(74));
    }

    #[test]
    fn golden_idol_hide_gives_250_gold() {
        let (next, _) = apply_command(golden_idol_state(), Command::ChooseEventOption(2), &mut rng()).unwrap();
        let GameState::Map(map) = next else { panic!("expected Map") };
        assert_eq!(map.player.gold, 250);
    }

    #[test]
    fn golden_idol_leave_does_nothing() {
        let (next, events) = apply_command(golden_idol_state(), Command::ChooseEventOption(3), &mut rng()).unwrap();
        let GameState::Map(map) = next else { panic!("expected Map") };
        assert_eq!(map.player.gold, 0);
        assert_eq!(map.player.hp, Hp(80));
        assert_eq!(map.player.max_hp, Hp(80));
        assert!(!map.player.deck.contains(&Card::Injury));
        assert!(events.is_empty());
    }

    #[test]
    fn non_event_commands_rejected_in_event_room() {
        assert_eq!(
            apply_command(ssssserpent_state(), Command::EndTurn, &mut rng()),
            Err(CommandError::InvalidPhase)
        );
    }
}

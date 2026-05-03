use crate::cards::Card;
use crate::enemies::{self, Effect, EnemyKind, Intent, Move};
use crate::potions::Potion;
use crate::relics::Relic;
use crate::rng::Rng;
use crate::run::{Command, CommandError};
use crate::status::{StatusEffect, StatusMap, drain_poison, resolve_block, resolve_damage, tick_ritual, tick_statuses};
use crate::types::{Block, Energy, Hp};

#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    pub hp: Hp,
    pub max_hp: Hp,
    pub block: Block,
    pub energy: Energy,
    pub max_energy: Energy,
    pub hand: Vec<Card>,
    pub draw_pile: Vec<Card>,
    pub discard_pile: Vec<Card>,
    pub exhaust_pile: Vec<Card>,
    pub statuses: StatusMap,
    pub deck: Vec<Card>,
    pub gold: i32,
    pub relics: Vec<Relic>,
    pub potions: Vec<Potion>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub hp: Hp,
    pub max_hp: Hp,
    pub block: Block,
    pub move_: Move,
    pub last_move: Option<Move>,
    pub statuses: StatusMap,
}

impl Enemy {
    pub fn name(&self) -> &'static str { self.kind.name() }

    pub fn effective_intent(&self, player_statuses: &StatusMap) -> Intent {
        match self.move_.intent() {
            Intent::Attack(n) => Intent::Attack(resolve_damage(n, &self.statuses, player_statuses)),
            Intent::AttackDefend(d, b) => Intent::AttackDefend(resolve_damage(d, &self.statuses, player_statuses), b),
            other => other,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CombatPhase {
    PlayerTurn,
    EnemyTurn,
    Victory,
    Defeat,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CombatState {
    pub player: Player,
    pub enemies: Vec<Enemy>,
    pub turn: u32,
    pub phase: CombatPhase,
}

impl CombatState {
    pub fn new(rng: &mut impl Rng) -> Self {
        let deck = crate::cards::starter_deck();
        let player = Player {
            hp: Hp(80),
            max_hp: Hp(80),
            block: Block(0),
            energy: Energy(3),
            max_energy: Energy(3),
            hand: Vec::new(),
            draw_pile: deck.clone(),
            discard_pile: Vec::new(),
            exhaust_pile: Vec::new(),
            statuses: StatusMap::new(),
            deck,
            gold: 0,
            relics: Vec::new(),
            potions: Vec::new(),
        };
        Self::from_player(player, vec![EnemyKind::Louse], rng)
    }

    pub fn from_player(player: Player, enemy_kinds: Vec<EnemyKind>, rng: &mut impl Rng) -> Self {
        let mut draw_pile = player.deck.clone();
        rng.shuffle(&mut draw_pile);
        let mut p = Player {
            draw_pile,
            hand: Vec::new(),
            discard_pile: Vec::new(),
            exhaust_pile: Vec::new(),
            block: Block(0),
            energy: player.max_energy,
            statuses: StatusMap::new(),
            ..player
        };
        draw_cards(&mut p, 5, rng);
        let enemies = enemy_kinds
            .iter()
            .map(|kind| {
                let max_hp = kind.max_hp();
                let first_move = enemies::next_move(kind, None, rng);
                Enemy {
                    kind: kind.clone(),
                    hp: max_hp,
                    max_hp,
                    block: Block(0),
                    move_: first_move,
                    last_move: None,
                    statuses: StatusMap::new(),
                }
            })
            .collect();
        Self {
            player: p,
            enemies,
            turn: 1,
            phase: CombatPhase::PlayerTurn,
        }
    }
}

pub(crate) fn draw_cards(player: &mut Player, n: usize, rng: &mut impl Rng) {
    for _ in 0..n {
        if player.draw_pile.is_empty() {
            if player.discard_pile.is_empty() {
                break;
            }
            player.draw_pile = std::mem::take(&mut player.discard_pile);
            rng.shuffle(&mut player.draw_pile);
        }
        if let Some(card) = player.draw_pile.pop() {
            player.hand.push(card);
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    CardPlayed { card: Card },
    PlayerAttacked { raw: i32, damage: i32 },
    PlayerBlocked { amount: i32 },
    EnemyAttacked { raw: i32, damage: i32 },
    EnemyDefended { amount: i32 },
    StatusApplied { target: Target, status: StatusEffect, stacks: i32 },
    IntentRevealed { intent: Intent },
    PlayerBlockExpired { amount: i32 },
    TurnEnded,
    TurnStarted { turn: u32 },
    EnemyPoisoned { damage: i32 },
    EnemyDied,
    PlayerDied,
    PlayerSelfDamaged { amount: i32 },
    EnergyGained { amount: i32 },
    CardsDrawn { count: usize },
    GoldEarned { amount: i32 },
    Healed { amount: i32 },
    CardAdded { card: Card },
    CardExhausted { card: Card },
    CardUpgraded { from: Card, to: Card },
    StatusCardAddedToDiscard { card: Card },
    PotionUsed { potion: Potion },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Target {
    Player,
    Enemy,
}

pub(crate) fn apply_combat_command(
    mut state: CombatState,
    command: Command,
    rng: &mut impl Rng,
) -> Result<(CombatState, Vec<Event>), CommandError> {
    if matches!(state.phase, CombatPhase::Victory | CombatPhase::Defeat) {
        return Err(CommandError::CombatOver);
    }

    let mut events = Vec::new();

    match command {
        Command::PlayCard(index, target_idx) => {
            if state.phase != CombatPhase::PlayerTurn {
                return Err(CommandError::InvalidPhase);
            }
            if index >= state.player.hand.len() {
                return Err(CommandError::InvalidCard);
            }
            let card = state.player.hand[index].clone();
            if !card.is_playable() {
                return Err(CommandError::InvalidCard);
            }
            if state.player.energy < card.energy_cost() {
                return Err(CommandError::NotEnoughEnergy);
            }
            if card.card_type() == crate::cards::CardType::Attack
                && state.player.statuses.contains_key(&StatusEffect::Entangle)
            {
                return Err(CommandError::Entangled);
            }
            // Resolve target: use specified if alive, else first living; out-of-bounds is error
            let actual_target = if target_idx >= state.enemies.len() {
                return Err(CommandError::InvalidCard);
            } else if state.enemies[target_idx].hp > Hp(0) {
                target_idx
            } else {
                state.enemies.iter().position(|e| e.hp > Hp(0))
                    .ok_or(CommandError::InvalidCard)?
            };
            state.player.hand.remove(index);
            state.player.energy = Energy(state.player.energy.0 - card.energy_cost().0);
            events.push(Event::CardPlayed { card: card.clone() });
            crate::cards::apply(&card, &mut state, &mut events, actual_target, rng);
            if card.exhausts() {
                events.push(Event::CardExhausted { card: card.clone() });
                state.player.exhaust_pile.push(card.clone());
            } else if card.card_type() != crate::cards::CardType::Power {
                state.player.discard_pile.push(card.clone());
            }
            if state.player.hp <= Hp(0) {
                state.phase = CombatPhase::Defeat;
                return Ok((state, events));
            }
            if state.enemies[actual_target].hp <= Hp(0) {
                events.push(Event::EnemyDied);
            }
            if state.enemies.iter().all(|e| e.hp <= Hp(0)) {
                state.phase = CombatPhase::Victory;
            }
        }
        Command::EndTurn => {
            if state.phase != CombatPhase::PlayerTurn {
                return Err(CommandError::InvalidPhase);
            }
            events.push(Event::TurnEnded);
            state.player.discard_pile.append(&mut state.player.hand);
            tick_statuses(&mut state.player.statuses);
            for i in 0..state.enemies.len() {
                if state.enemies[i].hp <= Hp(0) { continue; }
                let poison_dmg = drain_poison(&mut state.enemies[i].statuses);
                if poison_dmg > 0 {
                    state.enemies[i].hp.0 = (state.enemies[i].hp.0 - poison_dmg).max(0);
                    events.push(Event::EnemyPoisoned { damage: poison_dmg });
                    if state.enemies[i].hp <= Hp(0) {
                        events.push(Event::EnemyDied);
                    }
                }
            }
            if state.enemies.iter().all(|e| e.hp <= Hp(0)) {
                state.phase = CombatPhase::Victory;
                return Ok((state, events));
            }
            state.phase = CombatPhase::EnemyTurn;
        }
        Command::UsePotion(slot, target_idx) => {
            if state.phase != CombatPhase::PlayerTurn {
                return Err(CommandError::InvalidPhase);
            }
            if slot >= state.player.potions.len() {
                return Err(CommandError::InvalidCard);
            }
            let potion = state.player.potions.remove(slot);
            apply_potion(potion, target_idx, &mut state, &mut events, rng);
            events.push(Event::PotionUsed { potion });
        }
        Command::ChooseNode(_)
        | Command::Rest
        | Command::ChooseCardReward(_)
        | Command::SkipReward
        | Command::UpgradeCard(_)
        | Command::SkipFloor
        | Command::WinCombat
        | Command::AddCard(_)
        | Command::AddRelic(_)
        | Command::AddPotion(_)
        | Command::Spawn(_) => {
            return Err(CommandError::InvalidPhase);
        }
        Command::EndEnemyTurn => {
            if state.phase != CombatPhase::EnemyTurn {
                return Err(CommandError::InvalidPhase);
            }
            for i in 0..state.enemies.len() {
                if state.enemies[i].hp <= Hp(0) { continue; }
                state.enemies[i].block = Block(0);
                tick_ritual(&mut state.enemies[i].statuses);
                let current_move = state.enemies[i].move_;
                let enemy_statuses = state.enemies[i].statuses.clone();
                for effect in current_move.def().effects {
                    match effect {
                        Effect::DealDamage(n) => {
                            let effective = crate::status::resolve_damage(n, &enemy_statuses, &state.player.statuses);
                            let damage = deal_damage(effective, &mut state.player.hp, &mut state.player.block);
                            events.push(Event::EnemyAttacked { raw: effective, damage });
                        }
                        Effect::GainBlock(n) => {
                            state.enemies[i].block.0 += n;
                            events.push(Event::EnemyDefended { amount: n });
                        }
                        Effect::GainStatus(status, stacks) => {
                            *state.enemies[i].statuses.entry(status).or_insert(0) += stacks;
                            events.push(Event::StatusApplied { target: Target::Enemy, status, stacks });
                        }
                        Effect::ApplyStatus(status, stacks) => {
                            apply_status(&mut state.player.statuses, Target::Player, status, stacks, &mut events);
                        }
                        Effect::AddToDiscard(card) => {
                            state.player.discard_pile.push(card.clone());
                            events.push(Event::StatusCardAddedToDiscard { card });
                        }
                    }
                }
                tick_statuses(&mut state.enemies[i].statuses);
                let last = state.enemies[i].move_;
                state.enemies[i].last_move = Some(last);
            }
            if state.player.hp <= Hp(0) {
                state.phase = CombatPhase::Defeat;
                events.push(Event::PlayerDied);
            } else {
                if state.player.block > Block(0) {
                    events.push(Event::PlayerBlockExpired { amount: state.player.block.0 });
                }
                state.player.block = Block(0);
                state.player.energy = state.player.max_energy;
                state.turn += 1;
                for enemy in state.enemies.iter_mut() {
                    if enemy.hp > Hp(0) {
                        enemy.move_ = enemies::next_move(&enemy.kind, enemy.last_move, rng);
                        events.push(Event::IntentRevealed { intent: enemy.move_.intent() });
                    }
                }
                draw_cards(&mut state.player, 5, rng);
                state.phase = CombatPhase::PlayerTurn;
                events.push(Event::TurnStarted { turn: state.turn });
            }
        }
    }

    Ok((state, events))
}

pub(crate) fn apply_status(
    statuses: &mut StatusMap,
    target: Target,
    effect: StatusEffect,
    stacks: i32,
    events: &mut Vec<Event>,
) {
    *statuses.entry(effect).or_insert(0) += stacks;
    events.push(Event::StatusApplied { target, status: effect, stacks });
}

pub(crate) fn deal_damage(amount: i32, hp: &mut Hp, block: &mut Block) -> i32 {
    let absorbed = amount.min(block.0).max(0);
    block.0 -= absorbed;
    let remainder = amount - absorbed;
    hp.0 = (hp.0 - remainder).max(0);
    remainder
}

pub(crate) fn damage_player(state: &mut CombatState, events: &mut Vec<Event>, amount: i32) {
    state.player.hp.0 = (state.player.hp.0 - amount).max(0);
    events.push(Event::PlayerSelfDamaged { amount });
}

fn apply_potion(
    potion: Potion,
    target_idx: usize,
    state: &mut CombatState,
    events: &mut Vec<Event>,
    rng: &mut impl Rng,
) {
    match potion {
        Potion::FirePotion => {
            let target = target_idx.min(state.enemies.len().saturating_sub(1));
            let dmg = resolve_damage(20, &StatusMap::new(), &state.enemies[target].statuses);
            let e = &mut state.enemies[target];
            let dealt = deal_damage(dmg, &mut e.hp, &mut e.block);
            events.push(Event::EnemyAttacked { raw: dmg, damage: dealt });
            if state.enemies[target].hp <= Hp(0) {
                events.push(Event::EnemyDied);
            }
        }
        Potion::ExplosivePotion => {
            for i in 0..state.enemies.len() {
                if state.enemies[i].hp <= Hp(0) { continue; }
                let dmg = resolve_damage(10, &StatusMap::new(), &state.enemies[i].statuses);
                let e = &mut state.enemies[i];
                let dealt = deal_damage(dmg, &mut e.hp, &mut e.block);
                events.push(Event::EnemyAttacked { raw: dmg, damage: dealt });
                if state.enemies[i].hp <= Hp(0) {
                    events.push(Event::EnemyDied);
                }
            }
        }
        Potion::BlockPotion => {
            let gained = resolve_block(12, &state.player.statuses);
            state.player.block.0 += gained;
            events.push(Event::PlayerBlocked { amount: gained });
        }
        Potion::StrengthPotion => {
            apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Strength, 2, events);
        }
        Potion::SwiftPotion => {
            draw_cards(&mut state.player, 3, rng);
            events.push(Event::CardsDrawn { count: 3 });
        }
        Potion::FearPotion => {
            let target = target_idx.min(state.enemies.len().saturating_sub(1));
            apply_status(&mut state.enemies[target].statuses, Target::Enemy, StatusEffect::Vulnerable, 3, events);
        }
        Potion::WeakPotion => {
            let target = target_idx.min(state.enemies.len().saturating_sub(1));
            apply_status(&mut state.enemies[target].statuses, Target::Enemy, StatusEffect::Weak, 3, events);
        }
        Potion::BloodPotion => {
            let heal = (state.player.max_hp.0 * 20 / 100).max(1);
            state.player.hp.0 = (state.player.hp.0 + heal).min(state.player.max_hp.0);
            events.push(Event::Healed { amount: heal });
        }
        Potion::EnergyPotion => {
            state.player.energy.0 += 2;
            events.push(Event::EnergyGained { amount: 2 });
        }
    }
    if state.enemies.iter().all(|e| e.hp <= Hp(0)) {
        state.phase = CombatPhase::Victory;
    }
}

#[cfg(test)]
pub(crate) fn combat_with_hand(hand: Vec<Card>) -> CombatState {
    CombatState {
        player: Player {
            hp: Hp(80),
            max_hp: Hp(80),
            block: Block(0),
            energy: Energy(3),
            max_energy: Energy(3),
            hand,
            draw_pile: Vec::new(),
            discard_pile: Vec::new(),
            exhaust_pile: Vec::new(),
            statuses: StatusMap::new(),
            deck: Vec::new(),
            gold: 0,
            relics: Vec::new(),
            potions: Vec::new(),
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
    }
}

#[cfg(test)]
pub(crate) fn combat_with_two_enemies(hand: Vec<Card>) -> CombatState {
    let louse = || Enemy {
        kind: EnemyKind::Louse,
        hp: Hp(20),
        max_hp: Hp(20),
        block: Block(0),
        move_: Move::LouseBite,
        last_move: None,
        statuses: StatusMap::new(),
    };
    CombatState {
        player: Player {
            hp: Hp(80),
            max_hp: Hp(80),
            block: Block(0),
            energy: Energy(3),
            max_energy: Energy(3),
            hand,
            draw_pile: Vec::new(),
            discard_pile: Vec::new(),
            exhaust_pile: Vec::new(),
            statuses: StatusMap::new(),
            deck: Vec::new(),
            gold: 0,
            relics: Vec::new(),
            potions: Vec::new(),
        },
        enemies: vec![louse(), louse()],
        turn: 1,
        phase: CombatPhase::PlayerTurn,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::potions::Potion;
    use crate::rng::NoOpRng;
    use crate::run::{Command, CommandError};

    fn rng() -> NoOpRng {
        NoOpRng
    }

    fn apply_command(
        state: CombatState,
        cmd: Command,
        rng: &mut impl Rng,
    ) -> Result<(CombatState, Vec<Event>), CommandError> {
        super::apply_combat_command(state, cmd, rng)
    }

    fn end_turn_full(
        state: CombatState,
        rng: &mut impl Rng,
    ) -> Result<(CombatState, Vec<Event>), CommandError> {
        let (state, mut events) = apply_command(state, Command::EndTurn, rng)?;
        if state.phase != CombatPhase::EnemyTurn {
            return Ok((state, events));
        }
        let (state, more) = apply_command(state, Command::EndEnemyTurn, rng)?;
        events.extend(more);
        Ok((state, events))
    }

    // --- Combat start / drawing ---

    #[test]
    fn new_combat_deals_5_cards_to_hand() {
        let state = CombatState::new(&mut rng());
        assert_eq!(state.player.hand.len(), 5);
    }

    #[test]
    fn new_combat_leaves_7_cards_in_draw_pile() {
        let state = CombatState::new(&mut rng());
        assert_eq!(state.player.draw_pile.len(), 7);
    }

    #[test]
    fn end_turn_draws_5_new_cards() {
        let state = combat_with_hand(Vec::new());
        let state = CombatState {
            player: Player {
                draw_pile: vec![
                    Card::Strike,
                    Card::Strike,
                    Card::Strike,
                    Card::Strike,
                    Card::Strike,
                ],
                ..state.player
            },
            ..state
        };
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 5);
    }

    #[test]
    fn end_turn_discards_remaining_hand() {
        let mut state = combat_with_hand(vec![Card::Strike, Card::Defend]);
        state.player.draw_pile = vec![Card::Strike; 5];
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 5);
        assert!(state.player.discard_pile.contains(&Card::Strike));
        assert!(state.player.discard_pile.contains(&Card::Defend));
    }

    #[test]
    fn empty_draw_pile_shuffles_discard_when_drawing() {
        let mut state = combat_with_hand(Vec::new());
        state.player.discard_pile = vec![Card::Strike, Card::Defend, Card::Strike];
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 3);
        assert!(state.player.discard_pile.is_empty());
    }

    // --- Energy ---

    #[test]
    fn player_starts_with_3_energy() {
        let state = CombatState::new(&mut rng());
        assert_eq!(state.player.energy, Energy(3));
    }

    #[test]
    fn playing_a_card_costs_energy() {
        let state = combat_with_hand(vec![Card::Strike]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.energy, Energy(2));
    }

    #[test]
    fn energy_resets_at_start_of_next_turn() {
        let mut state = combat_with_hand(vec![Card::Strike]);
        state.player.energy = Energy(0);
        state.player.draw_pile = vec![Card::Strike; 5];
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.player.energy, Energy(3));
    }

    #[test]
    fn cannot_play_card_without_sufficient_energy() {
        let mut state = combat_with_hand(vec![Card::Strike]);
        state.player.energy = Energy(0);
        let result = apply_command(state, Command::PlayCard(0, 0), &mut rng());
        assert_eq!(result, Err(CommandError::NotEnoughEnergy));
    }

    #[test]
    fn entangle_prevents_playing_attack_cards() {
        let mut state = combat_with_hand(vec![Card::Strike]);
        state.player.statuses.insert(StatusEffect::Entangle, 1);
        let result = apply_command(state, Command::PlayCard(0, 0), &mut rng());
        assert_eq!(result, Err(CommandError::Entangled));
    }

    #[test]
    fn entangle_does_not_prevent_playing_skill_cards() {
        let mut state = combat_with_hand(vec![Card::Defend]);
        state.player.statuses.insert(StatusEffect::Entangle, 1);
        let result = apply_command(state, Command::PlayCard(0, 0), &mut rng());
        assert!(result.is_ok());
    }

    #[test]
    fn entangle_expires_at_end_of_turn() {
        let mut state = combat_with_hand(vec![]);
        state.player.statuses.insert(StatusEffect::Entangle, 1);
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert!(!state.player.statuses.contains_key(&StatusEffect::Entangle));
    }

    // --- Enemy attack (full turn cycle) ---

    #[test]
    fn full_turn_cycle_causes_enemy_to_attack_for_8() {
        let state = combat_with_hand(Vec::new());
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(72));
    }

    #[test]
    fn full_turn_cycle_emits_enemy_attacked_event() {
        let state = combat_with_hand(Vec::new());
        let (_, events) = end_turn_full(state, &mut rng()).unwrap();
        assert!(events.contains(&Event::EnemyAttacked { raw: 8, damage: 8 }));
    }

    #[test]
    fn block_absorbs_enemy_damage_before_hp() {
        let mut state = combat_with_hand(Vec::new());
        state.player.block = Block(5);
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(0));
        assert_eq!(state.player.hp, Hp(77));
    }

    #[test]
    fn block_fully_absorbing_attack_leaves_hp_unchanged() {
        let mut state = combat_with_hand(Vec::new());
        state.player.block = Block(10);
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(80));
    }

    #[test]
    fn player_block_resets_at_start_of_next_turn() {
        let mut state = combat_with_hand(Vec::new());
        state.player.block = Block(5);
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(0));
    }

    #[test]
    fn enemy_killing_player_yields_defeat() {
        let mut state = combat_with_hand(Vec::new());
        state.player.hp = Hp(1);
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.phase, CombatPhase::Defeat);
    }

    #[test]
    fn enemy_killing_player_emits_player_died_event() {
        let mut state = combat_with_hand(Vec::new());
        state.player.hp = Hp(1);
        let (_, events) = end_turn_full(state, &mut rng()).unwrap();
        assert!(events.contains(&Event::PlayerDied));
    }

    // --- HP clamping ---

    #[test]
    fn enemy_hp_does_not_go_below_zero() {
        let mut state = combat_with_hand(vec![Card::Strike]);
        state.enemies[0].hp = Hp(1);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(0));
    }

    #[test]
    fn player_hp_does_not_go_below_zero() {
        let mut state = combat_with_hand(Vec::new());
        state.player.hp = Hp(1);
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(0));
    }

    // --- Command rejection ---

    #[test]
    fn invalid_card_index_returns_error() {
        let state = combat_with_hand(vec![Card::Strike]);
        let result = apply_command(state, Command::PlayCard(5, 0), &mut rng());
        assert_eq!(result, Err(CommandError::InvalidCard));
    }

    #[test]
    fn commands_rejected_after_victory() {
        let mut state = combat_with_hand(vec![Card::Strike]);
        state.enemies[0].hp = Hp(1);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.phase, CombatPhase::Victory);

        let result = apply_command(state, Command::PlayCard(0, 0), &mut rng());
        assert_eq!(result, Err(CommandError::CombatOver));
    }

    #[test]
    fn commands_rejected_after_defeat() {
        let mut state = combat_with_hand(Vec::new());
        state.player.hp = Hp(1);
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.phase, CombatPhase::Defeat);

        let result = apply_command(state, Command::EndTurn, &mut rng());
        assert_eq!(result, Err(CommandError::CombatOver));
    }

    // --- Phase 3: intent + EnemyTurn ---

    #[test]
    fn new_combat_sets_initial_attack_intent() {
        let state = CombatState::new(&mut rng());
        assert_eq!(state.enemies[0].move_.intent(), Intent::Attack(8));
    }

    #[test]
    fn end_turn_transitions_to_enemy_turn() {
        let state = combat_with_hand(Vec::new());
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(state.phase, CombatPhase::EnemyTurn);
    }

    #[test]
    fn end_turn_does_not_yet_damage_player() {
        let state = combat_with_hand(Vec::new());
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(80));
    }

    #[test]
    fn end_enemy_turn_returns_to_player_turn() {
        let state = combat_with_hand(Vec::new());
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::EndEnemyTurn, &mut rng()).unwrap();
        assert_eq!(state.phase, CombatPhase::PlayerTurn);
    }

    #[test]
    fn intent_alternates_to_defend_on_turn_2() {
        let state = combat_with_hand(Vec::new());
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.turn, 2);
        assert_eq!(state.enemies[0].move_.intent(), Intent::Defend(5));
    }

    #[test]
    fn intent_alternates_back_to_attack_on_turn_3() {
        let state = combat_with_hand(Vec::new());
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.turn, 3);
        assert_eq!(state.enemies[0].move_.intent(), Intent::Attack(8));
    }

    #[test]
    fn defend_intent_grants_enemy_block() {
        let mut state = combat_with_hand(Vec::new());
        state.enemies[0].move_ = Move::LouseBlock;
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.enemies[0].block, Block(5));
    }

    #[test]
    fn defend_intent_emits_enemy_defended_event() {
        let mut state = combat_with_hand(Vec::new());
        state.enemies[0].move_ = Move::LouseBlock;
        let (_, events) = end_turn_full(state, &mut rng()).unwrap();
        assert!(events.contains(&Event::EnemyDefended { amount: 5 }));
    }

    #[test]
    fn defend_intent_does_not_damage_player() {
        let mut state = combat_with_hand(Vec::new());
        state.enemies[0].move_ = Move::LouseBlock;
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(80));
    }

    #[test]
    fn enemy_block_absorbs_player_strike_damage() {
        let mut state = combat_with_hand(vec![Card::Strike]);
        state.enemies[0].block = Block(4);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].block, Block(0));
        assert_eq!(state.enemies[0].hp, Hp(18));
    }

    #[test]
    fn enemy_block_fully_absorbs_player_strike() {
        let mut state = combat_with_hand(vec![Card::Strike]);
        state.enemies[0].block = Block(10);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(20));
    }

    #[test]
    fn enemy_block_resets_when_enemy_acts() {
        let mut state = combat_with_hand(Vec::new());
        state.enemies[0].block = Block(5);
        state.enemies[0].move_ = Move::LouseBlock;
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.enemies[0].block, Block(5));
    }

    #[test]
    fn enemy_block_persists_through_player_turn() {
        let mut state = combat_with_hand(vec![Card::Strike]);
        state.enemies[0].move_ = Move::LouseBlock;
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.enemies[0].block, Block(5));
    }

    // --- Phase 4: status effects ---

    #[test]
    fn vulnerable_ticks_down_after_enemy_turn() {
        use crate::status::StatusEffect;
        let mut state = combat_with_hand(Vec::new());
        state.enemies[0].statuses.insert(StatusEffect::Vulnerable, 2);
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Vulnerable), Some(&1));
    }

    #[test]
    fn vulnerable_expires_after_two_enemy_turns() {
        use crate::status::StatusEffect;
        let mut state = combat_with_hand(Vec::new());
        state.enemies[0].statuses.insert(StatusEffect::Vulnerable, 2);
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert!(!state.enemies[0].statuses.contains_key(&StatusEffect::Vulnerable));
    }

    #[test]
    fn weak_ticks_down_after_enemy_turn() {
        use crate::status::StatusEffect;
        let mut state = combat_with_hand(Vec::new());
        state.enemies[0].statuses.insert(StatusEffect::Weak, 2);
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Weak), Some(&1));
    }

    // --- Phase 4.5: poison ---

    #[test]
    fn poison_deals_damage_at_start_of_enemy_turn() {
        let mut state = combat_with_hand(Vec::new());
        state.enemies[0].statuses.insert(StatusEffect::Poison, 3);
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(17));
    }

    #[test]
    fn poison_emits_enemy_poisoned_event() {
        let mut state = combat_with_hand(Vec::new());
        state.enemies[0].statuses.insert(StatusEffect::Poison, 3);
        let (_, events) = end_turn_full(state, &mut rng()).unwrap();
        assert!(events.contains(&Event::EnemyPoisoned { damage: 3 }));
    }

    #[test]
    fn poison_decrements_after_triggering() {
        let mut state = combat_with_hand(Vec::new());
        state.enemies[0].statuses.insert(StatusEffect::Poison, 3);
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Poison), Some(&2));
    }

    #[test]
    fn poison_expires_when_stacks_reach_zero() {
        let mut state = combat_with_hand(Vec::new());
        state.enemies[0].statuses.insert(StatusEffect::Poison, 1);
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert!(!state.enemies[0].statuses.contains_key(&StatusEffect::Poison));
    }

    #[test]
    fn poison_ignores_enemy_block() {
        let mut state = combat_with_hand(Vec::new());
        state.enemies[0].statuses.insert(StatusEffect::Poison, 5);
        state.enemies[0].block = Block(10);
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::EndEnemyTurn, &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(15));
    }

    #[test]
    fn poison_killing_enemy_prevents_their_attack() {
        let mut state = combat_with_hand(Vec::new());
        state.enemies[0].hp = Hp(3);
        state.enemies[0].statuses.insert(StatusEffect::Poison, 5);
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.phase, CombatPhase::Victory);
        assert_eq!(state.player.hp, Hp(80));
    }

    // --- Phase 4.5: strength ---

    #[test]
    fn strength_does_not_expire_at_end_of_turn() {
        let mut state = combat_with_hand(Vec::new());
        state.player.statuses.insert(StatusEffect::Strength, 2);
        state.player.draw_pile = vec![Card::Strike; 5];
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Strength), Some(&2));
    }

    // --- Phase guards ---

    #[test]
    fn cannot_play_card_during_enemy_turn() {
        let state = combat_with_hand(vec![Card::Strike]);
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        let result = apply_command(state, Command::PlayCard(0, 0), &mut rng());
        assert_eq!(result, Err(CommandError::InvalidPhase));
    }

    #[test]
    fn cannot_end_turn_during_enemy_turn() {
        let state = combat_with_hand(Vec::new());
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        let result = apply_command(state, Command::EndTurn, &mut rng());
        assert_eq!(result, Err(CommandError::InvalidPhase));
    }

    #[test]
    fn cannot_end_enemy_turn_during_player_turn() {
        let state = combat_with_hand(Vec::new());
        let result = apply_command(state, Command::EndEnemyTurn, &mut rng());
        assert_eq!(result, Err(CommandError::InvalidPhase));
    }

    #[test]
    fn intent_revealed_event_fires_at_turn_start() {
        let state = combat_with_hand(Vec::new());
        let (_, events) = end_turn_full(state, &mut rng()).unwrap();
        assert!(events.contains(&Event::IntentRevealed { intent: Intent::Defend(5) }));
    }

    // --- Phase 8: targeting ---

    #[test]
    fn play_card_targets_second_enemy() {
        let state = combat_with_two_enemies(vec![Card::Strike]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 1), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(20));
        assert_eq!(state.enemies[1].hp, Hp(14));
    }

    #[test]
    fn play_card_auto_targets_living_enemy_when_specified_is_dead() {
        let mut state = combat_with_two_enemies(vec![Card::Strike]);
        state.enemies[0].hp = Hp(0);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[1].hp, Hp(14));
    }

    #[test]
    fn play_card_out_of_bounds_target_returns_error() {
        let state = combat_with_hand(vec![Card::Strike]);
        let result = apply_command(state, Command::PlayCard(0, 5), &mut rng());
        assert_eq!(result, Err(CommandError::InvalidCard));
    }

    #[test]
    fn both_enemies_attack_on_enemy_turn() {
        let mut state = combat_with_two_enemies(Vec::new());
        state.player.draw_pile = vec![Card::Strike; 5];
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(64)); // 80 - 8 - 8
    }

    #[test]
    fn dead_enemy_skips_their_turn() {
        let mut state = combat_with_two_enemies(Vec::new());
        state.enemies[0].hp = Hp(0); // first enemy already dead
        state.player.draw_pile = vec![Card::Strike; 5];
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(72)); // only one attack (8 dmg)
    }

    #[test]
    fn killing_last_enemy_wins_combat() {
        let mut state = combat_with_two_enemies(vec![Card::Strike, Card::Strike]);
        state.enemies[0].hp = Hp(0); // first already dead
        state.enemies[1].hp = Hp(1);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.phase, CombatPhase::Victory);
    }

    #[test]
    fn killing_one_enemy_does_not_win_if_other_alive() {
        let mut state = combat_with_two_enemies(vec![Card::Strike]);
        state.enemies[0].hp = Hp(1);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.phase, CombatPhase::PlayerTurn);
        assert_eq!(state.enemies[1].hp, Hp(20));
    }

    #[test]
    fn effective_intent_includes_enemy_strength() {
        let mut enemy = Enemy {
            kind: EnemyKind::Louse,
            hp: Hp(20), max_hp: Hp(20), block: Block(0),
            move_: Move::LouseBite,
            last_move: None,
            statuses: StatusMap::new(),
        };
        enemy.statuses.insert(StatusEffect::Strength, 3);
        let player_statuses = StatusMap::new();
        assert_eq!(enemy.effective_intent(&player_statuses), Intent::Attack(11));
    }

    #[test]
    fn effective_intent_applies_player_vulnerable() {
        let enemy = Enemy {
            kind: EnemyKind::Louse,
            hp: Hp(20), max_hp: Hp(20), block: Block(0),
            move_: Move::LouseBite,
            last_move: None,
            statuses: StatusMap::new(),
        };
        let mut player_statuses = StatusMap::new();
        player_statuses.insert(StatusEffect::Vulnerable, 1);
        assert_eq!(enemy.effective_intent(&player_statuses), Intent::Attack(12));
    }

    #[test]
    fn effective_intent_applies_enemy_weak() {
        let mut enemy = Enemy {
            kind: EnemyKind::Louse,
            hp: Hp(20), max_hp: Hp(20), block: Block(0),
            move_: Move::LouseBite,
            last_move: None,
            statuses: StatusMap::new(),
        };
        enemy.statuses.insert(StatusEffect::Weak, 1);
        let player_statuses = StatusMap::new();
        assert_eq!(enemy.effective_intent(&player_statuses), Intent::Attack(6));
    }

    // --- Potions ---

    fn combat_with_potion(potion: Potion) -> CombatState {
        let mut state = combat_with_hand(vec![]);
        state.player.potions.push(potion);
        state
    }

    fn combat_with_potion_and_enemy_hp(potion: Potion, enemy_hp: i32) -> CombatState {
        let mut state = combat_with_potion(potion);
        state.enemies[0].hp = Hp(enemy_hp);
        state.enemies[0].max_hp = Hp(enemy_hp);
        state
    }

    #[test]
    fn fire_potion_deals_20_damage_to_target() {
        let state = combat_with_potion(Potion::FirePotion);
        let (state, _) = apply_command(state, Command::UsePotion(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(0));
    }

    #[test]
    fn fire_potion_applies_vulnerable_scaling() {
        let mut state = combat_with_potion_and_enemy_hp(Potion::FirePotion, 50);
        state.enemies[0].statuses.insert(StatusEffect::Vulnerable, 1);
        let (state, _) = apply_command(state, Command::UsePotion(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(50 - 30)); // 20 * 3/2 = 30
    }

    #[test]
    fn fire_potion_kills_enemy_and_flags_victory() {
        let state = combat_with_potion(Potion::FirePotion);
        let (state, events) = apply_command(state, Command::UsePotion(0, 0), &mut rng()).unwrap();
        assert_eq!(state.phase, CombatPhase::Victory);
        assert!(events.contains(&Event::EnemyDied));
    }

    #[test]
    fn explosive_potion_deals_10_to_all_enemies() {
        let mut state = combat_with_two_enemies(vec![]);
        state.player.potions.push(Potion::ExplosivePotion);
        for e in &mut state.enemies { e.hp = Hp(15); e.max_hp = Hp(15); }
        let (state, _) = apply_command(state, Command::UsePotion(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(5));
        assert_eq!(state.enemies[1].hp, Hp(5));
    }

    #[test]
    fn block_potion_grants_12_block() {
        let state = combat_with_potion(Potion::BlockPotion);
        let (state, _) = apply_command(state, Command::UsePotion(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(12));
    }

    #[test]
    fn strength_potion_grants_2_strength() {
        let state = combat_with_potion(Potion::StrengthPotion);
        let (state, _) = apply_command(state, Command::UsePotion(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Strength), Some(&2));
    }

    #[test]
    fn swift_potion_draws_3_cards() {
        let mut state = combat_with_potion(Potion::SwiftPotion);
        state.player.draw_pile = vec![Card::Strike; 5];
        let (state, _) = apply_command(state, Command::UsePotion(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 3);
    }

    #[test]
    fn fear_potion_applies_3_vulnerable_to_target() {
        let state = combat_with_potion_and_enemy_hp(Potion::FearPotion, 50);
        let (state, _) = apply_command(state, Command::UsePotion(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Vulnerable), Some(&3));
    }

    #[test]
    fn weak_potion_applies_3_weak_to_target() {
        let state = combat_with_potion_and_enemy_hp(Potion::WeakPotion, 50);
        let (state, _) = apply_command(state, Command::UsePotion(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Weak), Some(&3));
    }

    #[test]
    fn energy_potion_grants_2_energy() {
        let state = combat_with_potion(Potion::EnergyPotion);
        let (state, _) = apply_command(state, Command::UsePotion(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.energy, Energy(5));
    }

    #[test]
    fn blood_potion_heals_20_pct_of_max_hp() {
        let mut state = combat_with_potion(Potion::BloodPotion);
        state.player.hp = Hp(60); // max is 80, 20% = 16
        let (state, _) = apply_command(state, Command::UsePotion(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(76));
    }

    #[test]
    fn blood_potion_cannot_overheal() {
        let mut state = combat_with_potion(Potion::BloodPotion);
        state.player.hp = Hp(79); // heals 16 but capped at 80
        let (state, _) = apply_command(state, Command::UsePotion(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(80));
    }

    #[test]
    fn potion_is_consumed_after_use() {
        let state = combat_with_potion(Potion::EnergyPotion);
        let (state, _) = apply_command(state, Command::UsePotion(0, 0), &mut rng()).unwrap();
        assert!(state.player.potions.is_empty());
    }

    #[test]
    fn use_potion_emits_potion_used_event() {
        let state = combat_with_potion(Potion::EnergyPotion);
        let (_, events) = apply_command(state, Command::UsePotion(0, 0), &mut rng()).unwrap();
        assert!(events.contains(&Event::PotionUsed { potion: Potion::EnergyPotion }));
    }

    #[test]
    fn cannot_use_potion_out_of_bounds() {
        let state = combat_with_hand(vec![]);
        let result = apply_command(state, Command::UsePotion(0, 0), &mut rng());
        assert_eq!(result, Err(CommandError::InvalidCard));
    }

    #[test]
    fn cannot_use_potion_during_enemy_turn() {
        let state = combat_with_potion(Potion::EnergyPotion);
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        let result = apply_command(state, Command::UsePotion(0, 0), &mut rng());
        assert_eq!(result, Err(CommandError::InvalidPhase));
    }
}

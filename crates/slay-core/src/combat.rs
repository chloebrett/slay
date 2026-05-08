use crate::cards::{Card, EndOfTurnHook};
use crate::enemies::{self, Effect, EnemyKind, Intent, Move};
use crate::potions::Potion;
use crate::relics::{apply_card_play_relics, apply_enemy_died_relics, Relic};
use crate::rng::Rng;
use crate::run::{Command, CommandError};
use crate::status::{StatusEffect, StatusMap, drain_poison, get_stacks, resolve_damage, tick_ritual, tick_statuses, tick_strength_modifiers};
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
    StartOfPlayerTurn,
    Victory,
    Defeat,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CombatState {
    pub player: Player,
    pub enemies: Vec<Enemy>,
    pub turn: u32,
    pub phase: CombatPhase,
    pub attacks_this_turn: u32,
    pub skills_this_turn: u32,
    pub attacks_this_combat: u32,
    pub skills_this_combat: u32,
    pub cards_played_this_turn: u32,
    pub extra_draws_next_turn: u32,
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
        Self::from_player(player, vec![EnemyKind::RedLouse], rng)
    }

    pub fn from_player(player: Player, enemy_kinds: Vec<EnemyKind>, rng: &mut impl Rng) -> Self {
        let mut draw_pile = player.deck.clone();
        rng.shuffle(&mut draw_pile);
        let (innate, rest): (Vec<_>, Vec<_>) = draw_pile.into_iter().partition(|c| c.is_innate());
        let innate_count = innate.len();
        let mut p = Player {
            draw_pile: rest,
            hand: innate,
            discard_pile: Vec::new(),
            exhaust_pile: Vec::new(),
            block: Block(0),
            energy: player.max_energy,
            statuses: StatusMap::new(),
            ..player
        };
        draw_cards(&mut p, 5usize.saturating_sub(innate_count), rng);
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
            attacks_this_turn: 0,
            skills_this_turn: 0,
            attacks_this_combat: 0,
            skills_this_combat: 0,
            cards_played_this_turn: 0,
            extra_draws_next_turn: 0,
        }
    }
}

pub(crate) fn draw_with_triggers(state: &mut CombatState, n: usize, events: &mut Vec<Event>, rng: &mut impl Rng) {
    use crate::cards::CardType;
    let before = state.player.hand.len();
    draw_cards(&mut state.player, n, rng);
    let after = state.player.hand.len();

    let status_drawn = state.player.hand[before..after].iter()
        .filter(|c| c.def().card_type == CardType::Status).count();
    let curse_drawn = state.player.hand[before..after].iter()
        .filter(|c| c.def().card_type == CardType::Curse).count();

    let fire_breathing = get_stacks(&state.player.statuses, StatusEffect::FireBreathing);
    if fire_breathing > 0 {
        let triggers = (status_drawn + curse_drawn) as i32;
        if triggers > 0 {
            damage_all_enemies(&mut state.enemies, events, fire_breathing * triggers);
        }
    }

    let evolve = get_stacks(&state.player.statuses, StatusEffect::Evolve);
    if evolve > 0 && status_drawn > 0 {
        for _ in 0..status_drawn {
            draw_with_triggers(state, evolve as usize, events, rng);
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
    PotionAwarded { potion: Potion },
    PotionDiscarded { potion: Potion },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Target {
    Player,
    Enemy,
}

fn resolve_target(enemies: &[Enemy], target_idx: usize) -> Result<usize, CommandError> {
    if target_idx >= enemies.len() {
        return Err(CommandError::InvalidCard);
    }
    if enemies[target_idx].hp > Hp(0) {
        return Ok(target_idx);
    }
    enemies.iter().position(|e| e.hp > Hp(0)).ok_or(CommandError::InvalidCard)
}

fn apply_play_card(
    mut state: CombatState,
    index: usize,
    target_idx: usize,
    rng: &mut impl Rng,
) -> Result<(CombatState, Vec<Event>), CommandError> {
    use crate::cards::CardType;
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
    if let Card::Clash(_) = &card {
        if !state.player.hand.iter().all(|c| c.card_type() == CardType::Attack) {
            return Err(CommandError::InvalidCard);
        }
    }
    if state.player.energy < card.energy_cost() {
        return Err(CommandError::NotEnoughEnergy);
    }
    if card.card_type() == CardType::Attack && state.player.statuses.contains_key(&StatusEffect::Entangle) {
        return Err(CommandError::Entangled);
    }
    let actual_target = resolve_target(&state.enemies, target_idx)?;
    let mut events = Vec::new();
    state.player.hand.remove(index);
    state.player.energy = Energy(state.player.energy.0 - card.energy_cost().0);
    events.push(Event::CardPlayed { card: card.clone() });
    let hp_before_card = state.enemies[actual_target].hp;
    crate::cards::apply(&card, &mut state, &mut events, actual_target, rng);
    if card.card_type() == CardType::Attack {
        let sharp_hide = get_stacks(&state.enemies[actual_target].statuses, StatusEffect::SharpHide);
        if sharp_hide > 0 {
            let raw = sharp_hide * 5;
            let damage = deal_damage(raw, &mut state.player.hp, &mut state.player.block);
            events.push(Event::EnemyAttacked { raw, damage });
        }
    }
    if state.enemies[actual_target].hp > Hp(0) {
        let hp_lost = (hp_before_card.0 - state.enemies[actual_target].hp.0).max(0);
        if let Some(reaction) = enemies::on_player_attack_damage(
            &state.enemies[actual_target].kind,
            &state.enemies[actual_target].statuses,
            hp_lost,
        ) {
            let enemy = &mut state.enemies[actual_target];
            enemy.block.0 += reaction.block_gain;
            for &(status, stacks) in &reaction.status_events {
                *enemy.statuses.entry(status).or_insert(0) += stacks;
            }
            for &(status, stacks) in &reaction.silent_adds {
                *enemy.statuses.entry(status).or_insert(0) += stacks;
            }
            for &(status, value) in &reaction.silent_sets {
                enemy.statuses.insert(status, value);
            }
            if let Some(mv) = reaction.force_move {
                enemy.move_ = mv;
            }
            if reaction.block_gain > 0 {
                events.push(Event::EnemyDefended { amount: reaction.block_gain });
            }
            for &(status, stacks) in &reaction.status_events {
                events.push(Event::StatusApplied { target: Target::Enemy, status, stacks });
            }
        }
    }
    if card.exhausts() {
        exhaust_card(card.clone(), &mut state, &mut events, rng);
    } else if card.card_type() != CardType::Power {
        state.player.discard_pile.push(card.clone());
    }
    let card_type = card.card_type();
    match card_type {
        CardType::Attack => { state.attacks_this_turn += 1; state.attacks_this_combat += 1; }
        CardType::Skill  => { state.skills_this_turn  += 1; state.skills_this_combat  += 1; }
        CardType::Power | CardType::Curse | CardType::Status => {}
    }
    state.cards_played_this_turn += 1;
    apply_card_play_relics(&mut state, &mut events, card_type, rng);
    if state.player.hp <= Hp(0) {
        state.phase = CombatPhase::Defeat;
        return Ok((state, events));
    }
    if state.enemies[actual_target].hp <= Hp(0) {
        events.push(Event::EnemyDied);
        apply_enemy_died_relics(&mut state, &mut events, rng);
    }
    if state.enemies.iter().all(|e| e.hp <= Hp(0)) {
        state.phase = CombatPhase::Victory;
    }
    Ok((state, events))
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
            return apply_play_card(state, index, target_idx, rng);
        }
        Command::EndTurn => {
            if state.phase != CombatPhase::PlayerTurn {
                return Err(CommandError::InvalidPhase);
            }
            events.push(Event::TurnEnded);
            let hand_size = state.player.hand.len() as i32;
            let hooks: Vec<EndOfTurnHook> = state.player.hand.iter()
                .filter_map(|c| c.end_of_turn_hook(hand_size))
                .collect();
            let (ethereal, normal): (Vec<_>, Vec<_>) = state.player.hand.drain(..).partition(|c| c.is_ethereal());
            for card in ethereal { exhaust_card(card, &mut state, &mut events, rng); }
            state.player.discard_pile.extend(normal);
            tick_statuses(&mut state.player.statuses);
            let strength_delta = tick_strength_modifiers(&mut state.player.statuses);
            if strength_delta != 0 {
                apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Strength, strength_delta, &mut events);
            }
            if apply_end_of_turn_card_hooks(&hooks, &mut state, &mut events) {
                return Ok((state, events));
            }
            // Combust: lose 1 HP, deal damage to all enemies
            let combust = get_stacks(&state.player.statuses, StatusEffect::Combust);
            if combust > 0 {
                damage_player(&mut state, &mut events, 1);
                if state.player.hp <= Hp(0) {
                    state.phase = CombatPhase::Defeat;
                    return Ok((state, events));
                }
                damage_all_enemies(&mut state.enemies, &mut events, combust);
                if state.enemies.iter().all(|e| e.hp <= Hp(0)) {
                    state.phase = CombatPhase::Victory;
                    return Ok((state, events));
                }
            }
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
            crate::potions::apply(potion, target_idx, &mut state, &mut events, rng);
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
        | Command::DiscardPotion(_)
        | Command::Spawn(_)
        | Command::LeaveShop
        | Command::BuyCard(_)
        | Command::BuyRelic
        | Command::BuyPotion
        | Command::LeaveTreasure
        | Command::ChooseEventOption(_) => {
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
                        Effect::ClearSelfStatus(status) => {
                            state.enemies[i].statuses.remove(&status);
                        }
                    }
                }
                tick_statuses(&mut state.enemies[i].statuses);
                let strength_delta = tick_strength_modifiers(&mut state.enemies[i].statuses);
                if strength_delta != 0 {
                    apply_status(&mut state.enemies[i].statuses, Target::Enemy, StatusEffect::Strength, strength_delta, &mut events);
                }
                let last = state.enemies[i].move_;
                state.enemies[i].last_move = Some(last);
            }
            if state.player.hp <= Hp(0) {
                state.phase = CombatPhase::Defeat;
                events.push(Event::PlayerDied);
            } else {
                state.phase = CombatPhase::StartOfPlayerTurn;
            }
        }
        Command::StartPlayerTurn => {
            if state.phase != CombatPhase::StartOfPlayerTurn {
                return Err(CommandError::InvalidPhase);
            }
            let barricade = state.player.statuses.contains_key(&StatusEffect::Barricade);
            if !barricade {
                if state.player.block > Block(0) {
                    events.push(Event::PlayerBlockExpired { amount: state.player.block.0 });
                }
                state.player.block = Block(0);
            }
            state.player.energy = state.player.max_energy;
            state.turn += 1;
            for enemy in state.enemies.iter_mut() {
                if enemy.hp > Hp(0) {
                    enemy.move_ = enemies::next_move(&enemy.kind, enemy.last_move, rng);
                    events.push(Event::IntentRevealed { intent: enemy.move_.intent() });
                }
            }
            let extra = state.extra_draws_next_turn as usize;
            state.attacks_this_turn = 0;
            state.skills_this_turn = 0;
            state.cards_played_this_turn = 0;
            state.extra_draws_next_turn = 0;
            // Start-of-turn power effects
            let demon_form = get_stacks(&state.player.statuses, StatusEffect::DemonForm);
            if demon_form > 0 {
                apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Strength, demon_form, &mut events);
            }
            let berserk = get_stacks(&state.player.statuses, StatusEffect::Berserk);
            if berserk > 0 {
                state.player.energy.0 += 1;
                events.push(Event::EnergyGained { amount: 1 });
            }
            let brutality = get_stacks(&state.player.statuses, StatusEffect::Brutality);
            if brutality > 0 {
                damage_player(&mut state, &mut events, 1);
                draw_with_triggers(&mut state, 1, &mut events, rng);
                events.push(Event::CardsDrawn { count: 1 });
            }
            draw_with_triggers(&mut state, 5, &mut events, rng);
            if extra > 0 {
                draw_with_triggers(&mut state, extra, &mut events, rng);
                events.push(Event::CardsDrawn { count: extra });
            }
            state.phase = CombatPhase::PlayerTurn;
            events.push(Event::TurnStarted { turn: state.turn });
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

/// Exhausts a card: pushes to exhaust_pile, emits CardExhausted, fires on-exhaust power hooks.
pub(crate) fn exhaust_card(card: crate::cards::Card, state: &mut CombatState, events: &mut Vec<Event>, rng: &mut impl Rng) {
    events.push(Event::CardExhausted { card: card.clone() });
    state.player.exhaust_pile.push(card);
    let feel_no_pain = get_stacks(&state.player.statuses, StatusEffect::FeelNoPain);
    if feel_no_pain > 0 {
        gain_player_block(state, events, feel_no_pain, rng);
    }
    let dark_embrace = get_stacks(&state.player.statuses, StatusEffect::DarkEmbrace);
    if dark_embrace > 0 {
        draw_cards(&mut state.player, 1, rng);
        events.push(Event::CardsDrawn { count: 1 });
    }
}

/// Gains block for the player: adds amount, emits PlayerBlocked, fires on-block-gain power hooks.
pub(crate) fn gain_player_block(state: &mut CombatState, events: &mut Vec<Event>, amount: i32, rng: &mut impl Rng) {
    if amount <= 0 { return; }
    state.player.block.0 += amount;
    events.push(Event::PlayerBlocked { amount });
    let juggernaut = get_stacks(&state.player.statuses, StatusEffect::Juggernaut);
    if juggernaut > 0 {
        let mut living: Vec<usize> = (0..state.enemies.len())
            .filter(|&i| state.enemies[i].hp.0 > 0)
            .collect();
        if !living.is_empty() {
            rng.shuffle(&mut living);
            let t = living[0];
            let e = &mut state.enemies[t];
            let dmg = deal_damage(juggernaut, &mut e.hp, &mut e.block);
            events.push(Event::PlayerAttacked { raw: juggernaut, damage: dmg });
        }
    }
}

/// Loses HP from a card effect: subtracts HP, emits PlayerSelfDamaged, fires on-player-turn-hp-loss hooks.
pub(crate) fn damage_player_from_card(state: &mut CombatState, events: &mut Vec<Event>, amount: i32) {
    state.player.hp.0 = (state.player.hp.0 - amount).max(0);
    events.push(Event::PlayerSelfDamaged { amount });
    let rupture = get_stacks(&state.player.statuses, StatusEffect::Rupture);
    if rupture > 0 {
        apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Strength, rupture, events);
    }
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

pub(crate) fn damage_all_enemies(enemies: &mut Vec<Enemy>, events: &mut Vec<Event>, base_damage: i32) {
    for i in 0..enemies.len() {
        if enemies[i].hp <= Hp(0) { continue; }
        let dmg = resolve_damage(base_damage, &StatusMap::new(), &enemies[i].statuses);
        let e = &mut enemies[i];
        let dealt = deal_damage(dmg, &mut e.hp, &mut e.block);
        events.push(Event::PlayerAttacked { raw: dmg, damage: dealt });
        if enemies[i].hp <= Hp(0) {
            events.push(Event::EnemyDied);
        }
    }
}

fn apply_end_of_turn_card_hooks(hooks: &[EndOfTurnHook], state: &mut CombatState, events: &mut Vec<Event>) -> bool {
    for &hook in hooks {
        match hook {
            EndOfTurnHook::BlockableDamage(amount) => {
                let dealt = deal_damage(amount, &mut state.player.hp, &mut state.player.block);
                events.push(Event::PlayerAttacked { raw: amount, damage: dealt });
            }
            EndOfTurnHook::DirectHpLoss(amount) => {
                damage_player(state, events, amount);
            }
            EndOfTurnHook::ApplyPlayerStatus { effect, amount } => {
                apply_status(&mut state.player.statuses, Target::Player, effect, amount, events);
            }
        }
        if state.player.hp <= Hp(0) {
            state.phase = CombatPhase::Defeat;
            return true;
        }
    }
    false
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
            kind: EnemyKind::RedLouse,
            hp: Hp(20),
            max_hp: Hp(20),
            block: Block(0),
            move_: Move::RedLouseBite,
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
    }
}

#[cfg(test)]
pub(crate) fn combat_with_deck(deck: Vec<Card>, rng: &mut impl Rng) -> CombatState {
    let player = Player {
        hp: Hp(80), max_hp: Hp(80), block: Block(0),
        energy: Energy(3), max_energy: Energy(3),
        hand: Vec::new(), draw_pile: Vec::new(),
        discard_pile: Vec::new(), exhaust_pile: Vec::new(),
        statuses: StatusMap::new(), gold: 0,
        relics: Vec::new(), potions: Vec::new(),
        deck,
    };
    CombatState::from_player(player, vec![EnemyKind::RedLouse], rng)
}

#[cfg(test)]
pub(crate) fn combat_with_two_enemies(hand: Vec<Card>) -> CombatState {
    let louse = || Enemy {
        kind: EnemyKind::RedLouse,
        hp: Hp(20),
        max_hp: Hp(20),
        block: Block(0),
        move_: Move::RedLouseBite,
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
        attacks_this_turn: 0,
        skills_this_turn: 0,
        attacks_this_combat: 0,
        skills_this_combat: 0,
        cards_played_this_turn: 0,
        extra_draws_next_turn: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::Grade;
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
        if state.phase != CombatPhase::StartOfPlayerTurn {
            return Ok((state, events));
        }
        let (state, more) = apply_command(state, Command::StartPlayerTurn, rng)?;
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
                    Card::Strike(Grade::Base),
                    Card::Strike(Grade::Base),
                    Card::Strike(Grade::Base),
                    Card::Strike(Grade::Base),
                    Card::Strike(Grade::Base),
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
        let mut state = combat_with_hand(vec![Card::Strike(Grade::Base), Card::Defend(Grade::Base)]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base); 5];
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 5);
        assert!(state.player.discard_pile.contains(&Card::Strike(Grade::Base)));
        assert!(state.player.discard_pile.contains(&Card::Defend(Grade::Base)));
    }

    #[test]
    fn empty_draw_pile_shuffles_discard_when_drawing() {
        let mut state = combat_with_hand(Vec::new());
        state.player.discard_pile = vec![Card::Strike(Grade::Base), Card::Defend(Grade::Base), Card::Strike(Grade::Base)];
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
        let state = combat_with_hand(vec![Card::Strike(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.energy, Energy(2));
    }

    #[test]
    fn energy_resets_at_start_of_next_turn() {
        let mut state = combat_with_hand(vec![Card::Strike(Grade::Base)]);
        state.player.energy = Energy(0);
        state.player.draw_pile = vec![Card::Strike(Grade::Base); 5];
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.player.energy, Energy(3));
    }

    #[test]
    fn cannot_play_card_without_sufficient_energy() {
        let mut state = combat_with_hand(vec![Card::Strike(Grade::Base)]);
        state.player.energy = Energy(0);
        let result = apply_command(state, Command::PlayCard(0, 0), &mut rng());
        assert_eq!(result, Err(CommandError::NotEnoughEnergy));
    }

    #[test]
    fn entangle_prevents_playing_attack_cards() {
        let mut state = combat_with_hand(vec![Card::Strike(Grade::Base)]);
        state.player.statuses.insert(StatusEffect::Entangle, 1);
        let result = apply_command(state, Command::PlayCard(0, 0), &mut rng());
        assert_eq!(result, Err(CommandError::Entangled));
    }

    #[test]
    fn entangle_does_not_prevent_playing_skill_cards() {
        let mut state = combat_with_hand(vec![Card::Defend(Grade::Base)]);
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
    fn full_turn_cycle_causes_enemy_to_attack_for_6() {
        let state = combat_with_hand(Vec::new());
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(74));
    }

    #[test]
    fn full_turn_cycle_emits_enemy_attacked_event() {
        let state = combat_with_hand(Vec::new());
        let (_, events) = end_turn_full(state, &mut rng()).unwrap();
        assert!(events.contains(&Event::EnemyAttacked { raw: 6, damage: 6 }));
    }

    #[test]
    fn block_absorbs_enemy_damage_before_hp() {
        let mut state = combat_with_hand(Vec::new());
        state.player.block = Block(5);
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(0));
        assert_eq!(state.player.hp, Hp(79)); // 6 damage - 5 block = 1 hp lost
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
        let mut state = combat_with_hand(vec![Card::Strike(Grade::Base)]);
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
        let state = combat_with_hand(vec![Card::Strike(Grade::Base)]);
        let result = apply_command(state, Command::PlayCard(5, 0), &mut rng());
        assert_eq!(result, Err(CommandError::InvalidCard));
    }

    #[test]
    fn commands_rejected_after_victory() {
        let mut state = combat_with_hand(vec![Card::Strike(Grade::Base)]);
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
        assert_eq!(state.enemies[0].move_.intent(), Intent::Attack(6));
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
        assert_eq!(state.phase, CombatPhase::StartOfPlayerTurn);
        let (state, _) = apply_command(state, Command::StartPlayerTurn, &mut rng()).unwrap();
        assert_eq!(state.phase, CombatPhase::PlayerTurn);
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
        let mut state = combat_with_hand(vec![Card::Strike(Grade::Base)]);
        state.enemies[0].block = Block(4);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].block, Block(0));
        assert_eq!(state.enemies[0].hp, Hp(18));
    }

    #[test]
    fn enemy_block_fully_absorbs_player_strike() {
        let mut state = combat_with_hand(vec![Card::Strike(Grade::Base)]);
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
        let mut state = combat_with_hand(vec![Card::Strike(Grade::Base)]);
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
        state.player.draw_pile = vec![Card::Strike(Grade::Base); 5];
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Strength), Some(&2));
    }

    // --- StrengthDown / Shackled ---

    #[test]
    fn strength_down_reduces_player_strength_at_end_of_turn() {
        let mut state = combat_with_hand(Vec::new());
        state.player.statuses.insert(StatusEffect::Strength, 5);
        state.player.statuses.insert(StatusEffect::StrengthDown, 2);
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Strength).copied(), Some(3));
    }

    #[test]
    fn strength_down_clears_after_player_end_of_turn() {
        let mut state = combat_with_hand(Vec::new());
        state.player.statuses.insert(StatusEffect::StrengthDown, 2);
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert!(!state.player.statuses.contains_key(&StatusEffect::StrengthDown));
    }

    #[test]
    fn shackled_increases_enemy_strength_at_end_of_enemy_turn() {
        let mut state = combat_with_hand(Vec::new());
        state.phase = CombatPhase::EnemyTurn;
        state.enemies[0].statuses.insert(StatusEffect::Shackled, 2);
        let (state, _) = apply_command(state, Command::EndEnemyTurn, &mut rng()).unwrap();
        assert_eq!(get_stacks(&state.enemies[0].statuses, StatusEffect::Strength), 2);
    }

    #[test]
    fn shackled_clears_after_end_of_enemy_turn() {
        let mut state = combat_with_hand(Vec::new());
        state.phase = CombatPhase::EnemyTurn;
        state.enemies[0].statuses.insert(StatusEffect::Shackled, 2);
        let (state, _) = apply_command(state, Command::EndEnemyTurn, &mut rng()).unwrap();
        assert!(!state.enemies[0].statuses.contains_key(&StatusEffect::Shackled));
    }

    // --- Phase guards ---

    #[test]
    fn cannot_play_card_during_enemy_turn() {
        let state = combat_with_hand(vec![Card::Strike(Grade::Base)]);
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
        assert!(events.contains(&Event::IntentRevealed { intent: Intent::Attack(6) }));
    }

    // --- Phase 8: targeting ---

    #[test]
    fn play_card_targets_second_enemy() {
        let state = combat_with_two_enemies(vec![Card::Strike(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 1), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(20));
        assert_eq!(state.enemies[1].hp, Hp(14));
    }

    #[test]
    fn play_card_auto_targets_living_enemy_when_specified_is_dead() {
        let mut state = combat_with_two_enemies(vec![Card::Strike(Grade::Base)]);
        state.enemies[0].hp = Hp(0);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[1].hp, Hp(14));
    }

    #[test]
    fn play_card_out_of_bounds_target_returns_error() {
        let state = combat_with_hand(vec![Card::Strike(Grade::Base)]);
        let result = apply_command(state, Command::PlayCard(0, 5), &mut rng());
        assert_eq!(result, Err(CommandError::InvalidCard));
    }

    #[test]
    fn both_enemies_attack_on_enemy_turn() {
        let mut state = combat_with_two_enemies(Vec::new());
        state.player.draw_pile = vec![Card::Strike(Grade::Base); 5];
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(68)); // 80 - 6 - 6
    }

    #[test]
    fn dead_enemy_skips_their_turn() {
        let mut state = combat_with_two_enemies(Vec::new());
        state.enemies[0].hp = Hp(0); // first enemy already dead
        state.player.draw_pile = vec![Card::Strike(Grade::Base); 5];
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(74)); // only one attack (6 dmg)
    }

    #[test]
    fn killing_last_enemy_wins_combat() {
        let mut state = combat_with_two_enemies(vec![Card::Strike(Grade::Base), Card::Strike(Grade::Base)]);
        state.enemies[0].hp = Hp(0); // first already dead
        state.enemies[1].hp = Hp(1);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.phase, CombatPhase::Victory);
    }

    #[test]
    fn killing_one_enemy_does_not_win_if_other_alive() {
        let mut state = combat_with_two_enemies(vec![Card::Strike(Grade::Base)]);
        state.enemies[0].hp = Hp(1);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.phase, CombatPhase::PlayerTurn);
        assert_eq!(state.enemies[1].hp, Hp(20));
    }

    #[test]
    fn effective_intent_includes_enemy_strength() {
        let mut enemy = Enemy {
            kind: EnemyKind::RedLouse,
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
            kind: EnemyKind::RedLouse,
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
            kind: EnemyKind::RedLouse,
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
    fn fire_potion_emits_player_attacked_not_enemy_attacked() {
        let state = combat_with_potion_and_enemy_hp(Potion::FirePotion, 50);
        let (_, events) = apply_command(state, Command::UsePotion(0, 0), &mut rng()).unwrap();
        assert!(events.iter().any(|e| matches!(e, Event::PlayerAttacked { .. })));
        assert!(!events.iter().any(|e| matches!(e, Event::EnemyAttacked { .. })));
    }

    #[test]
    fn explosive_potion_emits_player_attacked_not_enemy_attacked() {
        let mut state = combat_with_two_enemies(vec![]);
        state.player.potions.push(Potion::ExplosivePotion);
        for e in &mut state.enemies { e.hp = Hp(50); e.max_hp = Hp(50); }
        let (_, events) = apply_command(state, Command::UsePotion(0, 0), &mut rng()).unwrap();
        assert!(events.iter().any(|e| matches!(e, Event::PlayerAttacked { .. })));
        assert!(!events.iter().any(|e| matches!(e, Event::EnemyAttacked { .. })));
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
        state.player.draw_pile = vec![Card::Strike(Grade::Base); 5];
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

    // --- The Guardian: Sharp Hide ---

    fn guardian_combat(hand: Vec<Card>) -> CombatState {
        use crate::status::StatusMap;
        CombatState {
            player: Player {
                hp: Hp(80),
                max_hp: Hp(80),
                block: Block(0),
                energy: Energy(10),
                max_energy: Energy(10),
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
                kind: EnemyKind::TheGuardian,
                hp: Hp(240),
                max_hp: Hp(240),
                block: Block(0),
                move_: Move::GuardianChargingUp,
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
        }
    }

    #[test]
    fn sharp_hide_damages_player_when_playing_attack() {
        use crate::status::StatusEffect;
        let mut state = guardian_combat(vec![Card::Strike(Grade::Base)]);
        state.enemies[0].statuses.insert(StatusEffect::SharpHide, 2);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(70)); // 80 - 10 (2 stacks × 5)
    }

    #[test]
    fn sharp_hide_damage_absorbed_by_player_block() {
        use crate::status::StatusEffect;
        let mut state = guardian_combat(vec![Card::Strike(Grade::Base)]);
        state.player.block = Block(8);
        state.enemies[0].statuses.insert(StatusEffect::SharpHide, 2);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(78)); // 10 - 8 block = 2 hp damage
    }

    #[test]
    fn sharp_hide_triggers_once_for_multi_hit_card() {
        use crate::status::StatusEffect;
        let mut state = guardian_combat(vec![Card::TwinStrike(Grade::Base)]);
        state.enemies[0].statuses.insert(StatusEffect::SharpHide, 3);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(65)); // 80 - 15 (once per card, not per hit)
    }

    #[test]
    fn sharp_hide_does_not_trigger_for_skill_card() {
        use crate::status::StatusEffect;
        let mut state = guardian_combat(vec![Card::Defend(Grade::Base)]);
        state.enemies[0].statuses.insert(StatusEffect::SharpHide, 3);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(80)); // no sharp hide damage
    }

    #[test]
    fn sharp_hide_emits_enemy_attacked_event() {
        use crate::status::StatusEffect;
        let mut state = guardian_combat(vec![Card::Strike(Grade::Base)]);
        state.enemies[0].statuses.insert(StatusEffect::SharpHide, 2);
        let (_, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(events.contains(&Event::EnemyAttacked { raw: 10, damage: 10 }));
    }

    // --- The Guardian: Mode Shift ---

    #[test]
    fn mode_shift_triggers_at_30_cumulative_damage() {
        use crate::status::StatusEffect;
        let mut state = guardian_combat(vec![
            Card::Strike(Grade::Base), Card::Strike(Grade::Base), Card::Strike(Grade::Base),
            Card::Strike(Grade::Base), Card::Strike(Grade::Base),
        ]);
        for _ in 0..5 {
            state = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap().0;
        }
        let mode = get_stacks(&state.enemies[0].statuses, StatusEffect::GuardianMode);
        assert_eq!(mode, 1); // Defensive
    }

    #[test]
    fn mode_shift_grants_20_block() {
        use crate::status::StatusEffect;
        let mut state = guardian_combat(vec![
            Card::Strike(Grade::Base), Card::Strike(Grade::Base), Card::Strike(Grade::Base),
            Card::Strike(Grade::Base), Card::Strike(Grade::Base),
        ]);
        for _ in 0..5 {
            state = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap().0;
        }
        assert_eq!(state.enemies[0].block, Block(20));
    }

    #[test]
    fn mode_shift_grants_3_sharp_hide() {
        use crate::status::StatusEffect;
        let mut state = guardian_combat(vec![
            Card::Strike(Grade::Base), Card::Strike(Grade::Base), Card::Strike(Grade::Base),
            Card::Strike(Grade::Base), Card::Strike(Grade::Base),
        ]);
        for _ in 0..5 {
            state = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap().0;
        }
        let sharp_hide = get_stacks(&state.enemies[0].statuses, StatusEffect::SharpHide);
        assert_eq!(sharp_hide, 3);
    }

    #[test]
    fn mode_shift_sets_move_to_roll_attack() {
        use crate::status::StatusEffect;
        let mut state = guardian_combat(vec![
            Card::Strike(Grade::Base), Card::Strike(Grade::Base), Card::Strike(Grade::Base),
            Card::Strike(Grade::Base), Card::Strike(Grade::Base),
        ]);
        for _ in 0..5 {
            state = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap().0;
        }
        assert_eq!(state.enemies[0].move_, Move::GuardianRollAttack);
    }

    #[test]
    fn mode_shift_resets_progress_to_zero() {
        use crate::status::StatusEffect;
        let mut state = guardian_combat(vec![
            Card::Strike(Grade::Base), Card::Strike(Grade::Base), Card::Strike(Grade::Base),
            Card::Strike(Grade::Base), Card::Strike(Grade::Base),
        ]);
        for _ in 0..5 {
            state = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap().0;
        }
        let progress = state.enemies[0].statuses.get(&StatusEffect::ModeShiftProgress).copied().unwrap_or(99);
        assert_eq!(progress, 0);
    }

    #[test]
    fn mode_shift_increments_shift_count_to_one() {
        use crate::status::StatusEffect;
        let mut state = guardian_combat(vec![
            Card::Strike(Grade::Base), Card::Strike(Grade::Base), Card::Strike(Grade::Base),
            Card::Strike(Grade::Base), Card::Strike(Grade::Base),
        ]);
        for _ in 0..5 {
            state = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap().0;
        }
        let count = get_stacks(&state.enemies[0].statuses, StatusEffect::ModeShiftCount);
        assert_eq!(count, 1);
    }

    #[test]
    fn mode_shift_does_not_trigger_in_defensive_mode() {
        use crate::status::StatusEffect;
        let mut state = guardian_combat(vec![
            Card::Strike(Grade::Base), Card::Strike(Grade::Base), Card::Strike(Grade::Base),
            Card::Strike(Grade::Base), Card::Strike(Grade::Base),
        ]);
        state.enemies[0].statuses.insert(StatusEffect::GuardianMode, 1); // already Defensive
        for _ in 0..5 {
            state = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap().0;
        }
        let mode = get_stacks(&state.enemies[0].statuses, StatusEffect::GuardianMode);
        assert_eq!(mode, 1); // stays Defensive, no second shift
        let sharp_hide = get_stacks(&state.enemies[0].statuses, StatusEffect::SharpHide);
        assert_eq!(sharp_hide, 0); // no sharp hide gained from mode shift
    }

    #[test]
    fn second_mode_shift_triggers_at_40_hp_loss() {
        use crate::status::StatusEffect;
        // ModeShiftCount=1 → threshold=40. Pre-set progress to 34, Strike deals 6 → 40 = threshold.
        let mut state = guardian_combat(vec![Card::Strike(Grade::Base)]);
        state.enemies[0].statuses.insert(StatusEffect::ModeShiftCount, 1);
        state.enemies[0].statuses.insert(StatusEffect::ModeShiftProgress, 34);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let mode = get_stacks(&state.enemies[0].statuses, StatusEffect::GuardianMode);
        assert_eq!(mode, 1); // shifted to Defensive at 40
    }

    #[test]
    fn second_mode_shift_does_not_trigger_below_40() {
        use crate::status::StatusEffect;
        // ModeShiftCount=1 → threshold=40. Progress 29 + 6 = 35 < 40 → no shift.
        let mut state = guardian_combat(vec![Card::Strike(Grade::Base)]);
        state.enemies[0].statuses.insert(StatusEffect::ModeShiftCount, 1);
        state.enemies[0].statuses.insert(StatusEffect::ModeShiftProgress, 29);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let mode = get_stacks(&state.enemies[0].statuses, StatusEffect::GuardianMode);
        assert_eq!(mode, 0); // still Offensive, threshold not reached
    }

    // --- Carnage ---

    #[test]
    fn carnage_deals_20_damage() {
        let mut state = combat_with_hand(vec![Card::Carnage(Grade::Base)]);
        state.enemies[0].hp = Hp(100); state.enemies[0].max_hp = Hp(100);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(80));
    }

    #[test]
    fn carnage_plus_deals_28_damage() {
        let mut state = combat_with_hand(vec![Card::Carnage(Grade::Plus)]);
        state.enemies[0].hp = Hp(100); state.enemies[0].max_hp = Hp(100);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(72));
    }

    #[test]
    fn carnage_exhausts_at_end_of_turn_if_unplayed() {
        let mut state = combat_with_hand(vec![Card::Carnage(Grade::Base)]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base); 5];
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.contains(&Card::Carnage(Grade::Base)));
        assert!(!state.player.discard_pile.contains(&Card::Carnage(Grade::Base)));
    }

    // --- Clash ---

    #[test]
    fn clash_deals_14_damage_when_hand_all_attacks() {
        let mut state = combat_with_hand(vec![Card::Clash(Grade::Base)]);
        state.enemies[0].hp = Hp(100); state.enemies[0].max_hp = Hp(100);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(86));
    }

    #[test]
    fn clash_plus_deals_18_damage() {
        let mut state = combat_with_hand(vec![Card::Clash(Grade::Plus)]);
        state.enemies[0].hp = Hp(100); state.enemies[0].max_hp = Hp(100);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(82));
    }

    #[test]
    fn clash_is_invalid_when_hand_contains_skill() {
        let state = combat_with_hand(vec![Card::Clash(Grade::Base), Card::Defend(Grade::Base)]);
        let result = apply_command(state, Command::PlayCard(0, 0), &mut rng());
        assert_eq!(result, Err(CommandError::InvalidCard));
    }

    #[test]
    fn clash_is_playable_when_only_card_in_hand() {
        let mut state = combat_with_hand(vec![Card::Clash(Grade::Base)]);
        state.enemies[0].hp = Hp(100); state.enemies[0].max_hp = Hp(100);
        assert!(apply_command(state, Command::PlayCard(0, 0), &mut rng()).is_ok());
    }

    // --- Wild Strike ---

    #[test]
    fn wild_strike_deals_12_damage() {
        let mut state = combat_with_hand(vec![Card::WildStrike(Grade::Base)]);
        state.enemies[0].hp = Hp(100); state.enemies[0].max_hp = Hp(100);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(88));
    }

    #[test]
    fn wild_strike_plus_deals_17_damage() {
        let mut state = combat_with_hand(vec![Card::WildStrike(Grade::Plus)]);
        state.enemies[0].hp = Hp(100); state.enemies[0].max_hp = Hp(100);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(83));
    }

    #[test]
    fn wild_strike_shuffles_wound_into_draw_pile() {
        let state = combat_with_hand(vec![Card::WildStrike(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.draw_pile.contains(&Card::Wound));
    }

    // --- Heavy Blade ---

    #[test]
    fn heavy_blade_deals_14_damage_with_no_strength() {
        let mut state = combat_with_hand(vec![Card::HeavyBlade(Grade::Base)]);
        state.enemies[0].hp = Hp(100); state.enemies[0].max_hp = Hp(100);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(86));
    }

    #[test]
    fn heavy_blade_plus_deals_21_damage_with_no_strength() {
        let mut state = combat_with_hand(vec![Card::HeavyBlade(Grade::Plus)]);
        state.enemies[0].hp = Hp(100); state.enemies[0].max_hp = Hp(100);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(79));
    }

    #[test]
    fn heavy_blade_applies_strength_3_times() {
        use crate::status::StatusEffect;
        let mut state = combat_with_hand(vec![Card::HeavyBlade(Grade::Base)]);
        state.player.statuses.insert(StatusEffect::Strength, 2);
        state.enemies[0].hp = Hp(100); state.enemies[0].max_hp = Hp(100);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(80)); // 14 + 2*3 = 20
    }

    #[test]
    fn heavy_blade_plus_applies_strength_5_times() {
        use crate::status::StatusEffect;
        let mut state = combat_with_hand(vec![Card::HeavyBlade(Grade::Plus)]);
        state.player.statuses.insert(StatusEffect::Strength, 2);
        state.enemies[0].hp = Hp(100); state.enemies[0].max_hp = Hp(100);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(69)); // 21 + 2*5 = 31
    }

    // --- Sword Boomerang ---

    #[test]
    fn sword_boomerang_deals_3_damage_3_times() {
        let mut state = combat_with_hand(vec![Card::SwordBoomerang(Grade::Base)]);
        state.enemies[0].hp = Hp(100); state.enemies[0].max_hp = Hp(100);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(91)); // 3 * 3 = 9
    }

    #[test]
    fn sword_boomerang_plus_deals_3_damage_4_times() {
        let mut state = combat_with_hand(vec![Card::SwordBoomerang(Grade::Plus)]);
        state.enemies[0].hp = Hp(100); state.enemies[0].max_hp = Hp(100);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(88)); // 4 * 3 = 12
    }

    #[test]
    fn twin_slam_resets_guardian_mode_to_offensive() {
        use crate::status::StatusEffect;
        let mut state = guardian_combat(Vec::new());
        state.enemies[0].move_ = Move::GuardianTwinSlam;
        state.enemies[0].statuses.insert(StatusEffect::GuardianMode, 1); // Defensive
        state.player.draw_pile = vec![Card::Strike(Grade::Base); 5];
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        let mode = get_stacks(&state.enemies[0].statuses, StatusEffect::GuardianMode);
        assert_eq!(mode, 0); // back to Offensive after TwinSlam
    }
}

use crate::cards::Card;
use crate::enemies::{self, EnemyKind, Intent};
use crate::rng::Rng;
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub hp: Hp,
    pub max_hp: Hp,
    pub block: Block,
    pub intent: Intent,
}

impl Enemy {
    pub fn name(&self) -> &'static str { self.kind.name() }
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
    pub enemy: Enemy,
    pub turn: u32,
    pub phase: CombatPhase,
}

impl CombatState {
    pub fn new(rng: &mut impl Rng) -> Self {
        let kind = EnemyKind::Louse;
        let max_hp = kind.max_hp();
        let intent = enemies::next_intent(&kind, 1);
        let mut state = Self {
            player: Player {
                hp: Hp(80),
                max_hp: Hp(80),
                block: Block(0),
                energy: Energy(3),
                max_energy: Energy(3),
                hand: Vec::new(),
                draw_pile: starter_deck(),
                discard_pile: Vec::new(),
            },
            enemy: Enemy {
                kind,
                hp: max_hp,
                max_hp,
                block: Block(0),
                intent,
            },
            turn: 1,
            phase: CombatPhase::PlayerTurn,
        };
        rng.shuffle(&mut state.player.draw_pile);
        draw_cards(&mut state.player, 5, rng);
        state
    }
}

fn starter_deck() -> Vec<Card> {
    let mut deck = Vec::new();
    for _ in 0..5 {
        deck.push(Card::Strike);
    }
    for _ in 0..4 {
        deck.push(Card::Defend);
    }
    deck
}

fn draw_cards(player: &mut Player, n: usize, rng: &mut impl Rng) {
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
pub enum Command {
    PlayCard(usize),
    EndTurn,
    EndEnemyTurn,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandError {
    CombatOver,
    InvalidCard,
    NotEnoughEnergy,
    InvalidPhase,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    CardPlayed { card: Card },
    PlayerAttacked { raw: i32, damage: i32 },
    PlayerBlocked { amount: i32 },
    EnemyAttacked { raw: i32, damage: i32 },
    EnemyDefended { amount: i32 },
    IntentRevealed { intent: Intent },
    PlayerBlockExpired { amount: i32 },
    TurnEnded,
    TurnStarted { turn: u32 },
    EnemyDied,
    PlayerDied,
}

pub fn apply_command(
    mut state: CombatState,
    command: Command,
    rng: &mut impl Rng,
) -> Result<(CombatState, Vec<Event>), CommandError> {
    if matches!(state.phase, CombatPhase::Victory | CombatPhase::Defeat) {
        return Err(CommandError::CombatOver);
    }

    let mut events = Vec::new();

    match command {
        Command::PlayCard(index) => {
            if state.phase != CombatPhase::PlayerTurn {
                return Err(CommandError::InvalidPhase);
            }
            if index >= state.player.hand.len() {
                return Err(CommandError::InvalidCard);
            }
            let card = state.player.hand[index].clone();
            if state.player.energy < card.energy_cost() {
                return Err(CommandError::NotEnoughEnergy);
            }
            state.player.hand.remove(index);
            state.player.energy = Energy(state.player.energy.0 - card.energy_cost().0);
            state.player.discard_pile.push(card.clone());
            events.push(Event::CardPlayed { card: card.clone() });
            apply_card(&card, &mut state, &mut events);
        }
        Command::EndTurn => {
            if state.phase != CombatPhase::PlayerTurn {
                return Err(CommandError::InvalidPhase);
            }
            events.push(Event::TurnEnded);
            state.player.discard_pile.append(&mut state.player.hand);
            state.phase = CombatPhase::EnemyTurn;
        }
        Command::EndEnemyTurn => {
            if state.phase != CombatPhase::EnemyTurn {
                return Err(CommandError::InvalidPhase);
            }
            state.enemy.block = Block(0);
            execute_intent(&mut state, &mut events);
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
                let next = enemies::next_intent(&state.enemy.kind, state.turn);
                state.enemy.intent = next;
                events.push(Event::IntentRevealed { intent: next });
                draw_cards(&mut state.player, 5, rng);
                state.phase = CombatPhase::PlayerTurn;
                events.push(Event::TurnStarted { turn: state.turn });
            }
        }
    }

    Ok((state, events))
}

fn execute_intent(state: &mut CombatState, events: &mut Vec<Event>) {
    match state.enemy.intent {
        Intent::Attack(n) => {
            let damage = deal_damage(n, &mut state.player.hp, &mut state.player.block);
            events.push(Event::EnemyAttacked { raw: n, damage });
        }
        Intent::Defend(n) => {
            state.enemy.block = Block(state.enemy.block.0 + n);
            events.push(Event::EnemyDefended { amount: n });
        }
    }
}

fn apply_card(card: &Card, state: &mut CombatState, events: &mut Vec<Event>) {
    crate::cards::apply(card, state, events);
}

pub(crate) fn deal_damage(amount: i32, hp: &mut Hp, block: &mut Block) -> i32 {
    let absorbed = amount.min(block.0).max(0);
    block.0 -= absorbed;
    let remainder = amount - absorbed;
    hp.0 = (hp.0 - remainder).max(0);
    remainder
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::NoOpRng;

    fn rng() -> NoOpRng {
        NoOpRng
    }

    // Creates a state with a known hand, empty piles, full energy — no shuffle needed.
    // Default enemy intent is Attack(8) so legacy tests that bundle EndTurn keep working.
    fn combat_with_hand(hand: Vec<Card>) -> CombatState {
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
            },
            enemy: Enemy {
                kind: EnemyKind::Louse,
                hp: Hp(20),
                max_hp: Hp(20),
                block: Block(0),
                intent: Intent::Attack(8),
            },
            turn: 1,
            phase: CombatPhase::PlayerTurn,
        }
    }

    // Runs EndTurn followed by EndEnemyTurn — for tests that don't care about
    // the intermediate EnemyTurn state.
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
    fn new_combat_leaves_4_cards_in_draw_pile() {
        let state = CombatState::new(&mut rng());
        assert_eq!(state.player.draw_pile.len(), 4);
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
        // draw pile had 5 cards → those fill the new hand; original 2 stay in discard
        assert_eq!(state.player.hand.len(), 5);
        assert!(state.player.discard_pile.contains(&Card::Strike));
        assert!(state.player.discard_pile.contains(&Card::Defend));
    }

    #[test]
    fn empty_draw_pile_shuffles_discard_when_drawing() {
        let mut state = combat_with_hand(Vec::new());
        state.player.discard_pile = vec![Card::Strike, Card::Defend, Card::Strike];
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        // Only 3 cards available — all should be drawn
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
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
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
        let result = apply_command(state, Command::PlayCard(0), &mut rng());
        assert_eq!(result, Err(CommandError::NotEnoughEnergy));
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
        state.enemy.hp = Hp(1);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.enemy.hp, Hp(0));
    }

    #[test]
    fn player_hp_does_not_go_below_zero() {
        // EndEnemyTurn fires Attack(8); with 1 HP result should be Hp(0) not Hp(-7)
        let mut state = combat_with_hand(Vec::new());
        state.player.hp = Hp(1);
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(0));
    }

    // --- Command rejection ---

    #[test]
    fn invalid_card_index_returns_error() {
        let state = combat_with_hand(vec![Card::Strike]);
        let result = apply_command(state, Command::PlayCard(5), &mut rng());
        assert_eq!(result, Err(CommandError::InvalidCard));
    }

    #[test]
    fn commands_rejected_after_victory() {
        let mut state = combat_with_hand(vec![Card::Strike]);
        state.enemy.hp = Hp(1);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.phase, CombatPhase::Victory);

        let result = apply_command(state, Command::PlayCard(0), &mut rng());
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
        assert_eq!(state.enemy.intent, Intent::Attack(8));
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
        assert_eq!(state.enemy.intent, Intent::Defend(5));
    }

    #[test]
    fn intent_alternates_back_to_attack_on_turn_3() {
        let state = combat_with_hand(Vec::new());
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.turn, 3);
        assert_eq!(state.enemy.intent, Intent::Attack(8));
    }

    #[test]
    fn defend_intent_grants_enemy_block() {
        let mut state = combat_with_hand(Vec::new());
        state.enemy.intent = Intent::Defend(5);
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.enemy.block, Block(5));
    }

    #[test]
    fn defend_intent_emits_enemy_defended_event() {
        let mut state = combat_with_hand(Vec::new());
        state.enemy.intent = Intent::Defend(5);
        let (_, events) = end_turn_full(state, &mut rng()).unwrap();
        assert!(events.contains(&Event::EnemyDefended { amount: 5 }));
    }

    #[test]
    fn defend_intent_does_not_damage_player() {
        let mut state = combat_with_hand(Vec::new());
        state.enemy.intent = Intent::Defend(5);
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(80));
    }

    #[test]
    fn enemy_block_absorbs_player_strike_damage() {
        let mut state = combat_with_hand(vec![Card::Strike]);
        state.enemy.block = Block(4);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        // Strike does 6 damage; 4 absorbed by block, 2 remainder to HP
        assert_eq!(state.enemy.block, Block(0));
        assert_eq!(state.enemy.hp, Hp(18));
    }

    #[test]
    fn enemy_block_fully_absorbs_player_strike() {
        let mut state = combat_with_hand(vec![Card::Strike]);
        state.enemy.block = Block(10);
        let (state, _) = apply_command(state, Command::PlayCard(0), &mut rng()).unwrap();
        assert_eq!(state.enemy.hp, Hp(20));
    }

    #[test]
    fn enemy_block_resets_when_enemy_acts() {
        // Pre-existing block should reset to 0 before the intent fires,
        // so a Defend(5) intent yields exactly 5 block — not 10 stacked.
        let mut state = combat_with_hand(Vec::new());
        state.enemy.block = Block(5);
        state.enemy.intent = Intent::Defend(5);
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        assert_eq!(state.enemy.block, Block(5));
    }

    #[test]
    fn enemy_block_persists_through_player_turn() {
        // After a Defend turn, the enemy carries block into the player's next turn.
        let mut state = combat_with_hand(vec![Card::Strike]);
        state.enemy.intent = Intent::Defend(5);
        let (state, _) = end_turn_full(state, &mut rng()).unwrap();
        // Block should still be 5 at the start of player's next turn
        assert_eq!(state.enemy.block, Block(5));
    }

    // --- Phase guards ---

    #[test]
    fn cannot_play_card_during_enemy_turn() {
        let state = combat_with_hand(vec![Card::Strike]);
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        let result = apply_command(state, Command::PlayCard(0), &mut rng());
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
        // After full cycle: turn 2's intent is Defend(5)
        assert!(events.contains(&Event::IntentRevealed { intent: Intent::Defend(5) }));
    }
}

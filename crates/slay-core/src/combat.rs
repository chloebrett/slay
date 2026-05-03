use crate::cards::Card;
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
    pub name: String,
    pub hp: Hp,
    pub max_hp: Hp,
    pub block: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CombatPhase {
    PlayerTurn,
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
                name: "Louse".to_string(),
                hp: Hp(20),
                max_hp: Hp(20),
                block: Block(0),
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandError {
    CombatOver,
    InvalidCard,
    NotEnoughEnergy,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    CardPlayed { index: usize },
    PlayerAttacked { damage: i32 },
    PlayerBlocked { amount: i32 },
    EnemyAttacked { damage: i32 },
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
    if state.phase != CombatPhase::PlayerTurn {
        return Err(CommandError::CombatOver);
    }

    let mut events = Vec::new();

    match command {
        Command::PlayCard(index) => {
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
            events.push(Event::CardPlayed { index });
            apply_card(&card, &mut state, &mut events);
        }
        Command::EndTurn => {
            events.push(Event::TurnEnded);
            state.player.discard_pile.extend(state.player.hand.drain(..));
            let damage = deal_damage(8, &mut state.player.hp, &mut state.player.block);
            events.push(Event::EnemyAttacked { damage });
            if state.player.hp.0 <= 0 {
                state.phase = CombatPhase::Defeat;
                events.push(Event::PlayerDied);
            } else {
                state.player.block = Block(0);
                state.player.energy = state.player.max_energy;
                state.turn += 1;
                draw_cards(&mut state.player, 5, rng);
                events.push(Event::TurnStarted { turn: state.turn });
            }
        }
    }

    Ok((state, events))
}

fn apply_card(card: &Card, state: &mut CombatState, events: &mut Vec<Event>) {
    match card {
        Card::Strike => {
            let damage = deal_damage(6, &mut state.enemy.hp, &mut state.enemy.block);
            events.push(Event::PlayerAttacked { damage });
            if state.enemy.hp.0 <= 0 {
                state.phase = CombatPhase::Victory;
                events.push(Event::EnemyDied);
            }
        }
        Card::Defend => {
            let amount = 5;
            state.player.block = Block(state.player.block.0 + amount);
            events.push(Event::PlayerBlocked { amount });
        }
    }
}

fn deal_damage(amount: i32, hp: &mut Hp, block: &mut Block) -> i32 {
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

    // Creates a state with a known hand, empty piles, full energy — no shuffle needed
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
                name: "Louse".to_string(),
                hp: Hp(20),
                max_hp: Hp(20),
                block: Block(0),
            },
            turn: 1,
            phase: CombatPhase::PlayerTurn,
        }
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
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 5);
    }

    #[test]
    fn end_turn_discards_remaining_hand() {
        let mut state = combat_with_hand(vec![Card::Strike, Card::Defend]);
        state.player.draw_pile = vec![Card::Strike; 5];
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        // draw pile had 5 cards → those fill the new hand; original 2 stay in discard
        assert_eq!(state.player.hand.len(), 5);
        assert!(state.player.discard_pile.contains(&Card::Strike));
        assert!(state.player.discard_pile.contains(&Card::Defend));
    }

    #[test]
    fn empty_draw_pile_shuffles_discard_when_drawing() {
        let mut state = combat_with_hand(Vec::new());
        state.player.discard_pile = vec![Card::Strike, Card::Defend, Card::Strike];
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
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
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
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
        assert!(events.contains(&Event::PlayerAttacked { damage: 6 }));
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

    // --- Enemy attack (EndTurn) ---

    #[test]
    fn end_turn_causes_enemy_to_attack_for_8() {
        let state = combat_with_hand(Vec::new());
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(72));
    }

    #[test]
    fn end_turn_emits_enemy_attacked_event() {
        let state = combat_with_hand(Vec::new());
        let (_, events) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert!(events.contains(&Event::EnemyAttacked { damage: 8 }));
    }

    #[test]
    fn block_absorbs_enemy_damage_before_hp() {
        let mut state = combat_with_hand(Vec::new());
        state.player.block = Block(5);
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(0));
        assert_eq!(state.player.hp, Hp(77));
    }

    #[test]
    fn block_fully_absorbing_attack_leaves_hp_unchanged() {
        let mut state = combat_with_hand(Vec::new());
        state.player.block = Block(10);
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(80));
    }

    #[test]
    fn player_block_resets_at_start_of_next_turn() {
        let mut state = combat_with_hand(Vec::new());
        state.player.block = Block(5);
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(0));
    }

    #[test]
    fn enemy_killing_player_yields_defeat() {
        let mut state = combat_with_hand(Vec::new());
        state.player.hp = Hp(1);
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(state.phase, CombatPhase::Defeat);
    }

    #[test]
    fn enemy_killing_player_emits_player_died_event() {
        let mut state = combat_with_hand(Vec::new());
        state.player.hp = Hp(1);
        let (_, events) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
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
        // EndTurn triggers enemy attack (8 damage); with 1 HP result should be Hp(0) not Hp(-7)
        let mut state = combat_with_hand(Vec::new());
        state.player.hp = Hp(1);
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
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
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(state.phase, CombatPhase::Defeat);

        let result = apply_command(state, Command::EndTurn, &mut rng());
        assert_eq!(result, Err(CommandError::CombatOver));
    }
}

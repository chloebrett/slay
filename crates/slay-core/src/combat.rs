use crate::types::{Block, Hp};

#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    pub hp: Hp,
    pub max_hp: Hp,
    pub block: Block,
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
    pub fn new() -> Self {
        Self {
            player: Player { hp: Hp(80), max_hp: Hp(80), block: Block(0) },
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Attack,
    Block,
    EndTurn,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandError {
    CombatOver,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
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
) -> Result<(CombatState, Vec<Event>), CommandError> {
    if state.phase != CombatPhase::PlayerTurn {
        return Err(CommandError::CombatOver);
    }

    let mut events = Vec::new();

    match command {
        Command::Attack => {
            let damage = deal_damage(6, &mut state.enemy.hp, &mut state.enemy.block);
            events.push(Event::PlayerAttacked { damage });
            if state.enemy.hp.0 <= 0 {
                state.phase = CombatPhase::Victory;
                events.push(Event::EnemyDied);
            }
        }
        Command::Block => {
            let amount = 5;
            state.player.block = Block(state.player.block.0 + amount);
            events.push(Event::PlayerBlocked { amount });
        }
        Command::EndTurn => {
            events.push(Event::TurnEnded);
            let damage = deal_damage(8, &mut state.player.hp, &mut state.player.block);
            events.push(Event::EnemyAttacked { damage });
            if state.player.hp.0 <= 0 {
                state.phase = CombatPhase::Defeat;
                events.push(Event::PlayerDied);
            } else {
                state.player.block = Block(0);
                state.turn += 1;
                events.push(Event::TurnStarted { turn: state.turn });
            }
        }
    }

    Ok((state, events))
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

    fn combat() -> CombatState {
        CombatState::new()
    }

    // --- Attack ---

    #[test]
    fn attack_deals_6_damage_to_enemy() {
        let (state, _) = apply_command(combat(), Command::Attack).unwrap();
        assert_eq!(state.enemy.hp, Hp(14));
    }

    #[test]
    fn attack_emits_player_attacked_event() {
        let (_, events) = apply_command(combat(), Command::Attack).unwrap();
        assert!(events.contains(&Event::PlayerAttacked { damage: 6 }));
    }

    #[test]
    fn attack_killing_enemy_yields_victory() {
        let mut state = combat();
        state.enemy.hp = Hp(1);
        let (state, _) = apply_command(state, Command::Attack).unwrap();
        assert_eq!(state.phase, CombatPhase::Victory);
    }

    #[test]
    fn attack_killing_enemy_emits_enemy_died_event() {
        let mut state = combat();
        state.enemy.hp = Hp(1);
        let (_, events) = apply_command(state, Command::Attack).unwrap();
        assert!(events.contains(&Event::EnemyDied));
    }

    // --- Block ---

    #[test]
    fn block_grants_5_block_to_player() {
        let (state, _) = apply_command(combat(), Command::Block).unwrap();
        assert_eq!(state.player.block, Block(5));
    }

    #[test]
    fn block_emits_player_blocked_event() {
        let (_, events) = apply_command(combat(), Command::Block).unwrap();
        assert!(events.contains(&Event::PlayerBlocked { amount: 5 }));
    }

    // --- End Turn / Enemy Attack ---

    #[test]
    fn end_turn_causes_enemy_to_attack_for_8() {
        let (state, _) = apply_command(combat(), Command::EndTurn).unwrap();
        assert_eq!(state.player.hp, Hp(72));
    }

    #[test]
    fn end_turn_emits_enemy_attacked_event() {
        let (_, events) = apply_command(combat(), Command::EndTurn).unwrap();
        assert!(events.contains(&Event::EnemyAttacked { damage: 8 }));
    }

    #[test]
    fn block_absorbs_enemy_damage_before_hp() {
        let mut state = combat();
        state.player.block = Block(5);
        let (state, _) = apply_command(state, Command::EndTurn).unwrap();
        assert_eq!(state.player.block, Block(0));
        assert_eq!(state.player.hp, Hp(77)); // 80 - (8 - 5)
    }

    #[test]
    fn block_fully_absorbing_attack_leaves_hp_unchanged() {
        let mut state = combat();
        state.player.block = Block(10);
        let (state, _) = apply_command(state, Command::EndTurn).unwrap();
        assert_eq!(state.player.hp, Hp(80));
    }

    #[test]
    fn player_block_resets_at_start_of_next_turn() {
        let mut state = combat();
        state.player.block = Block(5);
        let (state, _) = apply_command(state, Command::EndTurn).unwrap();
        assert_eq!(state.player.block, Block(0));
    }

    #[test]
    fn enemy_killing_player_yields_defeat() {
        let mut state = combat();
        state.player.hp = Hp(1);
        let (state, _) = apply_command(state, Command::EndTurn).unwrap();
        assert_eq!(state.phase, CombatPhase::Defeat);
    }

    #[test]
    fn enemy_killing_player_emits_player_died_event() {
        let mut state = combat();
        state.player.hp = Hp(1);
        let (_, events) = apply_command(state, Command::EndTurn).unwrap();
        assert!(events.contains(&Event::PlayerDied));
    }

    // --- HP clamping ---

    #[test]
    fn enemy_hp_does_not_go_below_zero() {
        let mut state = combat();
        state.enemy.hp = Hp(1);
        let (state, _) = apply_command(state, Command::Attack).unwrap();
        assert_eq!(state.enemy.hp, Hp(0));
    }

    #[test]
    fn player_hp_does_not_go_below_zero() {
        // EndTurn triggers enemy attack (8 damage); with 1 HP result should be Hp(0) not Hp(-7)
        let mut state = combat();
        state.player.hp = Hp(1);
        let (state, _) = apply_command(state, Command::EndTurn).unwrap();
        assert_eq!(state.player.hp, Hp(0));
    }

    // --- Command rejection ---

    #[test]
    fn commands_rejected_after_victory() {
        let mut state = combat();
        state.enemy.hp = Hp(1);
        let (state, _) = apply_command(state, Command::Attack).unwrap();
        assert_eq!(state.phase, CombatPhase::Victory);

        let result = apply_command(state, Command::Attack);
        assert_eq!(result, Err(CommandError::CombatOver));
    }

    #[test]
    fn commands_rejected_after_defeat() {
        let mut state = combat();
        state.player.hp = Hp(1);
        let (state, _) = apply_command(state, Command::EndTurn).unwrap();
        assert_eq!(state.phase, CombatPhase::Defeat);

        let result = apply_command(state, Command::Attack);
        assert_eq!(result, Err(CommandError::CombatOver));
    }
}

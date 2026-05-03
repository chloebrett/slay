use slay_core::{
    apply_command, starter_deck, Block, Card, CombatPhase, CombatState, Command, Enemy, EnemyKind,
    Energy, GameState, Hp, Intent, NoOpRng, Player, StatusMap,
};

struct TestHarness {
    state: GameState,
    rng: NoOpRng,
}

impl TestHarness {
    fn with_hand(hand: Vec<Card>) -> Self {
        let state = GameState::Combat {
            state: CombatState {
                player: Player {
                    hp: Hp(80),
                    max_hp: Hp(80),
                    block: Block(0),
                    energy: Energy(3),
                    max_energy: Energy(3),
                    hand,
                    draw_pile: Vec::new(),
                    discard_pile: Vec::new(),
                    statuses: StatusMap::new(),
                    deck: starter_deck(),
                    gold: 0,
                },
                enemy: Enemy {
                    kind: EnemyKind::Louse,
                    hp: Hp(20),
                    max_hp: Hp(20),
                    block: Block(0),
                    intent: Intent::Attack(8),
                    statuses: StatusMap::new(),
                },
                turn: 1,
                phase: CombatPhase::PlayerTurn,
            },
            floor: 0,
        };
        Self { state, rng: NoOpRng }
    }

    // Issues a player command, then auto-drains any EnemyTurn phase — same behavior
    // as the TUI loop.
    fn send(&mut self, input: &str) -> Result<(), String> {
        let command = slay_tui::command::parse(input, &self.state)
            .ok_or_else(|| format!("unknown command: '{input}'"))?;
        let (new_state, _) = apply_command(self.state.clone(), command, &mut self.rng)
            .map_err(|e| format!("{e:?}"))?;
        self.state = new_state;
        loop {
            let is_enemy_turn = matches!(
                &self.state,
                GameState::Combat { state: cs, .. } if cs.phase == CombatPhase::EnemyTurn
            );
            if !is_enemy_turn {
                break;
            }
            let (ns, _) = apply_command(
                self.state.clone(),
                Command::EndEnemyTurn,
                &mut self.rng,
            )
            .map_err(|e| format!("{e:?}"))?;
            self.state = ns;
        }
        Ok(())
    }

    fn player_hp(&self) -> i32 {
        match &self.state {
            GameState::Combat { state, .. } => state.player.hp.0,
            _ => panic!("not in combat: {:?}", std::mem::discriminant(&self.state)),
        }
    }

    fn enemy_hp(&self) -> i32 {
        match &self.state {
            GameState::Combat { state, .. } => state.enemy.hp.0,
            _ => panic!("not in combat"),
        }
    }

    fn combat_phase(&self) -> Option<&CombatPhase> {
        match &self.state {
            GameState::Combat { state, .. } => Some(&state.phase),
            _ => None,
        }
    }

    fn is_victory(&self) -> bool {
        // Victory means we transitioned out of combat — either CardReward or GameOver
        matches!(
            &self.state,
            GameState::CardReward(_) | GameState::GameOver { victory: true }
        )
    }
}

#[test]
fn play_strike_reduces_enemy_hp() {
    let mut game = TestHarness::with_hand(vec![Card::Strike]);
    game.send("play 1").unwrap();
    assert_eq!(game.enemy_hp(), 14);
}

#[test]
fn play_defend_then_end_reduces_damage_taken() {
    let mut game = TestHarness::with_hand(vec![Card::Defend]);
    if let GameState::Combat { state, .. } = &mut game.state {
        state.player.draw_pile = vec![Card::Defend; 5];
    }
    game.send("play 1").unwrap();
    game.send("end").unwrap(); // turn 1 end → enemy attacks 8, block 5 absorbs → 3 dmg
    assert_eq!(game.player_hp(), 77); // 80 - (8 - 5)
}

#[test]
fn unknown_command_returns_error_without_crashing() {
    let mut game = TestHarness::with_hand(vec![Card::Strike]);
    let result = game.send("fireball");
    assert!(result.is_err());
    assert_eq!(game.player_hp(), 80);
}

#[test]
fn player_wins_by_playing_strikes_until_enemy_dead() {
    // Enemy 20 HP, Strike 6 dmg, player 3 energy/turn → 3 strikes per turn.
    // Turn 1: 3 strikes → enemy 2 HP. Player takes 8 from Attack intent → 72 HP.
    // Turn 2: 1 strike kills.
    let mut game = TestHarness::with_hand(vec![Card::Strike; 5]);
    if let GameState::Combat { state, .. } = &mut game.state {
        state.player.draw_pile = vec![Card::Strike; 10];
    }

    game.send("play 1").unwrap(); // enemy hp 14
    game.send("play 1").unwrap(); // enemy hp 8
    game.send("play 1").unwrap(); // enemy hp 2
    game.send("end").unwrap();    // enemy attacks 8 → player 72
    assert_eq!(game.player_hp(), 72);

    game.send("play 1").unwrap(); // enemy hp 0 → leaves combat
    assert!(game.is_victory());
}

#[test]
fn play_zero_is_invalid() {
    let mut game = TestHarness::with_hand(vec![Card::Strike]);
    let result = game.send("play 0");
    assert!(result.is_err());
}

#[test]
fn enemy_alternates_attack_and_defend_intents() {
    let mut game = TestHarness::with_hand(Vec::new());
    if let GameState::Combat { state, .. } = &mut game.state {
        state.player.draw_pile = vec![Card::Strike; 10];
    }
    let intent_attack = matches!(
        &game.state,
        GameState::Combat { state: cs, .. } if cs.enemy.intent == Intent::Attack(8)
    );
    assert!(intent_attack);

    game.send("end").unwrap();
    let intent_defend = matches!(
        &game.state,
        GameState::Combat { state: cs, .. } if cs.enemy.intent == Intent::Defend(5)
    );
    assert!(intent_defend);

    game.send("end").unwrap();
    let intent_attack2 = matches!(
        &game.state,
        GameState::Combat { state: cs, .. } if cs.enemy.intent == Intent::Attack(8)
    );
    assert!(intent_attack2);
}

#[test]
fn enemy_block_from_defend_absorbs_player_attack() {
    let mut game = TestHarness::with_hand(Vec::new());
    if let GameState::Combat { state, .. } = &mut game.state {
        state.player.draw_pile = vec![Card::Strike; 10];
    }
    game.send("end").unwrap(); // turn 2: intent Defend
    game.send("end").unwrap(); // enemy defends; now turn 3, enemy has 5 block

    let enemy_block = match &game.state {
        GameState::Combat { state, .. } => state.enemy.block,
        _ => panic!("not in combat"),
    };
    assert_eq!(enemy_block, slay_core::Block(5));

    game.send("play 1").unwrap(); // Strike: 5 absorbed, 1 to HP
    assert_eq!(game.enemy_hp(), 19);

    let enemy_block2 = match &game.state {
        GameState::Combat { state, .. } => state.enemy.block,
        _ => panic!("not in combat"),
    };
    assert_eq!(enemy_block2, slay_core::Block(0));
}

use slay_core::{
    Block, Card, Command, CombatPhase, CombatState, Enemy, EnemyKind, Energy, Hp, Intent, NoOpRng,
    Player,
};

struct TestHarness {
    state: CombatState,
    rng: NoOpRng,
}

impl TestHarness {
    fn with_hand(hand: Vec<Card>) -> Self {
        let state = CombatState {
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
        };
        Self { state, rng: NoOpRng }
    }

    // Issues a player command, then auto-drains any EnemyTurn phase — same behavior
    // as the TUI loop. Tests that need to inspect intermediate EnemyTurn state can
    // call apply_command directly instead.
    fn send(&mut self, input: &str) -> Result<(), String> {
        let command = slay_tui::command::parse(input)
            .ok_or_else(|| format!("unknown command: '{input}'"))?;
        let (new_state, _) = slay_core::apply_command(self.state.clone(), command, &mut self.rng)
            .map_err(|e| format!("{e:?}"))?;
        self.state = new_state;
        while self.state.phase == CombatPhase::EnemyTurn {
            let (new_state, _) = slay_core::apply_command(
                self.state.clone(),
                Command::EndEnemyTurn,
                &mut self.rng,
            )
            .map_err(|e| format!("{e:?}"))?;
            self.state = new_state;
        }
        Ok(())
    }

    fn player_hp(&self) -> i32 { self.state.player.hp.0 }
    fn enemy_hp(&self) -> i32 { self.state.enemy.hp.0 }
    fn phase(&self) -> &CombatPhase { &self.state.phase }
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
    game.state.player.draw_pile = vec![Card::Defend; 5];
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
    // Turn 2: 1 strike kills (enemy block 0, intent was Defend but enemy dies first).
    let mut game = TestHarness::with_hand(vec![Card::Strike; 5]);
    game.state.player.draw_pile = vec![Card::Strike; 10];

    game.send("play 1").unwrap(); // enemy hp 14
    game.send("play 1").unwrap(); // enemy hp 8
    game.send("play 1").unwrap(); // enemy hp 2
    game.send("end").unwrap();    // enemy attacks 8 → player 72
    assert_eq!(game.player_hp(), 72);

    game.send("play 1").unwrap(); // enemy hp 0 → Victory
    assert_eq!(game.phase(), &CombatPhase::Victory);
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
    game.state.player.draw_pile = vec![Card::Strike; 10];
    assert_eq!(game.state.enemy.intent, Intent::Attack(8));
    game.send("end").unwrap();
    assert_eq!(game.state.enemy.intent, Intent::Defend(5));
    game.send("end").unwrap();
    assert_eq!(game.state.enemy.intent, Intent::Attack(8));
}

#[test]
fn enemy_block_from_defend_absorbs_player_attack() {
    // Burn turn 1 (Attack intent), turn 2 enemy defends, turn 3 player attacks
    // through enemy's block.
    let mut game = TestHarness::with_hand(Vec::new());
    game.state.player.draw_pile = vec![Card::Strike; 10];
    game.send("end").unwrap(); // turn 2: intent Defend
    game.send("end").unwrap(); // enemy defends; now turn 3, enemy has 5 block
    assert_eq!(game.state.enemy.block, Block(5));
    game.send("play 1").unwrap(); // Strike: 5 absorbed, 1 to HP
    assert_eq!(game.enemy_hp(), 19);
    assert_eq!(game.state.enemy.block, Block(0));
}

use slay_core::{Card, CombatPhase, CombatState, NoOpRng, Player, Block, Energy, Hp, Enemy};

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
                name: "Louse".to_string(),
                hp: Hp(20),
                max_hp: Hp(20),
                block: Block(0),
            },
            turn: 1,
            phase: CombatPhase::PlayerTurn,
        };
        Self { state, rng: NoOpRng }
    }

    fn send(&mut self, input: &str) -> Result<(), String> {
        let command = slay_tui::command::parse(input)
            .ok_or_else(|| format!("unknown command: '{input}'"))?;
        let (new_state, _) = slay_core::apply_command(self.state.clone(), command, &mut self.rng)
            .map_err(|e| format!("{e:?}"))?;
        self.state = new_state;
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
    game.send("play 1").unwrap();
    game.send("end").unwrap();
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
    // enemy 20 HP, Strike does 6 — need 4 hits (24 > 20)
    // each turn: play strike, end turn (take 8 damage)
    let five_strikes = vec![Card::Strike; 5];
    let mut game = TestHarness::with_hand(five_strikes);
    // give enough cards in draw pile for subsequent turns
    game.state.player.draw_pile = vec![Card::Strike; 10];

    game.send("play 1").unwrap(); // enemy: 14
    game.send("end").unwrap();    // player: 72
    game.send("play 1").unwrap(); // enemy: 8
    game.send("end").unwrap();    // player: 64
    game.send("play 1").unwrap(); // enemy: 2
    game.send("end").unwrap();    // player: 56
    game.send("play 1").unwrap(); // enemy: dead
    assert_eq!(game.phase(), &CombatPhase::Victory);
}

#[test]
fn play_zero_is_invalid() {
    let mut game = TestHarness::with_hand(vec![Card::Strike]);
    let result = game.send("play 0");
    assert!(result.is_err());
}

use slay_core::{CombatPhase, CombatState};

struct TestHarness {
    state: CombatState,
}

impl TestHarness {
    fn new() -> Self {
        Self { state: CombatState::new() }
    }

    fn send(&mut self, input: &str) -> Result<(), String> {
        let command = slay_tui::command::parse(input)
            .ok_or_else(|| format!("unknown command: '{input}'"))?;
        let (new_state, _) = slay_core::apply_command(self.state.clone(), command)
            .map_err(|e| format!("{e:?}"))?;
        self.state = new_state;
        Ok(())
    }

    fn player_hp(&self) -> i32 { self.state.player.hp.0 }
    fn enemy_hp(&self) -> i32 { self.state.enemy.hp.0 }
    fn phase(&self) -> &CombatPhase { &self.state.phase }
}

#[test]
fn attack_command_reduces_enemy_hp() {
    let mut game = TestHarness::new();
    game.send("attack").unwrap();
    assert_eq!(game.enemy_hp(), 14);
}

#[test]
fn block_command_then_end_turn_reduces_damage_taken() {
    let mut game = TestHarness::new();
    game.send("block").unwrap();
    game.send("end").unwrap();
    assert_eq!(game.player_hp(), 77); // 80 - (8 - 5)
}

#[test]
fn unknown_command_returns_error_without_crashing() {
    let mut game = TestHarness::new();
    let result = game.send("fireball");
    assert!(result.is_err());
    assert_eq!(game.player_hp(), 80); // state unchanged
}

#[test]
fn player_wins_by_attacking_until_enemy_dead() {
    let mut game = TestHarness::new();
    // enemy has 20 HP, attack does 6 — need 4 attacks (6+6+6+6=24 > 20)
    // but we also have to survive enemy turns in between
    // attack, end, attack, end, attack, end, attack → enemy dead on 4th attack
    game.send("attack").unwrap(); // enemy: 14
    game.send("end").unwrap();    // player: 72
    game.send("attack").unwrap(); // enemy: 8
    game.send("end").unwrap();    // player: 64
    game.send("attack").unwrap(); // enemy: 2
    game.send("end").unwrap();    // player: 56
    game.send("attack").unwrap(); // enemy: dead → victory
    assert_eq!(game.phase(), &CombatPhase::Victory);
}

#[test]
fn aliases_a_and_b_work() {
    let mut game = TestHarness::new();
    game.send("a").unwrap();
    assert_eq!(game.enemy_hp(), 14);
    game.send("b").unwrap();
    assert_eq!(game.state.player.block.0, 5);
}

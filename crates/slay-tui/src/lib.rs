pub mod command;
pub mod engine;
pub mod game;
pub mod tui;

#[cfg(test)]
mod tests {
    use super::game::run_game;
    use slay_core::{AnyRng, NoOpRng, new_simple_run};

    #[test]
    fn run_game_writes_welcome_message() {
        let input = b"";
        let mut output = Vec::<u8>::new();
        let state = new_simple_run();
        run_game(state, input.as_ref(), &mut output, &mut AnyRng::NoOp(NoOpRng), false);
        let text = String::from_utf8(output).unwrap();
        assert!(text.contains("Slay the Spire"), "expected welcome in output, got:\n{text}");
    }

    #[test]
    fn combat_hud_shows_pile_counts() {
        use slay_core::{Command, EnemyKind, AnyRng, NoOpRng, new_simple_run};
        use slay_core::apply_command;

        let mut rng = AnyRng::NoOp(NoOpRng);
        let mut state = new_simple_run();
        // Spawn a louse and enter combat
        state = apply_command(state, Command::Spawn(vec![EnemyKind::Louse]), &mut rng).unwrap().0;
        state = apply_command(state, Command::ChooseNode(0), &mut rng).unwrap().0;

        let mut output = Vec::<u8>::new();
        run_game(state, b"".as_ref(), &mut output, &mut rng, false);
        let text = String::from_utf8(output).unwrap();
        assert!(text.contains("Draw:"), "expected 'Draw:' in combat HUD, got:\n{text}");
        assert!(text.contains("Discard:"), "expected 'Discard:' in combat HUD, got:\n{text}");
    }
}

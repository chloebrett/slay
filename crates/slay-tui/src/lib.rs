pub mod command;
pub mod game;

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
}

use slay_core::Command;

pub fn parse(input: &str) -> Option<Command> {
    match input.trim().to_lowercase().as_str() {
        "attack" | "a" => Some(Command::Attack),
        "block" | "b" => Some(Command::Block),
        "end" | "end turn" | "pass" => Some(Command::EndTurn),
        _ => None,
    }
}

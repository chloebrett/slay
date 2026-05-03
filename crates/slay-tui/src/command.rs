use slay_core::Command;

pub fn parse(input: &str) -> Option<Command> {
    let s = input.trim().to_lowercase();
    if let Some(rest) = s.strip_prefix("play ") {
        let n: usize = rest.trim().parse().ok()?;
        if n == 0 {
            return None;
        }
        return Some(Command::PlayCard(n - 1));
    }
    if let Ok(n) = s.trim().parse::<usize>() {
        if n > 0 {
            return Some(Command::PlayCard(n - 1));
        }
    }
    match s.as_str() {
        "end" | "end turn" | "pass" => Some(Command::EndTurn),
        _ => None,
    }
}

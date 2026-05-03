use slay_core::{Command, GameState};

pub fn parse(input: &str, state: &GameState) -> Option<Command> {
    let s = input.trim().to_lowercase();
    match state {
        GameState::Map(_) => parse_map(&s),
        GameState::Combat { .. } => parse_combat(&s),
        GameState::RestSite(_) => parse_rest(&s),
        GameState::CardReward(_) => parse_card_reward(&s),
        GameState::GameOver { .. } => None,
    }
}

fn parse_map(s: &str) -> Option<Command> {
    if let Ok(n) = s.trim().parse::<usize>() {
        if n > 0 {
            return Some(Command::ChooseNode(n - 1));
        }
    }
    None
}

fn parse_combat(s: &str) -> Option<Command> {
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
    match s {
        "end" | "end turn" | "pass" | "e" => Some(Command::EndTurn),
        _ => None,
    }
}

fn parse_rest(s: &str) -> Option<Command> {
    match s {
        "rest" | "r" => Some(Command::Rest),
        _ => None,
    }
}

fn parse_card_reward(s: &str) -> Option<Command> {
    if let Some(rest) = s.strip_prefix("pick ") {
        let n: usize = rest.trim().parse().ok()?;
        if n == 0 {
            return None;
        }
        return Some(Command::ChooseCardReward(n - 1));
    }
    if let Ok(n) = s.trim().parse::<usize>() {
        if n > 0 {
            return Some(Command::ChooseCardReward(n - 1));
        }
    }
    None
}

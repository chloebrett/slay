use slay_core::{Command, GameState};

pub fn parse(input: &str, state: &GameState, debug: bool) -> Option<Command> {
    let s = input.trim().to_lowercase();
    match state {
        GameState::Map(_) => parse_map(&s, debug),
        GameState::Combat { .. } => parse_combat(&s, debug),
        GameState::RestSite(_) => parse_rest(&s),
        GameState::CardReward(_) => parse_card_reward(&s),
        GameState::GameOver { .. } => None,
    }
}

fn parse_map(s: &str, debug: bool) -> Option<Command> {
    if debug && s == "skip" {
        return Some(Command::SkipFloor);
    }
    if let Ok(n) = s.trim().parse::<usize>() {
        if n > 0 {
            return Some(Command::ChooseNode(n - 1));
        }
    }
    None
}

fn parse_combat(s: &str, debug: bool) -> Option<Command> {
    if debug && s == "win" {
        return Some(Command::WinCombat);
    }
    let num_str = s.strip_prefix("play ").unwrap_or(s);
    if let Ok(n) = num_str.trim().parse::<usize>() {
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
    let after = s.strip_prefix("upgrade ").or_else(|| s.strip_prefix("u "));
    if let Some(rest) = after {
        let n: usize = rest.trim().parse().ok()?;
        return (n > 0).then_some(Command::UpgradeCard(n - 1));
    }
    match s {
        "rest" | "r" => Some(Command::Rest),
        _ => None,
    }
}

fn parse_card_reward(s: &str) -> Option<Command> {
    let num_str = s.strip_prefix("pick ").unwrap_or(s);
    if let Ok(n) = num_str.trim().parse::<usize>() {
        if n > 0 {
            return Some(Command::ChooseCardReward(n - 1));
        }
    }
    match s {
        "skip" | "s" => Some(Command::SkipReward),
        _ => None,
    }
}

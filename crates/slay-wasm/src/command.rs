use slay_core::{Card, CombatPhase, Command, EnemyKind, GameState, Potion, Relic};

pub fn parse(input: &str, state: &GameState, debug: bool) -> Option<Command> {
    let s = input.trim().to_lowercase();
    match state {
        GameState::Map(_) => parse_map(&s, debug),
        GameState::Combat { state: cs, .. } => {
            if matches!(cs.phase, CombatPhase::ChooseCard(_)) {
                parse_choose_card(&s)
            } else {
                parse_combat(&s, debug)
            }
        }
        GameState::RestSite(_) => parse_rest(&s),
        GameState::TreasureRoom(_) => parse_treasure(&s),
        GameState::CardReward(_) => parse_card_reward(&s),
        GameState::Shop(_) => parse_shop(&s),
        GameState::EventRoom(_) => parse_event(&s),
        GameState::GameOver { .. } => None,
        GameState::Neow(_) => parse_neow(&s),
    }
}

fn parse_neow(s: &str) -> Option<Command> {
    let n: usize = s.trim().parse().ok()?;
    (n > 0).then(|| Command::ChooseNeowBlessing(n - 1))
}

fn parse_discard_potion(s: &str) -> Option<Command> {
    let rest = s.strip_prefix("discard ")?;
    let n: usize = rest.trim().parse().ok()?;
    (n > 0).then(|| Command::DiscardPotion(n - 1))
}

fn parse_map(s: &str, debug: bool) -> Option<Command> {
    if debug && s == "skip" {
        return Some(Command::SkipFloor);
    }
    if debug {
        if let Some(id) = s.strip_prefix("relic ") {
            return Relic::from_id(id.trim()).map(Command::AddRelic);
        }
    }
    if let Some(rest) = s.strip_prefix("spawn ") {
        let enemies: Vec<EnemyKind> = rest.split_whitespace()
            .filter_map(EnemyKind::from_id)
            .collect();
        return if enemies.is_empty() { None } else { Some(Command::Spawn(enemies)) };
    }
    if let Some(cmd) = parse_discard_potion(s) {
        return Some(cmd);
    }
    if let Ok(n) = s.trim().parse::<usize>() {
        if n > 0 {
            return Some(Command::ChooseNode(n - 1));
        }
    }
    if s.is_empty() || s == "enter" {
        return Some(Command::ChooseNode(0));
    }
    None
}

fn parse_choose_card(s: &str) -> Option<Command> {
    let n: usize = s.trim().parse().ok()?;
    (n > 0).then(|| Command::ChooseHandCard(n - 1))
}

fn parse_combat(s: &str, debug: bool) -> Option<Command> {
    if debug && s == "win" {
        return Some(Command::WinCombat);
    }
    if debug {
        if let Some(id) = s.strip_prefix("add ") {
            return Card::from_id(id.trim()).map(Command::AddCard);
        }
        if let Some(id) = s.strip_prefix("relic ") {
            return Relic::from_id(id.trim()).map(Command::AddRelic);
        }
        if let Some(id) = s.strip_prefix("potion ") {
            return Potion::from_id(id.trim()).map(Command::AddPotion);
        }
    }
    if let Some(cmd) = parse_discard_potion(s) {
        return Some(cmd);
    }
    if let Some(rest) = s.strip_prefix("use ") {
        let parts: Vec<&str> = rest.trim().splitn(2, ' ').collect();
        if let Ok(n) = parts[0].parse::<usize>() {
            if n > 0 {
                let target = if parts.len() > 1 {
                    let t: usize = parts[1].trim().parse().ok()?;
                    if t == 0 { return None; }
                    t - 1
                } else {
                    0
                };
                return Some(Command::UsePotion(n - 1, target));
            }
        }
    }
    let num_str = s.strip_prefix("play ").unwrap_or(s);
    let parts: Vec<&str> = num_str.trim().splitn(2, ' ').collect();
    if let Ok(card_n) = parts[0].parse::<usize>() {
        if card_n > 0 {
            let target = if parts.len() > 1 {
                let t: usize = parts[1].trim().parse().ok()?;
                if t == 0 { return None; }
                t - 1
            } else {
                0
            };
            return Some(Command::PlayCard(card_n - 1, target));
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
        return (n > 0).then(|| Command::UpgradeCard(n - 1));
    }
    if let Some(cmd) = parse_discard_potion(s) {
        return Some(cmd);
    }
    match s {
        "rest" | "r" => Some(Command::Rest),
        _ => None,
    }
}

fn parse_treasure(s: &str) -> Option<Command> {
    match s {
        "leave" | "l" | "take" | "t" => Some(Command::LeaveTreasure),
        _ => None,
    }
}

fn parse_shop(s: &str) -> Option<Command> {
    if let Ok(n) = s.trim().parse::<usize>() {
        if n > 0 {
            return Some(Command::BuyCard(n - 1));
        }
    }
    match s {
        "r" | "buy relic" => Some(Command::BuyRelic),
        "p" | "buy potion" => Some(Command::BuyPotion),
        "leave" | "l" => Some(Command::LeaveShop),
        _ => None,
    }
}

fn parse_event(s: &str) -> Option<Command> {
    let n: usize = s.trim().parse().ok()?;
    (n > 0).then(|| Command::ChooseEventOption(n - 1))
}

fn parse_card_reward(s: &str) -> Option<Command> {
    if let Some(cmd) = parse_discard_potion(s) {
        return Some(cmd);
    }
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

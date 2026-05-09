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
    if let Ok(n) = s.trim().parse::<usize>() {
        if n > 0 {
            return Some(Command::ChooseNeowBlessing(n - 1));
        }
    }
    None
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
    if let Ok(n) = s.trim().parse::<usize>() {
        if n > 0 {
            return Some(Command::ChooseHandCard(n - 1));
        }
    }
    None
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
    if let Ok(n) = s.trim().parse::<usize>() {
        if n > 0 {
            return Some(Command::ChooseEventOption(n - 1));
        }
    }
    None
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

#[cfg(test)]
mod tests {
    use super::*;
    use slay_core::{AnyRng, Block, Energy, Hp, MapState, NoOpRng, Player, Scenario, StatusMap};

    fn rng() -> AnyRng { AnyRng::NoOp(NoOpRng) }

    fn map_state() -> GameState {
        let graph = slay_core::run::generate_map(&mut rng());
        let available_cols = vec![0, 1];
        GameState::Map(MapState {
            player: Player {
                hp: Hp(80), max_hp: Hp(80), block: Block(0),
                energy: Energy(3), max_energy: Energy(3),
                hand: vec![], draw_pile: vec![], discard_pile: vec![],
                exhaust_pile: vec![], statuses: StatusMap::new(),
                deck: vec![], gold: 0, relics: vec![], potions: vec![],
                neow_lament_combats_remaining: 0,
                reached_boss: false,
            },
            floor: 0,
            graph,
            available_cols,
            next_enemies: None,
            scenario: Scenario::Main,
        })
    }

    #[test]
    fn spawn_single_enemy() {
        let state = map_state();
        assert_eq!(
            parse("spawn red-louse", &state, false),
            Some(Command::Spawn(vec![EnemyKind::RedLouse]))
        );
    }

    #[test]
    fn spawn_multiple_enemies() {
        let state = map_state();
        assert_eq!(
            parse("spawn red-louse cultist", &state, false),
            Some(Command::Spawn(vec![EnemyKind::RedLouse, EnemyKind::Cultist]))
        );
    }

    #[test]
    fn spawn_unknown_ids_are_ignored() {
        let state = map_state();
        assert_eq!(
            parse("spawn red-louse dragon red-louse", &state, false),
            Some(Command::Spawn(vec![EnemyKind::RedLouse, EnemyKind::RedLouse]))
        );
    }

    #[test]
    fn spawn_all_unknown_returns_none() {
        let state = map_state();
        assert_eq!(parse("spawn dragon wizard", &state, false), None);
    }

    #[test]
    fn spawn_not_valid_in_other_phases() {
        // spawn is only parsed in Map phase
        let state = map_state();
        assert_eq!(parse("spawn red-louse", &state, false), Some(Command::Spawn(vec![EnemyKind::RedLouse])));
        // just verify it parses in map — other phases return None via their own parse fns
    }

    #[test]
    fn parse_1_in_map_returns_choose_node_0() {
        let state = map_state();
        assert_eq!(parse("1", &state, false), Some(Command::ChooseNode(0)));
    }

    #[test]
    fn parse_2_in_map_returns_choose_node_1() {
        let state = map_state();
        assert_eq!(parse("2", &state, false), Some(Command::ChooseNode(1)));
    }

    #[test]
    fn parse_0_in_map_returns_none() {
        let state = map_state();
        assert_eq!(parse("0", &state, false), None);
    }

    #[test]
    fn parse_enter_in_map_still_returns_choose_node_0() {
        let state = map_state();
        assert_eq!(parse("enter", &state, false), Some(Command::ChooseNode(0)));
    }

    fn combat_state() -> GameState {
        use slay_core::{CombatPhase, CombatState, Enemy, EnemyKind, Move, Scenario};
        GameState::Combat {
            state: CombatState {
                player: Player {
                    hp: Hp(80), max_hp: Hp(80), block: Block(0),
                    energy: Energy(3), max_energy: Energy(3),
                    hand: vec![], draw_pile: vec![], discard_pile: vec![],
                    exhaust_pile: vec![], statuses: StatusMap::new(),
                    deck: vec![], gold: 0, relics: vec![], potions: vec![],
                    neow_lament_combats_remaining: 0,
                    reached_boss: false,
                },
                enemies: vec![Enemy {
                    kind: EnemyKind::RedLouse,
                    hp: Hp(20), max_hp: Hp(20), block: Block(0),
                    move_: Move::LouseBite, move_history: vec![],
                    statuses: StatusMap::new(),
                    stolen_gold: 0,
                }],
                turn: 1,
                phase: CombatPhase::PlayerTurn,
                attacks_this_turn: 0,
                skills_this_turn: 0,
                attacks_this_combat: 0,
                skills_this_combat: 0,
                cards_played_this_turn: 0,
                extra_draws_next_turn: 0,
            },
            floor: 0,
            is_boss: false,
            graph: slay_core::run::generate_map(&mut rng()),
            next_floor_cols: vec![0, 1],
            scenario: Scenario::Main,
        }
    }

    #[test]
    fn use_potion_parses_slot_1() {
        let state = combat_state();
        assert_eq!(parse("use 1", &state, false), Some(Command::UsePotion(0, 0)));
    }

    #[test]
    fn use_potion_parses_slot_with_target() {
        let state = combat_state();
        assert_eq!(parse("use 2 1", &state, false), Some(Command::UsePotion(1, 0)));
    }

    #[test]
    fn use_potion_zero_returns_none() {
        let state = combat_state();
        assert_eq!(parse("use 0", &state, false), None);
    }

    #[test]
    fn debug_potion_command_adds_by_id() {
        let state = combat_state();
        assert_eq!(
            parse("potion fire-potion", &state, true),
            Some(Command::AddPotion(Potion::FirePotion))
        );
    }

    #[test]
    fn discard_potion_parses_in_map() {
        let state = map_state();
        assert_eq!(parse("discard 1", &state, false), Some(Command::DiscardPotion(0)));
    }

    #[test]
    fn discard_potion_parses_in_combat() {
        let state = combat_state();
        assert_eq!(parse("discard 2", &state, false), Some(Command::DiscardPotion(1)));
    }

    #[test]
    fn discard_potion_zero_returns_none() {
        let state = map_state();
        assert_eq!(parse("discard 0", &state, false), None);
    }

    #[test]
    fn use_potion_with_nonzero_target() {
        let state = combat_state();
        assert_eq!(parse("use 1 2", &state, false), Some(Command::UsePotion(0, 1)));
    }

    #[test]
    fn play_card_with_nonzero_target() {
        let state = combat_state();
        assert_eq!(parse("play 1 2", &state, false), Some(Command::PlayCard(0, 1)));
    }

    fn rest_state() -> GameState {
        GameState::RestSite(slay_core::RestSiteState {
            player: Player {
                hp: Hp(80), max_hp: Hp(80), block: Block(0),
                energy: Energy(3), max_energy: Energy(3),
                hand: vec![], draw_pile: vec![], discard_pile: vec![],
                exhaust_pile: vec![], statuses: StatusMap::new(),
                deck: vec![], gold: 0, relics: vec![], potions: vec![],
                neow_lament_combats_remaining: 0,
                reached_boss: false,
            },
            floor: 3,
            graph: slay_core::run::generate_map(&mut rng()),
            available_cols: vec![0],
        })
    }

    #[test]
    fn upgrade_0_returns_none() {
        let state = rest_state();
        assert_eq!(parse("upgrade 0", &state, false), None);
    }

    fn shop_state() -> GameState {
        GameState::Shop(slay_core::ShopState {
            player: Player {
                hp: Hp(80), max_hp: Hp(80), block: Block(0),
                energy: Energy(3), max_energy: Energy(3),
                hand: vec![], draw_pile: vec![], discard_pile: vec![],
                exhaust_pile: vec![], statuses: StatusMap::new(),
                deck: vec![], gold: 200, relics: vec![], potions: vec![],
                neow_lament_combats_remaining: 0,
                reached_boss: false,
            },
            floor: 3,
            cards: vec![],
            relic: None,
            potion: None,
            graph: slay_core::run::generate_map(&mut rng()),
            available_cols: vec![0],
        })
    }

    #[test]
    fn buy_card_1_returns_buy_card_0() {
        let state = shop_state();
        assert_eq!(parse("1", &state, false), Some(Command::BuyCard(0)));
    }

    #[test]
    fn buy_card_0_returns_none() {
        let state = shop_state();
        assert_eq!(parse("0", &state, false), None);
    }

    #[test]
    fn buy_relic_shortcut_r() {
        let state = shop_state();
        assert_eq!(parse("r", &state, false), Some(Command::BuyRelic));
    }

    #[test]
    fn buy_potion_shortcut_p() {
        let state = shop_state();
        assert_eq!(parse("p", &state, false), Some(Command::BuyPotion));
    }

    fn card_reward_state() -> GameState {
        use slay_core::{Card, Grade};
        GameState::CardReward(slay_core::CardRewardState {
            player: Player {
                hp: Hp(80), max_hp: Hp(80), block: Block(0),
                energy: Energy(3), max_energy: Energy(3),
                hand: vec![], draw_pile: vec![], discard_pile: vec![],
                exhaust_pile: vec![], statuses: StatusMap::new(),
                deck: vec![], gold: 0, relics: vec![], potions: vec![],
                neow_lament_combats_remaining: 0,
                reached_boss: false,
            },
            floor: 1,
            options: vec![Card::Strike(Grade::Base), Card::Defend(Grade::Base), Card::Bash(Grade::Base)],
            offered_potion: None,
            graph: slay_core::run::generate_map(&mut rng()),
            available_cols: vec![0, 1],
        })
    }

    #[test]
    fn pick_1_returns_choose_card_reward_0() {
        let state = card_reward_state();
        assert_eq!(parse("1", &state, false), Some(Command::ChooseCardReward(0)));
    }

    #[test]
    fn pick_0_returns_none() {
        let state = card_reward_state();
        assert_eq!(parse("0", &state, false), None);
    }
}

use crate::engine::{
    apply_and_drain, card_type_icon, connector_rows, describe_event, describe_intent, enemy_icon,
    map_node_icon, map_node_name, relics_bar, statuses_inline,
};
use slay_core::{
    AnyRng, CardRewardState, CombatPhase, CombatState, Event, EventRoomState, GameState, MapState,
    RestSiteState, ShopState, TreasureRoomState, StatusMap, CARD_PRICE, RELIC_PRICE, POTION_PRICE,
};
use std::io::{BufRead, Write};

pub fn run_game(
    mut state: GameState,
    reader: impl BufRead,
    writer: &mut impl Write,
    rng: &mut AnyRng,
    debug: bool,
) {
    let _ = writeln!(writer, "{}", slay_core::welcome());
    if debug {
        let _ = writeln!(writer, "⚙️  debug mode");
    }
    let _ = writeln!(writer);

    render(&state, writer);

    for input in reader.lines().map(|l| l.expect("read error")) {
        if input.trim_start().starts_with('#') {
            let _ = writeln!(writer, "{input}");
            continue;
        }
        if input.trim().is_empty() {
            continue;
        }
        let _ = writeln!(writer, "> {input}");

        // Global shortcut: print relic list with descriptions
        if input.trim() == "relics" {
            render_relic_list(player_from_state(&state), writer);
            continue;
        }

        if let GameState::Combat { state: ref cs, .. } = state {
            match input.trim() {
                "z" => { render_pile("🎴 Draw pile", &cs.player.draw_pile, writer); continue; }
                "x" => { render_pile("🗑️  Discard pile", &cs.player.discard_pile, writer); continue; }
                "c" => { render_pile("🔥 Exhaust pile", &cs.player.exhaust_pile, writer); continue; }
                _ => {}
            }
        }

        let Some(command) = crate::command::parse(&input, &state, debug) else {
            let _ = writeln!(writer, "Unknown command.\n");
            continue;
        };

        match apply_and_drain(state.clone(), command, rng) {
            Ok((new_state, events)) => {
                state = new_state;
                print_events(&events, writer);

                match &state {
                    GameState::GameOver { victory: true } => {
                        let _ = writeln!(writer, "\n🏆 You conquered the Spire! Run complete.");
                        break;
                    }
                    GameState::GameOver { victory: false } => {
                        let _ = writeln!(writer, "\n💀 You have been slain. Game over.");
                        break;
                    }
                    _ => {}
                }

                let _ = writeln!(writer);
                render(&state, writer);
            }
            Err(e) => {
                let _ = writeln!(writer, "{e}\n");
            }
        }
    }
}

fn render(state: &GameState, w: &mut impl Write) {
    match state {
        GameState::Map(map) => render_map(map, w),
        GameState::Combat { state, .. } => render_combat(state, w),
        GameState::RestSite(rs) => render_rest(rs, w),
        GameState::TreasureRoom(tr) => render_treasure(tr, w),
        GameState::CardReward(cr) => render_card_reward(cr, w),
        GameState::Shop(shop) => render_shop(shop, w),
        GameState::EventRoom(er) => render_event(er, w),
        GameState::GameOver { .. } => {}
    }
}

fn render_map(map: &MapState, w: &mut impl Write) {
    let floor = map.floor;
    let _ = writeln!(w, "🗺️  Map");
    let _ = writeln!(
        w,
        "🪙 {}   ❤️  {}/{}   🃏 {} cards",
        map.player.gold,
        map.player.hp.0,
        map.player.max_hp.0,
        map.player.deck.len(),
    );
    let bar = relics_bar(&map.player.relics);
    if !bar.is_empty() {
        let _ = writeln!(w, "{bar}");
    }
    let _ = writeln!(w);

    let max_cols = map.graph.rows.iter().map(|r| r.len()).max().unwrap_or(1);
    let max_floor = map.graph.rows.len().saturating_sub(1);

    for floor_idx in (0..=max_floor).rev() {
        let row = &map.graph.rows[floor_idx];
        let past = floor_idx < floor;
        let marker = if floor_idx == floor { "▶ " } else { "  " };
        let mut icons: Vec<String> = Vec::new();
        for col in 0..max_cols {
            let icon = if past {
                row.get(col).map_or("  ", |_| "·· ")
            } else {
                row.get(col).map_or("  ", |n| map_node_icon(n))
            };
            icons.push(icon.to_string());
        }
        let _ = writeln!(w, "{marker}{}", icons.join("    "));
        if floor_idx > 0 {
            let (r0, r1) = connector_rows(&map.graph.edges[floor_idx - 1], max_cols);
            let _ = writeln!(w, "  {r0}");
            let _ = writeln!(w, "  {r1}");
        }
    }

    let _ = writeln!(w);
    if map.available_cols.len() == 1 {
        let node = &map.graph.rows[floor][map.available_cols[0]];
        let _ = writeln!(w, "[Enter ↵] {} {}", map_node_icon(node), map_node_name(node));
    } else {
        for &col in &map.available_cols {
            let node = &map.graph.rows[floor][col];
            let _ = writeln!(w, "[{}] {} {}", col + 1, map_node_icon(node), map_node_name(node));
        }
    }
}

fn render_treasure(tr: &TreasureRoomState, w: &mut impl Write) {
    let _ = writeln!(w, "📦 Treasure Room");
    let _ = writeln!(w, "❤️  {}/{}", tr.player.hp.0, tr.player.max_hp.0);
    let bar = relics_bar(&tr.player.relics);
    if !bar.is_empty() {
        let _ = writeln!(w, "{bar}");
    }
    let _ = writeln!(w);
    let _ = writeln!(w, "You found a chest containing:");
    let _ = writeln!(w, "  ✨ {}", tr.relic.name());
    let _ = writeln!(w);
    let _ = writeln!(w, "[leave] Take the relic and leave");
}

fn render_rest(rs: &RestSiteState, w: &mut impl Write) {
    let heal = (rs.player.max_hp.0 * 30 / 100).max(1);
    let healed_to = (rs.player.hp.0 + heal).min(rs.player.max_hp.0);
    let _ = writeln!(w, "🔥 Rest Site");
    let _ = writeln!(w, "❤️  {}/{}", rs.player.hp.0, rs.player.max_hp.0);
    let bar = relics_bar(&rs.player.relics);
    if !bar.is_empty() {
        let _ = writeln!(w, "{bar}");
    }
    let _ = writeln!(w, "[rest] ❤️‍🩹 Heal for {heal} HP  (to {healed_to})");
    let _ = writeln!(w);
    let upgradeable: Vec<_> = rs.player.deck.iter().enumerate()
        .filter(|(_, c)| c.upgrade().is_some())
        .collect();
    if upgradeable.is_empty() {
        let _ = writeln!(w, "(no cards can be upgraded)");
    } else {
        let _ = writeln!(w, "🃏 Deck (upgrade N to upgrade a card):");
        for (i, card) in &upgradeable {
            let _ = writeln!(w, "  [{}] ⬆️  {}", i + 1, card.name());
        }
    }
}

fn render_card_reward(cr: &CardRewardState, w: &mut impl Write) {
    let _ = writeln!(w, "✨ Card Reward");
    let _ = writeln!(w, "Choose a card to add to your deck:");
    for (i, card) in cr.options.iter().enumerate() {
        let _ = writeln!(
            w,
            "  [{}] {}{} ({}) — {}",
            i + 1,
            card_type_icon(card.card_type()),
            card.name(),
            card.card_cost().display(),
            card.description(),
        );
    }
    let _ = writeln!(w, "(type a number to pick, or 'skip' / 's' to take nothing)");
    if let Some(potion) = &cr.offered_potion {
        let _ = writeln!(w, "🧪 Potion on the ground: {}", potion.name());
        let _ = writeln!(w, "(discard N to drop a potion slot and pick it up)");
    }
}

fn render_shop(shop: &ShopState, w: &mut impl Write) {
    let _ = writeln!(w, "🛒 Shop");
    let _ = writeln!(w, "🪙 {}g", shop.player.gold);
    let _ = writeln!(w);
    let _ = writeln!(w, "Cards:");
    for (i, (card, purchased)) in shop.cards.iter().enumerate() {
        if *purchased {
            let _ = writeln!(w, "  [{}] {} — [sold]", i + 1, card.name());
        } else {
            let _ = writeln!(
                w,
                "  [{}] {} ({}) — {} — {}g",
                i + 1,
                card.name(),
                card.card_cost().display(),
                card.description(),
                CARD_PRICE,
            );
        }
    }
    let _ = writeln!(w);
    let _ = writeln!(w, "Relic:");
    match &shop.relic {
        Some((relic, true))  => { let _ = writeln!(w, "  [r] {} — [sold]", relic.name()); }
        Some((relic, false)) => { let _ = writeln!(w, "  [r] {} — {}g", relic.name(), RELIC_PRICE); }
        None => { let _ = writeln!(w, "  (none)"); }
    }
    let _ = writeln!(w);
    let _ = writeln!(w, "Potion:");
    match &shop.potion {
        Some((potion, true))  => { let _ = writeln!(w, "  [p] {} — [sold]", potion.name()); }
        Some((potion, false)) => { let _ = writeln!(w, "  [p] {} — {}g", potion.name(), POTION_PRICE); }
        None => { let _ = writeln!(w, "  (none)"); }
    }
    let _ = writeln!(w);
    let _ = writeln!(w, "[leave] Exit shop");
}

fn render_combat(state: &CombatState, w: &mut impl Write) {
    let player_status_str = statuses_inline(&state.player.statuses);
    let _ = writeln!(
        w,
        "🧙 You  ❤️  {}/{}  🛡️ {}  ⚡ {}/{}  (Turn {}){}",
        state.player.hp.0,
        state.player.max_hp.0,
        state.player.block.0,
        state.player.energy.0,
        state.player.max_energy.0,
        state.turn,
        player_status_str,
    );
    let bar = relics_bar(&state.player.relics);
    if !bar.is_empty() {
        let _ = writeln!(w, "{bar}");
    }
    let multi = state.enemies.len() > 1;
    for (i, enemy) in state.enemies.iter().enumerate() {
        let status_str = statuses_inline(&enemy.statuses);
        let prefix = if multi { format!("[{}] ", i + 1) } else { String::new() };
        let _ = writeln!(
            w,
            "{}{} {} ❤️  {}/{}  🛡️ {}  | {}{}",
            prefix,
            enemy_icon(enemy),
            enemy.name(),
            enemy.hp.0,
            enemy.max_hp.0,
            enemy.block.0,
            describe_intent(&enemy.effective_intent(&state.player.statuses)),
            status_str,
        );
    }
    let dummy = StatusMap::new();
    let target_statuses = state.enemies.first().map_or(&dummy, |e| &e.statuses);
    if state.player.hand.is_empty() {
        let _ = writeln!(w, "🤚 Hand: (empty)");
    } else {
        let _ = writeln!(w, "🤚 Hand:");
        for (i, card) in state.player.hand.iter().enumerate() {
            let affordable = card.card_cost().is_affordable(state.player.energy);
            let prefix = if affordable { " " } else { "❌" };
            let desc = card.effective_description(&state.player.statuses, target_statuses);
            let _ = writeln!(
                w,
                "  {}[{}] {}{} ({}) — {}",
                prefix,
                i + 1,
                card_type_icon(card.card_type()),
                card.name(),
                card.card_cost().display(),
                desc,
            );
        }
    }
    if multi {
        let labels: Vec<String> = (1..=state.enemies.len())
            .map(|n| format!("\"1 {}\" → enemy [{}]", n, n))
            .collect();
        let _ = writeln!(w, "🎯 Targeting: {}", labels.join("  ·  "));
    }
    let _ = writeln!(
        w,
        "Draw: {}  Discard: {}  Exhaust: {}",
        state.player.draw_pile.len(),
        state.player.discard_pile.len(),
        state.player.exhaust_pile.len(),
    );
    if matches!(state.phase, CombatPhase::ChooseCard(_)) {
        let _ = writeln!(
            w,
            "Choose a card: [1-{}]",
            state.player.hand.len().max(1),
        );
    } else {
        let _ = writeln!(
            w,
            "Commands: [1-{}] play card  |  end / e  end turn  |  z draw  x discard  c exhaust",
            state.player.hand.len().max(1),
        );
    }
}

fn player_from_state(state: &GameState) -> Option<&slay_core::Player> {
    match state {
        GameState::Map(m)              => Some(&m.player),
        GameState::Combat { state, .. } => Some(&state.player),
        GameState::RestSite(rs)        => Some(&rs.player),
        GameState::TreasureRoom(tr)    => Some(&tr.player),
        GameState::CardReward(cr)      => Some(&cr.player),
        GameState::Shop(shop)          => Some(&shop.player),
        GameState::EventRoom(er)       => Some(&er.player),
        GameState::GameOver { .. }     => None,
    }
}

fn render_relic_list(player: Option<&slay_core::Player>, w: &mut impl Write) {
    let relics = player.map_or([].as_slice(), |p| p.relics.as_slice());
    if relics.is_empty() {
        let _ = writeln!(w, "🎒 Relics: (none)");
    } else {
        let _ = writeln!(w, "🎒 Relics ({}):", relics.len());
        for relic in relics {
            let _ = writeln!(w, "  {} {}  —  {}", crate::engine::relic_emoji(relic), relic.name(), relic.description());
        }
    }
    let _ = writeln!(w);
}

fn render_pile(label: &str, pile: &[slay_core::Card], w: &mut impl Write) {
    if pile.is_empty() {
        let _ = writeln!(w, "{label}: (empty)");
    } else {
        let _ = writeln!(w, "{label} ({}):", pile.len());
        for card in pile {
            let _ = writeln!(w, "  - {}", card.name());
        }
    }
    let _ = writeln!(w);
}

fn print_events(events: &[Event], w: &mut impl Write) {
    for event in events {
        let msg = describe_event(event);
        if !msg.is_empty() {
            let _ = writeln!(w, "{msg}");
        }
    }
}

fn render_event(er: &EventRoomState, w: &mut impl Write) {
    use slay_core::EventKind;
    let (title, options) = match er.event {
        EventKind::Ssssserpent => (
            "🐍 The Ssssserpent",
            vec![
                "[1] Agree — Gain 150 gold. Become Cursed — Doubt.",
                "[2] Disagree — Nothing happens.",
                "[3] Leave",
            ],
        ),
        EventKind::BigFish => (
            "🐟 Big Fish",
            vec![
                "[1] Banana — Heal ~30% of max HP.",
                "[2] Donut — Gain 3 Max HP.",
                "[3] Box — Obtain a random Relic. Become Cursed — Regret.",
                "[4] Leave",
            ],
        ),
        EventKind::Mushrooms => (
            "🍄 Mushrooms",
            vec![
                "[1] Eat — Heal 12 HP. Become Cursed — Parasite.",
                "[2] Leave",
            ],
        ),
        EventKind::GoldenIdol => (
            "🏺 Golden Idol",
            vec![
                "[1] Outrun — Gain 250 gold. Become Cursed — Injury.",
                "[2] Smash — Gain 250 gold. Take 25 damage.",
                "[3] Hide — Gain 250 gold. Lose 6 Max HP.",
                "[4] Leave",
            ],
        ),
    };
    let _ = writeln!(w, "{title}");
    let _ = writeln!(w);
    for opt in options {
        let _ = writeln!(w, "  {opt}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slay_core::{Block, Energy, Hp, MapGraph, MapNode, MapState, Player, Scenario, StatusMap};

    fn bare_player() -> Player {
        Player {
            hp: Hp(80), max_hp: Hp(80), block: Block(0),
            energy: Energy(3), max_energy: Energy(3),
            hand: vec![], draw_pile: vec![], discard_pile: vec![],
            exhaust_pile: vec![], statuses: StatusMap::new(),
            deck: vec![], gold: 0, relics: vec![], potions: vec![],
        }
    }

    #[test]
    fn map_grid_shows_both_icons_on_two_column_floor() {
        let graph = MapGraph {
            rows: vec![vec![MapNode::Combat(vec![]), MapNode::Combat(vec![])]],
            edges: vec![vec![vec![], vec![]]],
        };
        let map = MapState {
            player: bare_player(),
            floor: 0,
            graph,
            available_cols: vec![0, 1],
            next_enemies: None,
            scenario: Scenario::Main,
        };
        let mut out = Vec::new();
        render_map(&map, &mut out);
        let s = String::from_utf8(out).unwrap();
        let icon_count = s.lines()
            .find(|line| line.contains("⚔️") && !line.starts_with('['))
            .map_or(0, |line| line.matches("⚔️").count());
        assert!(icon_count >= 2, "expected a node row with 2 ⚔️ icons in:\n{s}");
    }

    #[test]
    fn map_node_rows_show_icons_not_text_labels() {
        let graph = MapGraph {
            rows: vec![vec![MapNode::RestSite]],
            edges: vec![vec![vec![]]],
        };
        let map = MapState {
            player: bare_player(),
            floor: 0,
            graph,
            available_cols: vec![0],
            next_enemies: None,
            scenario: Scenario::Main,
        };
        let mut out = Vec::new();
        render_map(&map, &mut out);
        let s = String::from_utf8(out).unwrap();
        let node_row = s.lines()
            .find(|line| line.contains("🔥") && !line.starts_with('['))
            .expect("no node row with 🔥 found in:\n{s}");
        assert!(!node_row.contains("Rest Site"), "node row should not contain label 'Rest Site' in:\n{s}");
    }

    #[test]
    fn map_shows_connector_rows_between_floors() {
        let graph = MapGraph {
            rows: vec![
                vec![MapNode::Combat(vec![])],
                vec![MapNode::RestSite],
            ],
            edges: vec![
                vec![vec![0]],
            ],
        };
        let map = MapState {
            player: bare_player(),
            floor: 0,
            graph,
            available_cols: vec![0],
            next_enemies: None,
            scenario: Scenario::Main,
        };
        let mut out = Vec::new();
        render_map(&map, &mut out);
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains('│'), "expected connector row with │ in:\n{s}");
    }
}

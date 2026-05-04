use crate::engine::{
    apply_and_drain, card_type_icon, describe_event, describe_intent, enemy_icon, statuses_inline,
};
use slay_core::{
    AnyRng, CardRewardState, CombatState, Event, GameState, MapState, RestSiteState, ShopState,
    StatusMap, CARD_PRICE, RELIC_PRICE, POTION_PRICE,
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
        GameState::CardReward(cr) => render_card_reward(cr, w),
        GameState::Shop(shop) => render_shop(shop, w),
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
    let _ = writeln!(w);
    for (i, row) in map.graph.rows.iter().enumerate().rev() {
        let node = &row[0];
        let (icon, name) = map_node_label(node);
        let marker = match i.cmp(&floor) {
            std::cmp::Ordering::Less    => "✓",
            std::cmp::Ordering::Equal   => "▶",
            std::cmp::Ordering::Greater => " ",
        };
        let _ = writeln!(w, "  {marker} {}. {icon} {name}", i + 1);
    }
    let _ = writeln!(w);
    if map.available_cols.len() == 1 {
        let node = &map.graph.rows[floor][map.available_cols[0]];
        let (icon, name) = map_node_label(node);
        let _ = writeln!(w, "[Enter ↵] {icon} {name}");
    } else {
        for &col in &map.available_cols {
            let node = &map.graph.rows[floor][col];
            let (icon, name) = map_node_label(node);
            let _ = writeln!(w, "[{}] {icon} {name}", col + 1);
        }
    }
}

fn map_node_label(node: &slay_core::MapNode) -> (&'static str, &'static str) {
    match node {
        slay_core::MapNode::Combat(_) => ("⚔️ ", "Combat"),
        slay_core::MapNode::RestSite  => ("🔥", "Rest Site"),
        slay_core::MapNode::Boss(_)   => ("💀", "Boss"),
        slay_core::MapNode::Merchant  => ("🛒", "Shop"),
    }
}

fn render_rest(rs: &RestSiteState, w: &mut impl Write) {
    let heal = (rs.player.max_hp.0 * 30 / 100).max(1);
    let healed_to = (rs.player.hp.0 + heal).min(rs.player.max_hp.0);
    let _ = writeln!(w, "🔥 Rest Site");
    let _ = writeln!(w, "❤️  {}/{}", rs.player.hp.0, rs.player.max_hp.0);
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
            card.energy_cost().0,
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
                card.energy_cost().0,
                card.description(),
                CARD_PRICE,
            );
        }
    }
    let _ = writeln!(w);
    let _ = writeln!(w, "Relic:");
    match &shop.relic {
        Some((relic, true))  => { let _ = writeln!(w, "  [r] {} — [sold]", relic.id()); }
        Some((relic, false)) => { let _ = writeln!(w, "  [r] {} — {}g", relic.id(), RELIC_PRICE); }
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
            let affordable = card.energy_cost() <= state.player.energy;
            let prefix = if affordable { " " } else { "❌" };
            let desc = card.effective_description(&state.player.statuses, target_statuses);
            let _ = writeln!(
                w,
                "  {}[{}] {}{} ({}) — {}",
                prefix,
                i + 1,
                card_type_icon(card.card_type()),
                card.name(),
                card.energy_cost().0,
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
    let _ = writeln!(
        w,
        "Commands: [1-{}] play card  |  end / e  end turn  |  z draw  x discard  c exhaust",
        state.player.hand.len().max(1),
    );
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

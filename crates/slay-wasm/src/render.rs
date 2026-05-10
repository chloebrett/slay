use std::io::Write;
use slay_core::{
    CardRewardState, CombatPhase, CombatState, Event, EventRoomState, GameState, MapState,
    RestSiteState, ShopState, StatusMap, TreasureRoomState, CARD_PRICE, POTION_PRICE, RELIC_PRICE,
};
use crate::engine::{
    card_type_icon, connector_rows, describe_event, describe_intent, enemy_icon, map_node_icon,
    map_node_name, relics_bar, relic_emoji, statuses_inline,
};

pub fn render(state: &GameState, w: &mut impl Write) {
    match state {
        GameState::Map(map)             => render_map(map, w),
        GameState::Combat { state, .. } => render_combat(state, w),
        GameState::RestSite(rs)         => render_rest(rs, w),
        GameState::TreasureRoom(tr)     => render_treasure(tr, w),
        GameState::CardReward(cr)       => render_card_reward(cr, w),
        GameState::Shop(shop)           => render_shop(shop, w),
        GameState::EventRoom(er)        => render_event(er, w),
        GameState::Neow(neow)           => render_neow(neow, w),
        GameState::GameOver { .. }      => {}
    }
}

pub fn print_events(events: &[Event], w: &mut impl Write) {
    for event in events {
        let msg = describe_event(event);
        if !msg.is_empty() {
            let _ = writeln!(w, "{msg}");
        }
    }
}

fn render_neow(neow: &slay_core::NeowState, w: &mut impl Write) {
    let _ = writeln!(w, "🌟 Neow's Blessings");
    let _ = writeln!(w, "Choose a blessing:");
    for (i, blessing) in neow.blessings.iter().enumerate() {
        let _ = writeln!(w, "  [{}] {:?}", i + 1, blessing);
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
        let _ = writeln!(w, "Choose a card: [1-{}]", state.player.hand.len().max(1));
    } else {
        let _ = writeln!(
            w,
            "Commands: [1-{}] play card  |  end / e  end turn  |  z draw  x discard  c exhaust",
            state.player.hand.len().max(1),
        );
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

pub fn render_relic_list(player: Option<&slay_core::Player>, w: &mut impl Write) {
    let relics = player.map_or([].as_slice(), |p| p.relics.as_slice());
    if relics.is_empty() {
        let _ = writeln!(w, "🎒 Relics: (none)");
    } else {
        let _ = writeln!(w, "🎒 Relics ({}):", relics.len());
        for relic in relics {
            let _ = writeln!(w, "  {} {}  —  {}", relic_emoji(relic), relic.name(), relic.description());
        }
    }
    let _ = writeln!(w);
}

pub fn render_pile(label: &str, pile: &[slay_core::Card], w: &mut impl Write) {
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

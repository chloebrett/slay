use slay_core::{
    apply_command, AnyRng, CardRewardState, CardType, CombatPhase, CombatState, Enemy, EnemyKind,
    Event, GameState, Intent, MapState, RestSiteState, StatusEffect, StatusMap, Target,
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

        match apply_command(state.clone(), command, rng) {
            Ok((new_state, events)) => {
                state = new_state;
                print_events(&events, writer);

                loop {
                    let is_enemy_turn = matches!(
                        &state,
                        GameState::Combat { state: cs, .. } if cs.phase == CombatPhase::EnemyTurn
                    );
                    if !is_enemy_turn {
                        break;
                    }
                    match apply_command(state.clone(), slay_core::Command::EndEnemyTurn, rng) {
                        Ok((ns, evts)) => {
                            state = ns;
                            print_events(&evts, writer);
                        }
                        Err(e) => {
                            let _ = writeln!(writer, "Internal error advancing enemy turn: {e:?}");
                            break;
                        }
                    }
                }

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
                let _ = writeln!(writer, "Error: {e:?}\n");
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
        GameState::GameOver { .. } => {}
    }
}

fn render_map(map: &MapState, w: &mut impl Write) {
    let floor = map.floor;
    let nodes = slay_core::run::MAP_NODES;
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
    for (i, node) in nodes.iter().enumerate().rev() {
        let (icon, name) = match node {
            slay_core::MapNode::Combat => ("⚔️ ", "Combat"),
            slay_core::MapNode::RestSite => ("🔥", "Rest Site"),
            slay_core::MapNode::Boss => ("💀", "Boss"),
        };
        let marker = match i.cmp(&floor) {
            std::cmp::Ordering::Less    => "✓",
            std::cmp::Ordering::Equal   => "▶",
            std::cmp::Ordering::Greater => " ",
        };
        let _ = writeln!(w, "  {marker} {}. {icon} {name}", i + 1);
    }
    let _ = writeln!(w);
    let node = nodes.get(floor).unwrap_or(&slay_core::MapNode::Combat);
    let (icon, name) = match node {
        slay_core::MapNode::Combat => ("⚔️ ", "Combat"),
        slay_core::MapNode::RestSite => ("🔥", "Rest Site"),
        slay_core::MapNode::Boss => ("💀", "Boss"),
    };
    let _ = writeln!(w, "[Enter ↵] {icon} {name}");
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
}

fn card_type_icon(card_type: CardType) -> &'static str {
    match card_type {
        CardType::Attack => "⚔️ ",
        CardType::Skill  => "🪄 ",
        CardType::Power  => "🔮 ",
    }
}

fn enemy_icon(enemy: &Enemy) -> &'static str {
    match enemy.kind {
        EnemyKind::Louse => "🐛",
        EnemyKind::Fungibeast => "🍄",
        EnemyKind::Cultist => "🐦",
        EnemyKind::JawWorm => "🪱",
        EnemyKind::SmallSpikeSlime => "🫧",
        EnemyKind::RedLouse => "🦟",
    }
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
            describe_intent(&enemy.move_.intent()),
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
        let msg = describe(event);
        if !msg.is_empty() {
            let _ = writeln!(w, "{msg}");
        }
    }
}

fn status_display(status: StatusEffect) -> (&'static str, &'static str) {
    match status {
        StatusEffect::Vulnerable => ("🎯", "Vulnerable"),
        StatusEffect::Weak       => ("🪫", "Weak"),
        StatusEffect::Poison     => ("🟢", "Poison"),
        StatusEffect::Strength   => ("💪", "Strength"),
        StatusEffect::Ritual     => ("🔮", "Ritual"),
        StatusEffect::Dexterity  => ("🛡️", "Dexterity"),
    }
}

fn statuses_inline(statuses: &StatusMap) -> String {
    if statuses.is_empty() {
        return String::new();
    }
    let parts: Vec<String> = statuses
        .iter()
        .map(|(s, n)| {
            let (icon, _) = status_display(*s);
            format!("{icon}{n}")
        })
        .collect();
    format!("  [{}]", parts.join(" "))
}

fn describe_intent(intent: &Intent) -> String {
    match intent {
        Intent::Attack(n) => format!("⚔️  Attack {n}"),
        Intent::Defend(n) => format!("🛡️  Defend {n}"),
        Intent::AttackDefend(d, b) => format!("⚔️🛡️  Attack {d} + Defend {b}"),
        Intent::Buff => "✨ Buff".into(),
    }
}

fn describe(event: &Event) -> String {
    match event {
        Event::CardPlayed { card } => format!("▶ You play {}.", card.name()),
        Event::PlayerAttacked { raw, damage } => {
            if *damage == 0 {
                format!("⚔️  You attack {raw}. (fully blocked)")
            } else if *damage < *raw {
                format!("⚔️  You deal {damage} damage. ({} blocked by enemy)", raw - damage)
            } else {
                format!("⚔️  You deal {damage} damage.")
            }
        }
        Event::PlayerBlocked { amount } => format!("🛡️  You gain {amount} block."),
        Event::EnemyAttacked { raw, damage } => {
            if *damage == 0 {
                format!("⚔️  Enemy attacks {raw}. (fully blocked)")
            } else if *damage < *raw {
                format!("⚔️  Enemy attacks {raw}. ({} blocked, {damage} damage)", raw - damage)
            } else {
                format!("⚔️  Enemy attacks {damage}.")
            }
        }
        Event::EnemyDefended { amount } => format!("🛡️  Enemy gains {amount} block."),
        Event::IntentRevealed { intent } => format!("👁  Enemy prepares: {}.", describe_intent(intent)),
        Event::PlayerBlockExpired { amount } => format!("🛡️  Your {amount} block expired."),
        Event::EnemyDied => "💀 Enemy slain!".into(),
        Event::PlayerDied => "💀 You have been slain.".into(),
        Event::EnemyPoisoned { damage } => format!("{} Poison deals {damage} to enemy.", status_display(StatusEffect::Poison).0),
        Event::TurnEnded => String::new(),
        Event::TurnStarted { turn } => format!("─── Turn {turn} ───"),
        Event::StatusApplied { target, status, stacks } => {
            let (icon, name) = status_display(*status);
            match target {
                Target::Player => format!("{icon} You gain {stacks} {name}."),
                Target::Enemy => format!("{icon} Enemy gains {stacks} {name}."),
            }
        }
        Event::GoldEarned { amount } => format!("🪙 You earn {amount} gold."),
        Event::Healed { amount } => format!("❤️‍🩹 You heal for {amount} HP."),
        Event::PlayerSelfDamaged { amount } => format!("🩸 You lose {amount} HP."),
        Event::EnergyGained { amount } => format!("⚡ You gain {amount} energy."),
        Event::CardsDrawn { count } => format!("🃏 You draw {count} card{}.", if *count == 1 { "" } else { "s" }),
        Event::CardAdded { card } => format!("✨ {} added to your deck.", card.name()),
        Event::CardExhausted { card } => format!("🔥 {} was exhausted.", card.name()),
        Event::CardUpgraded { from, to } => format!("⬆️  {} upgraded to {}.", from.name(), to.name()),
        Event::StatusCardAddedToDiscard { card } => format!("🃏 {} added to your discard.", card.name()),
    }
}

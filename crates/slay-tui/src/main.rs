use slay_core::{
    apply_command, new_run, CardRewardState, CardType, CombatPhase, CombatState, Enemy, EnemyKind,
    Event, GameState, Intent, MapState, RestSiteState, StatusEffect, StatusMap, Target, ThreadRng,
};
use std::io::{self, BufRead, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let debug = args.iter().any(|a| a == "--debug");
    let script = args.windows(2)
        .find(|w| w[0] == "--script")
        .map(|w| w[1].clone());

    let mut rng = ThreadRng::new();
    let mut state = new_run(&mut rng);

    println!("{}", slay_core::welcome());
    if debug { println!("⚙️  debug mode"); }
    println!();

    render(&state);

    let scripted = script.is_some();
    let reader: Box<dyn BufRead> = match script {
        Some(path) => {
            let file = std::fs::File::open(&path)
                .unwrap_or_else(|e| panic!("Cannot open script {path}: {e}"));
            Box::new(io::BufReader::new(file))
        }
        None => Box::new(io::BufReader::new(io::stdin())),
    };

    for input in reader.lines().map(|l| l.expect("read error")) {
        if scripted && input.trim_start().starts_with('#') {
            println!("{input}");
            continue;
        }
        if scripted {
            println!("> {input}");
        } else {
            print!("> ");
            io::stdout().flush().ok();
        }
        if let GameState::Combat { state: cs, .. } = &state {
            match input.trim() {
                "z" => { render_pile("🎴 Draw pile", &cs.player.draw_pile); continue; }
                "x" => { render_pile("🗑️  Discard pile", &cs.player.discard_pile); continue; }
                "c" => { render_pile("🔥 Exhaust pile", &cs.player.exhaust_pile); continue; }
                _ => {}
            }
        }

        let Some(command) = slay_tui::command::parse(&input, &state, debug) else {
            println!("Unknown command.\n");
            continue;
        };

        match apply_command(state.clone(), command, &mut rng) {
            Ok((new_state, events)) => {
                state = new_state;
                print_events(&events);

                // Auto-drain EnemyTurn — no player decisions needed
                loop {
                    let is_enemy_turn = matches!(
                        &state,
                        GameState::Combat { state: cs, .. } if cs.phase == CombatPhase::EnemyTurn
                    );
                    if !is_enemy_turn {
                        break;
                    }
                    match apply_command(
                        state.clone(),
                        slay_core::Command::EndEnemyTurn,
                        &mut rng,
                    ) {
                        Ok((ns, evts)) => {
                            state = ns;
                            print_events(&evts);
                        }
                        Err(e) => {
                            println!("Internal error advancing enemy turn: {e:?}");
                            break;
                        }
                    }
                }

                match &state {
                    GameState::GameOver { victory: true } => {
                        println!("\n🏆 You conquered the Spire! Run complete.");
                        break;
                    }
                    GameState::GameOver { victory: false } => {
                        println!("\n💀 You have been slain. Game over.");
                        break;
                    }
                    _ => {}
                }

                println!();
                render(&state);
            }
            Err(e) => println!("Error: {e:?}\n"),
        }
    }
}

fn render(state: &GameState) {
    match state {
        GameState::Map(map) => render_map(map),
        GameState::Combat { state, .. } => render_combat(state),
        GameState::RestSite(rs) => render_rest(rs),
        GameState::CardReward(cr) => render_card_reward(cr),
        GameState::GameOver { .. } => {}
    }
}

fn render_map(map: &MapState) {
    let floor = map.floor;
    let nodes = slay_core::run::MAP_NODES;
    println!("🗺️  Map");
    println!(
        "🪙 {}   ❤️  {}/{}   🃏 {} cards",
        map.player.gold,
        map.player.hp.0,
        map.player.max_hp.0,
        map.player.deck.len(),
    );
    println!();
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
        println!("  {marker} {}. {icon} {name}", i + 1);
    }
    println!();
    let (icon, name) = match &nodes[floor] {
        slay_core::MapNode::Combat => ("⚔️ ", "Combat"),
        slay_core::MapNode::RestSite => ("🔥", "Rest Site"),
        slay_core::MapNode::Boss => ("💀", "Boss"),
    };
    println!("[Enter ↵] {icon} {name}");
}

fn render_rest(rs: &RestSiteState) {
    let heal = (rs.player.max_hp.0 * 30 / 100).max(1);
    let healed_to = (rs.player.hp.0 + heal).min(rs.player.max_hp.0);
    println!("🔥 Rest Site");
    println!("❤️  {}/{}", rs.player.hp.0, rs.player.max_hp.0);
    println!("[rest] ❤️‍🩹 Heal for {heal} HP  (to {healed_to})");
    println!();
    let upgradeable: Vec<_> = rs.player.deck.iter().enumerate()
        .filter(|(_, c)| c.upgrade().is_some())
        .collect();
    if upgradeable.is_empty() {
        println!("(no cards can be upgraded)");
    } else {
        println!("🃏 Deck (upgrade N to upgrade a card):");
        for (i, card) in &upgradeable {
            println!("  [{}] ⬆️  {}", i + 1, card.name());
        }
    }
}

fn render_card_reward(cr: &CardRewardState) {
    println!("✨ Card Reward");
    println!("Choose a card to add to your deck:");
    for (i, card) in cr.options.iter().enumerate() {
        println!(
            "  [{}] {}{} ({}) — {}",
            i + 1,
            card_type_icon(card.card_type()),
            card.name(),
            card.energy_cost().0,
            card.description(),
        );
    }
    println!("(type a number to pick, or 'skip' / 's' to take nothing)");
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

fn render_combat(state: &CombatState) {
    let player_status_str = statuses_inline(&state.player.statuses);
    println!(
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
        println!(
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
        println!("🤚 Hand: (empty)");
    } else {
        println!("🤚 Hand:");
        for (i, card) in state.player.hand.iter().enumerate() {
            let affordable = card.energy_cost() <= state.player.energy;
            let prefix = if affordable { " " } else { "❌" };
            let desc = card.effective_description(&state.player.statuses, target_statuses);
            println!(
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
        println!("🎯 Targeting: {}", labels.join("  ·  "));
    }
    println!(
        "Commands: [1-{}] play card  |  end / e  end turn  |  z draw  x discard  c exhaust",
        state.player.hand.len().max(1),
    );
}

fn render_pile(label: &str, pile: &[slay_core::Card]) {
    if pile.is_empty() {
        println!("{label}: (empty)");
    } else {
        println!("{label} ({}):", pile.len());
        for card in pile {
            println!("  - {}", card.name());
        }
    }
    println!();
}

fn print_events(events: &[Event]) {
    for event in events {
        let msg = describe(event);
        if !msg.is_empty() {
            println!("{msg}");
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

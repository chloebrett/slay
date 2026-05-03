use slay_core::{
    apply_command, new_run, CardRewardState, CombatPhase, CombatState, Event, GameState, Intent,
    MapState, RestSiteState, StatusEffect, StatusMap, Target, ThreadRng,
};
use std::io::{self, BufRead, Write};

fn main() {
    let mut rng = ThreadRng::new();
    let mut state = new_run(&mut rng);

    println!("{}", slay_core::welcome());
    println!();

    render(&state);

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        print!("> ");
        io::stdout().flush().ok();

        let input = line.expect("failed to read input");
        let Some(command) = slay_tui::command::parse(&input, &state) else {
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
                        println!("\nYou conquered the Spire! Run complete.");
                        break;
                    }
                    GameState::GameOver { victory: false } => {
                        println!("\nYou have been slain. Game over.");
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
    println!("=== Map ===");
    println!(
        "Gold: {}   HP: {}/{}   Deck: {} cards",
        map.player.gold,
        map.player.hp.0,
        map.player.max_hp.0,
        map.player.deck.len(),
    );
    println!();
    for (i, node) in nodes.iter().enumerate().rev() {
        let name = match node {
            slay_core::MapNode::Combat => "Combat",
            slay_core::MapNode::RestSite => "Rest Site",
            slay_core::MapNode::Boss => "Boss",
        };
        let marker = if i < floor {
            "  -"
        } else if i == floor {
            "  >"
        } else {
            "   "
        };
        println!("{marker} {}. {name}", i + 1);
    }
    println!();
    let current_name = match &nodes[floor] {
        slay_core::MapNode::Combat => "Combat",
        slay_core::MapNode::RestSite => "Rest Site",
        slay_core::MapNode::Boss => "Boss",
    };
    println!("[1] Enter: {current_name}");
}

fn render_rest(rs: &RestSiteState) {
    let heal = (rs.player.max_hp.0 * 30 / 100).max(1);
    let healed_to = (rs.player.hp.0 + heal).min(rs.player.max_hp.0);
    println!("=== Rest Site ===");
    println!("HP: {}/{}", rs.player.hp.0, rs.player.max_hp.0);
    println!("[rest] Heal for {heal} HP  (to {healed_to})");
}

fn render_card_reward(cr: &CardRewardState) {
    println!("=== Card Reward ===");
    println!("Choose a card to add to your deck:");
    for (i, card) in cr.options.iter().enumerate() {
        println!(
            "  [{}] {} ({}) — {}",
            i + 1,
            card.name(),
            card.energy_cost().0,
            card.description(),
        );
    }
    println!("(type a number to pick, or 'skip' / 's' to take nothing)");
}

fn render_combat(state: &CombatState) {
    let enemy_status_str = statuses_inline(&state.enemy.statuses);
    println!(
        "[ {} ] HP: {}/{}  Block: {}  | Intent: {}{}",
        state.enemy.name(),
        state.enemy.hp.0,
        state.enemy.max_hp.0,
        state.enemy.block.0,
        describe_intent(&state.enemy.intent),
        enemy_status_str,
    );
    let player_status_str = statuses_inline(&state.player.statuses);
    println!(
        "[ You  ] HP: {}/{}  Block: {}  Energy: {}/{}  (Turn {}){}",
        state.player.hp.0,
        state.player.max_hp.0,
        state.player.block.0,
        state.player.energy.0,
        state.player.max_energy.0,
        state.turn,
        player_status_str,
    );
    if state.player.hand.is_empty() {
        println!("Hand: (empty)");
    } else {
        println!("Hand:");
        for (i, card) in state.player.hand.iter().enumerate() {
            let affordable = card.energy_cost() <= state.player.energy;
            let prefix = if affordable { " " } else { "×" };
            let desc = card.effective_description(&state.player.statuses, &state.enemy.statuses);
            println!(
                "  {}[{}] {} ({}) — {}",
                prefix,
                i + 1,
                card.name(),
                card.energy_cost().0,
                desc,
            );
        }
    }
    println!("Commands: [1-{}] play card  |  end / e  end turn", state.player.hand.len().max(1));
}

fn print_events(events: &[Event]) {
    for event in events {
        let msg = describe(event);
        if !msg.is_empty() {
            println!("{msg}");
        }
    }
}

fn statuses_inline(statuses: &StatusMap) -> String {
    if statuses.is_empty() {
        return String::new();
    }
    let parts: Vec<String> = statuses
        .iter()
        .map(|(s, n)| {
            let name = match s {
                StatusEffect::Vulnerable => "Vuln",
                StatusEffect::Weak => "Weak",
                StatusEffect::Poison => "Psn",
                StatusEffect::Strength => "Str",
            };
            format!("{name} {n}")
        })
        .collect();
    format!("  [{}]", parts.join(", "))
}

fn describe_intent(intent: &Intent) -> String {
    match intent {
        Intent::Attack(n) => format!("Attack {n}"),
        Intent::Defend(n) => format!("Defend {n}"),
    }
}

fn describe(event: &Event) -> String {
    match event {
        Event::CardPlayed { card } => format!("You play {}.", card.name()),
        Event::PlayerAttacked { raw, damage } => {
            if *damage == 0 {
                format!("You attack {raw}. (fully blocked)")
            } else if *damage < *raw {
                format!("You deal {damage} damage. ({} blocked by enemy)", raw - damage)
            } else {
                format!("You deal {damage} damage.")
            }
        }
        Event::PlayerBlocked { amount } => format!("You gain {amount} block."),
        Event::EnemyAttacked { raw, damage } => {
            if *damage == 0 {
                format!("Enemy attacks {raw}. (fully blocked)")
            } else if *damage < *raw {
                format!("Enemy attacks {raw}. ({} blocked, {damage} damage)", raw - damage)
            } else {
                format!("Enemy attacks {damage}.")
            }
        }
        Event::EnemyDefended { amount } => format!("Enemy gains {amount} block."),
        Event::IntentRevealed { intent } => format!("Enemy prepares: {}.", describe_intent(intent)),
        Event::PlayerBlockExpired { amount } => format!("Your {amount} block expired."),
        Event::EnemyDied => "Enemy slain!".into(),
        Event::PlayerDied => "You have been slain.".into(),
        Event::EnemyPoisoned { damage } => format!("Poison deals {damage} to enemy."),
        Event::TurnEnded => String::new(),
        Event::TurnStarted { turn } => format!("--- Turn {turn} ---"),
        Event::StatusApplied { target, status, stacks } => {
            let name = match status {
                StatusEffect::Vulnerable => "Vulnerable",
                StatusEffect::Weak => "Weak",
                StatusEffect::Poison => "Poison",
                StatusEffect::Strength => "Strength",
            };
            match target {
                Target::Player => format!("You gain {stacks} {name}."),
                Target::Enemy => format!("Enemy gains {stacks} {name}."),
            }
        }
        Event::GoldEarned { amount } => format!("You earn {amount} gold."),
        Event::Healed { amount } => format!("You heal for {amount} HP."),
        Event::CardAdded { card } => format!("{} added to your deck.", card.name()),
        Event::CardExhausted { card } => format!("{} was exhausted.", card.name()),
    }
}

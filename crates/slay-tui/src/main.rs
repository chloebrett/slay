use slay_core::{apply_command, Command, CombatPhase, CombatState, Event, Intent, StatusEffect, StatusMap, Target, ThreadRng};
use std::io::{self, BufRead, Write};

fn main() {
    let mut rng = ThreadRng::new();
    let mut state = CombatState::new(&mut rng);

    println!("{}", slay_core::welcome());
    println!("Commands: <n> to play card n, end (or e) to end turn\n");

    render(&state);

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        print!("> ");
        io::stdout().flush().ok();

        let input = line.expect("failed to read input");
        let Some(command) = slay_tui::command::parse(&input) else {
            println!("Unknown command.\n");
            continue;
        };

        match apply_command(state.clone(), command, &mut rng) {
            Ok((new_state, events)) => {
                state = new_state;
                print_events(&events);
                // Auto-drain EnemyTurn — no player decisions during it
                while state.phase == CombatPhase::EnemyTurn {
                    match apply_command(state.clone(), Command::EndEnemyTurn, &mut rng) {
                        Ok((new_state, events)) => {
                            state = new_state;
                            print_events(&events);
                        }
                        Err(e) => {
                            println!("Internal error advancing enemy turn: {e:?}");
                            break;
                        }
                    }
                }
                if state.phase == CombatPhase::Victory {
                    println!("You win!");
                    break;
                }
                if state.phase == CombatPhase::Defeat {
                    break;
                }
                println!();
                render(&state);
            }
            Err(e) => println!("Error: {e:?}\n"),
        }
    }
}

fn print_events(events: &[Event]) {
    for event in events {
        let msg = describe(event);
        if !msg.is_empty() {
            println!("{msg}");
        }
    }
}

fn render(state: &CombatState) {
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
            let eff = card.effective_damage(&state.player.statuses, &state.enemy.statuses);
            let damage_hint = match (card.def().base_damage, eff) {
                (Some(base), Some(eff)) if eff != base => {
                    format!("  →{eff} vs {}", state.enemy.name())
                }
                _ => String::new(),
            };
            println!(
                "  {}[{}] {} ({}) — {}{}",
                prefix,
                i + 1,
                card.name(),
                card.energy_cost().0,
                card.description(),
                damage_hint,
            );
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
        Event::EnemyDied => String::new(),
        Event::PlayerDied => "You have been slain.".into(),
        Event::TurnEnded => String::new(),
        Event::TurnStarted { turn } => format!("--- Turn {turn} ---"),
        Event::StatusApplied { target, status, stacks } => {
            let who = match target {
                Target::Player => "You",
                Target::Enemy => "Enemy",
            };
            let name = match status {
                StatusEffect::Vulnerable => "Vulnerable",
                StatusEffect::Weak => "Weak",
            };
            format!("{who} gains {stacks} {name}.")
        }
    }
}

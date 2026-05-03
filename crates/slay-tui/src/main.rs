use slay_core::{apply_command, Command, CombatPhase, CombatState, Event, Intent, ThreadRng};
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
    println!(
        "[ {} ] HP: {}/{}  Block: {}  | Intent: {}",
        state.enemy.name(),
        state.enemy.hp.0,
        state.enemy.max_hp.0,
        state.enemy.block.0,
        describe_intent(&state.enemy.intent),
    );
    println!(
        "[ You  ] HP: {}/{}  Block: {}  Energy: {}/{}  (Turn {})",
        state.player.hp.0,
        state.player.max_hp.0,
        state.player.block.0,
        state.player.energy.0,
        state.player.max_energy.0,
        state.turn
    );
    if state.player.hand.is_empty() {
        println!("Hand: (empty)");
    } else {
        println!("Hand:");
        for (i, card) in state.player.hand.iter().enumerate() {
            let affordable = card.energy_cost() <= state.player.energy;
            let prefix = if affordable { " " } else { "×" };
            println!(
                "  {}[{}] {} ({}) — {}",
                prefix,
                i + 1,
                card.name(),
                card.energy_cost().0,
                card.description()
            );
        }
    }
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
    }
}

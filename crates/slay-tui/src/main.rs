use slay_core::{apply_command, Command, CombatPhase, CombatState, Event, Intent, ThreadRng};
use std::io::{self, BufRead, Write};

fn main() {
    let mut rng = ThreadRng::new();
    let mut state = CombatState::new(&mut rng);

    println!("{}", slay_core::welcome());
    println!("Commands: <n> to play card n, end to end turn\n");

    render(&state);

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = line.expect("failed to read input");
        let Some(command) = slay_tui::command::parse(&input) else {
            println!("Unknown command. Try: play 1, end\n");
            continue;
        };

        match apply_command(state.clone(), command, &mut rng) {
            Ok((new_state, events)) => {
                state = new_state;
                for event in &events {
                    println!("{}", describe(event));
                }
                // Auto-drain EnemyTurn — player makes no decisions during it
                while state.phase == CombatPhase::EnemyTurn {
                    match apply_command(state.clone(), Command::EndEnemyTurn, &mut rng) {
                        Ok((new_state, events)) => {
                            state = new_state;
                            for event in &events {
                                println!("{}", describe(event));
                            }
                        }
                        Err(e) => {
                            println!("Internal error advancing enemy turn: {e:?}");
                            break;
                        }
                    }
                }
                println!();
                render(&state);
                if state.phase == CombatPhase::Victory {
                    println!("You win!");
                    break;
                }
                if state.phase == CombatPhase::Defeat {
                    println!("You died.");
                    break;
                }
            }
            Err(e) => println!("Invalid command: {e:?}\n"),
        }

        print!("> ");
        io::stdout().flush().ok();
    }
}

fn render(state: &CombatState) {
    println!(
        "[ {} ] HP: {}  Block: {}  | {}",
        state.enemy.name(),
        state.enemy.hp.0,
        state.enemy.block.0,
        describe_intent(&state.enemy.intent),
    );
    println!(
        "[ You  ] HP: {}  Block: {}  Energy: {}/{}  (Turn {})",
        state.player.hp.0,
        state.player.block.0,
        state.player.energy.0,
        state.player.max_energy.0,
        state.turn
    );
    println!("Hand:");
    for (i, card) in state.player.hand.iter().enumerate() {
        println!("  [{}] {} — {}", i + 1, card.name(), card.description());
    }
    print!("> ");
    io::stdout().flush().ok();
}

fn describe_intent(intent: &Intent) -> String {
    match intent {
        Intent::Attack(n) => format!("Intent: Attack {n}"),
        Intent::Defend(n) => format!("Intent: Defend {n}"),
    }
}

fn describe(event: &Event) -> String {
    match event {
        Event::CardPlayed { index } => format!("You play card {}.", index + 1),
        Event::PlayerAttacked { damage } => format!("You deal {damage} damage."),
        Event::PlayerBlocked { amount } => format!("You gain {amount} block."),
        Event::EnemyAttacked { damage } => format!("Enemy deals {damage} damage."),
        Event::EnemyDefended { amount } => format!("Enemy gains {amount} block."),
        Event::IntentRevealed { intent } => format!("Enemy prepares: {}", describe_intent(intent)),
        Event::EnemyDied => "The enemy is defeated!".into(),
        Event::PlayerDied => "You have been slain.".into(),
        Event::TurnEnded => "Turn ended.".into(),
        Event::TurnStarted { turn } => format!("--- Turn {turn} ---"),
    }
}

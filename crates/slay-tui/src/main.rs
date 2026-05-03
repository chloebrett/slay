use slay_core::{apply_command, CombatPhase, CombatState, Event};
use std::io::{self, BufRead, Write};

fn main() {
    let mut state = CombatState::new();

    println!("{}", slay_core::welcome());
    println!("Commands: attack (a), block (b), end\n");

    render(&state);

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = line.expect("failed to read input");
        let Some(command) = slay_tui::command::parse(&input) else {
            println!("Unknown command. Try: attack, block, end\n");
            continue;
        };

        match apply_command(state.clone(), command) {
            Ok((new_state, events)) => {
                state = new_state;
                for event in &events {
                    println!("{}", describe(event));
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
        "[ {} ] HP: {}  Block: {}",
        state.enemy.name, state.enemy.hp.0, state.enemy.block.0
    );
    println!(
        "[ You  ] HP: {}  Block: {}  (Turn {})",
        state.player.hp.0, state.player.block.0, state.turn
    );
    print!("> ");
    io::stdout().flush().ok();
}

fn describe(event: &Event) -> String {
    match event {
        Event::PlayerAttacked { damage } => format!("You deal {damage} damage."),
        Event::PlayerBlocked { amount } => format!("You gain {amount} block."),
        Event::EnemyAttacked { damage } => format!("Enemy deals {damage} damage."),
        Event::EnemyDied => "The enemy is defeated!".into(),
        Event::PlayerDied => "You have been slain.".into(),
        Event::TurnEnded => "Turn ended.".into(),
        Event::TurnStarted { turn } => format!("--- Turn {turn} ---"),
    }
}

use slay_core::{new_run, AnyRng, GameState};
use std::io::{self, BufRead, IsTerminal, Write};
use std::sync::mpsc;
use std::thread;

fn prompt_continue(save_exists: bool) -> bool {
    if !save_exists {
        return false;
    }
    print!("A saved run was found. Continue? [y/n]: ");
    io::stdout().flush().ok();
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line).ok();
    matches!(line.trim().to_lowercase().as_str(), "y" | "yes")
}

fn start_save_writer() -> mpsc::SyncSender<Option<(GameState, u64)>> {
    let (tx, rx) = mpsc::sync_channel::<Option<(GameState, u64)>>(32);
    thread::spawn(move || {
        for msg in &rx {
            match msg {
                Some((state, seed)) => slay_tui::save::save_run(&state, seed),
                None => break,
            }
        }
    });
    tx
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let debug = args.iter().any(|a| a == "--debug");
    let plain = args.iter().any(|a| a == "--plain");
    let script = args
        .windows(2)
        .find(|w| w[0] == "--script")
        .map(|w| w[1].clone());

    let existing_save = slay_tui::save::load_run();
    let continue_run = prompt_continue(existing_save.is_some());

    let (state, seed) = if continue_run {
        let (state, seed) = existing_save.unwrap();
        (state, seed)
    } else {
        slay_tui::save::delete_run();
        let seed: u64 = rand::random();
        let mut rng = AnyRng::seeded(seed);
        (new_run(&mut rng), seed)
    };

    let mut rng = AnyRng::seeded(seed);
    let save_tx = start_save_writer();

    let save_tx_ctrlc = save_tx.clone();
    ctrlc::set_handler(move || {
        let _ = save_tx_ctrlc.send(None);
        std::process::exit(0);
    }).ok();

    if let Some(path) = script {
        let file = std::fs::File::open(&path)
            .unwrap_or_else(|e| panic!("Cannot open script {path}: {e}"));
        let reader: Box<dyn BufRead> = Box::new(io::BufReader::new(file));
        let mut stdout = io::stdout();
        slay_tui::game::run_game(state, reader, &mut stdout, &mut rng, debug, None);
        return;
    }

    let use_tui = !plain && io::stdout().is_terminal() && io::stdin().is_terminal();

    if use_tui {
        if let Err(e) = slay_tui::tui::run_tui(state, &mut rng, debug, Some(save_tx.clone())) {
            eprintln!("TUI error: {e}");
            std::process::exit(1);
        }
    } else {
        let reader: Box<dyn BufRead> = Box::new(io::BufReader::new(io::stdin()));
        let mut stdout = io::stdout();
        slay_tui::game::run_game(state, reader, &mut stdout, &mut rng, debug, Some(save_tx.clone()));
    }

    let _ = save_tx.send(None);
}

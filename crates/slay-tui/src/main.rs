use slay_core::{new_run, AnyRng, ThreadRng};
use std::io::{self, BufRead, IsTerminal};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let debug = args.iter().any(|a| a == "--debug");
    let plain = args.iter().any(|a| a == "--plain");
    let script = args
        .windows(2)
        .find(|w| w[0] == "--script")
        .map(|w| w[1].clone());

    let mut thread_rng = ThreadRng::new();
    let state = new_run(&mut thread_rng);
    let mut rng = AnyRng::Thread(thread_rng);

    if let Some(path) = script {
        let file = std::fs::File::open(&path)
            .unwrap_or_else(|e| panic!("Cannot open script {path}: {e}"));
        let reader: Box<dyn BufRead> = Box::new(io::BufReader::new(file));
        let mut stdout = io::stdout();
        slay_tui::game::run_game(state, reader, &mut stdout, &mut rng, debug);
        return;
    }

    let use_tui = !plain && io::stdout().is_terminal() && io::stdin().is_terminal();

    if use_tui {
        if let Err(e) = slay_tui::tui::run_tui(state, &mut rng, debug) {
            eprintln!("TUI error: {e}");
            std::process::exit(1);
        }
    } else {
        let reader: Box<dyn BufRead> = Box::new(io::BufReader::new(io::stdin()));
        let mut stdout = io::stdout();
        slay_tui::game::run_game(state, reader, &mut stdout, &mut rng, debug);
    }
}

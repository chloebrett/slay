use slay_core::{AnyRng, ThreadRng, new_run};
use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let debug = args.iter().any(|a| a == "--debug");
    let script = args.windows(2)
        .find(|w| w[0] == "--script")
        .map(|w| w[1].clone());

    let mut thread_rng = ThreadRng::new();
    let state = new_run(&mut thread_rng);
    let mut rng = AnyRng::Thread(thread_rng);

    let reader: Box<dyn BufRead> = match script {
        Some(path) => {
            let file = std::fs::File::open(&path)
                .unwrap_or_else(|e| panic!("Cannot open script {path}: {e}"));
            Box::new(io::BufReader::new(file))
        }
        None => Box::new(io::BufReader::new(io::stdin())),
    };

    let mut stdout = io::stdout();
    slay_tui::game::run_game(state, reader, &mut stdout, &mut rng, debug);
}

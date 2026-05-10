use slay_core::{AnyRng, Command, NeowContext, NoOpRng, new_run, new_simple_run};
use std::fs;
use std::path::Path;

fn run_script(script_content: &str) -> String {
    let state = new_simple_run();
    let mut rng = AnyRng::NoOp(NoOpRng);
    let mut output = Vec::<u8>::new();
    slay_tui::game::run_game(state, script_content.as_bytes(), &mut output, &mut rng, true, None);
    String::from_utf8(output).expect("output is valid utf8")
}

fn run_seeded_map_script(seed: u64, script_content: &str) -> String {
    let mut rng = AnyRng::seeded(seed);
    let state = new_run(&mut rng, &NeowContext::default());
    let (state, _) = slay_core::apply_command(state, Command::ChooseNeowBlessing(0), &mut rng).unwrap();
    let mut output = Vec::<u8>::new();
    slay_tui::game::run_game(state, script_content.as_bytes(), &mut output, &mut rng, false, None);
    String::from_utf8(output).expect("output is valid utf8")
}

fn scripts_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/scripts")
}

#[test]
fn snapshot_all_simple_scripts() {
    let dir = scripts_dir();
    let mut entries: Vec<_> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("scripts/simple directory not found: {e}"))
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "slay"))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    assert!(!entries.is_empty(), "no .slay scripts found in scripts/simple/");

    for entry in entries {
        let path = entry.path();
        let name = path.file_stem().unwrap().to_string_lossy().into_owned();
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        let output = run_script(&content);
        insta::assert_snapshot!(name, output);
    }
}

#[test]
fn snapshot_seeded_map_seed_1() {
    let output = run_seeded_map_script(1, "");
    insta::assert_snapshot!(output);
}

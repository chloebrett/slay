use slay_core::{AnyRng, NoOpRng, new_simple_run};
use std::fs;
use std::path::Path;

fn run_script(script_content: &str) -> String {
    let state = new_simple_run();
    let mut rng = AnyRng::NoOp(NoOpRng);
    let mut output = Vec::<u8>::new();
    slay_tui::game::run_game(state, script_content.as_bytes(), &mut output, &mut rng, true);
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
        .filter(|e| e.path().extension().map_or(false, |x| x == "slay"))
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

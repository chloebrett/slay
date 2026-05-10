use slay_wasm::WasmSession;

#[test]
fn new_session_initial_render_is_non_empty() {
    let mut session = WasmSession::new();
    let output = session.send("");
    assert!(!output.is_empty(), "initial render should not be empty");
}

#[test]
fn send_win_combat_in_debug_mode_returns_output() {
    let mut session = WasmSession::from_simple_run(true);
    let _enter_combat = session.send("1");
    let output = session.send("win");
    assert!(!output.is_empty(), "win command should produce output");
    assert!(!session.is_over(), "game should not be over after winning first combat");
}

#[test]
fn full_combat_round_trip() {
    // simple run: Map → Combat → Card Reward → Map
    let mut session = WasmSession::from_simple_run(true);

    // Enter combat.
    let enter_output = session.send("1");
    assert!(enter_output.contains("🧙"), "should show combat view: {enter_output}");
    assert!(!session.is_over());

    // Win combat (debug).
    let win_output = session.send("win");
    assert!(!win_output.is_empty(), "win should produce output");
    assert!(!session.is_over(), "game not over after one combat");

    // After winning we should be at card reward — skip it.
    let render = session.send("");
    assert!(
        render.contains("Card Reward") || render.contains("🗺"),
        "should be at card reward or map after combat: {render}"
    );

    // If card reward, skip it.
    if render.contains("Card Reward") {
        let skip_output = session.send("skip");
        assert!(!skip_output.is_empty());
        assert!(!session.is_over());

        // Should now be back on Map.
        let map_render = session.send("");
        assert!(map_render.contains("🗺"), "should be on map after skipping reward: {map_render}");
    }
}

#[test]
fn unknown_command_returns_error_message() {
    let mut session = WasmSession::from_simple_run(false);
    let output = session.send("zzz_invalid_xyz");
    assert!(output.contains("Unknown"), "should report unknown command: {output}");
}

#[test]
fn is_over_false_at_start() {
    let session = WasmSession::from_simple_run(false);
    assert!(!session.is_over());
}

#[test]
fn send_empty_returns_current_state() {
    let mut session = WasmSession::from_simple_run(false);
    let first = session.send("");
    let second = session.send("");
    assert_eq!(first, second, "send('') twice should return same render");
}

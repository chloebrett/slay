use ratatui::{Terminal, widgets::Paragraph};
use slay_wasm::WasmBackend;

#[test]
fn wasm_backend_draw_paragraph_contains_ansi_sequences() {
    let backend = WasmBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| {
        let area = f.area();
        f.render_widget(Paragraph::new("hello wasm"), area);
    }).unwrap();
    let output = terminal.backend().output();
    // Ratatui diffs against the blank initial buffer, so only non-space characters
    // are sent. "hello" and "wasm" appear as runs of characters with a cursor
    // reposition in between where the space was.
    assert!(output.contains("hello"), "rendered text should contain 'hello': {output:?}");
    assert!(output.contains("wasm"), "rendered text should contain 'wasm': {output:?}");
    assert!(output.contains('\x1b'), "output should contain ANSI escape sequences: {output:?}");
}

#[test]
fn wasm_session_send_returns_ansi_output() {
    let mut session = slay_wasm::WasmSession::new_tui(true);
    // Advance to combat via simple run.
    let _enter = session.send("1");
    let combat = session.send("");
    // The TUI renders HP bar and combat info.
    assert!(combat.contains('\x1b'), "TUI output should contain ANSI escapes: {combat:?}");
}

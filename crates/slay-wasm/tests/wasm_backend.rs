use ratatui::{Terminal, widgets::Paragraph};
use slay_wasm::WasmBackend;

fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            for nc in chars.by_ref() {
                if nc.is_ascii_alphabetic() { break; }
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[test]
fn wasm_backend_draw_paragraph_contains_ansi_sequences() {
    let backend = WasmBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| {
        let area = f.area();
        f.render_widget(Paragraph::new("hello wasm"), area);
    }).unwrap();
    let output = terminal.backend().output();
    assert!(output.contains('\x1b'), "output should contain ANSI escape sequences: {output:?}");
    // Each character gets its own absolute cursor position, so check plain text after stripping.
    let plain = strip_ansi(output);
    assert!(plain.contains("hello"), "plain text should contain 'hello': {plain:?}");
    assert!(plain.contains("wasm"), "plain text should contain 'wasm': {plain:?}");
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

/// A platform-agnostic key event. Crossterm events are converted to this type
/// before being passed to `handle_key`, allowing the renderer to be used from
/// both the native TUI and the WASM build.
#[derive(Debug, Clone, PartialEq)]
pub enum Key {
    Char(char),
    Enter,
    Backspace,
    Esc,
    Up,
    Down,
    CtrlC,
    Other,
}

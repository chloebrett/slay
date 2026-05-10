use std::io::Write;
use wasm_bindgen::prelude::*;
use slay_core::{AnyRng, GameState, NeowContext, NoOpRng, ThreadRng};

mod command;
mod engine;
mod persist;
mod render;
mod wasm_backend;

pub use wasm_backend::WasmBackend;

#[wasm_bindgen]
pub struct WasmSession {
    state: GameState,
    rng: AnyRng,
    debug: bool,
}

#[wasm_bindgen]
impl WasmSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmSession {
        let mut rng = AnyRng::Thread(ThreadRng::new());
        let ctx = NeowContext { runs_completed: 0, prev_run_reached_boss: false };
        let state = slay_core::new_run(&mut rng, &ctx);
        WasmSession { state, rng, debug: false }
    }

    pub fn send(&mut self, input: &str) -> String {
        let mut out: Vec<u8> = Vec::new();

        if input.trim().is_empty() {
            render::render(&self.state, &mut out);
            return String::from_utf8_lossy(&out).into_owned();
        }

        if input.trim() == "relics" {
            let player = player_from_state(&self.state);
            render::render_relic_list(player, &mut out);
            return String::from_utf8_lossy(&out).into_owned();
        }

        if let GameState::Combat { state: ref cs, .. } = self.state {
            match input.trim() {
                "z" => { render::render_pile("🎴 Draw pile", &cs.player.draw_pile, &mut out); return String::from_utf8_lossy(&out).into_owned(); }
                "x" => { render::render_pile("🗑️  Discard pile", &cs.player.discard_pile, &mut out); return String::from_utf8_lossy(&out).into_owned(); }
                "c" => { render::render_pile("🔥 Exhaust pile", &cs.player.exhaust_pile, &mut out); return String::from_utf8_lossy(&out).into_owned(); }
                _ => {}
            }
        }

        let Some(command) = command::parse(input, &self.state, self.debug) else {
            let _ = writeln!(out, "Unknown command.");
            return String::from_utf8_lossy(&out).into_owned();
        };

        match engine::apply_and_drain(self.state.clone(), command, &mut self.rng) {
            Ok((new_state, events)) => {
                self.state = new_state;
                render::print_events(&events, &mut out);
                match &self.state {
                    GameState::GameOver { victory } => {
                        if *victory {
                            let _ = writeln!(out, "\n🏆 You conquered the Spire! Run complete.");
                        } else {
                            let _ = writeln!(out, "\n💀 You have been slain. Game over.");
                        }
                    }
                    _ => {
                        let _ = writeln!(out);
                        render::render(&self.state, &mut out);
                    }
                }
            }
            Err(e) => {
                let _ = writeln!(out, "{e}");
            }
        }

        String::from_utf8_lossy(&out).into_owned()
    }

    pub fn is_over(&self) -> bool {
        matches!(self.state, GameState::GameOver { .. })
    }
}

impl WasmSession {
    pub fn from_simple_run(debug: bool) -> WasmSession {
        WasmSession {
            state: slay_core::new_simple_run(),
            rng: AnyRng::NoOp(NoOpRng),
            debug,
        }
    }

    /// Creates a session that renders via the ratatui TUI (returns ANSI sequences).
    pub fn new_tui(debug: bool) -> TuiSession {
        let state = slay_core::new_simple_run();
        let rng = AnyRng::NoOp(NoOpRng);
        TuiSession::new(state, rng, debug)
    }
}

/// A wasm-bindgen-exposed TUI session backed by ratatui + WasmBackend.
/// Returns ANSI escape sequences; pass directly to `term.write()` in xterm.js.
#[wasm_bindgen]
pub struct WasmTuiSession {
    inner: TuiSession,
    game_over_recorded: bool,
}

#[wasm_bindgen]
impl WasmTuiSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmTuiSession {
        let mut rng = AnyRng::Thread(ThreadRng::new());
        #[cfg(feature = "browser")]
        let state = persist::start_or_resume(&persist::LocalStorage, &mut rng);
        #[cfg(not(feature = "browser"))]
        let state = {
            let ctx = NeowContext { runs_completed: 0, prev_run_reached_boss: false };
            slay_core::new_run(&mut rng, &ctx)
        };
        WasmTuiSession { inner: TuiSession::new(state, rng, false), game_over_recorded: false }
    }

    pub fn send(&mut self, input: &str) -> String {
        self.inner.process(input);
        self.after_action()
    }

    pub fn send_key(&mut self, key: &str) -> String {
        use slay_tui::key::Key;
        let k = match key {
            "Enter"     => Key::Enter,
            "Backspace" => Key::Backspace,
            "Esc"       => Key::Esc,
            "Up"        => Key::Up,
            "Down"      => Key::Down,
            _           => return self.inner.render(),
        };
        slay_tui::tui::handle_key(&mut self.inner.tui, &mut self.inner.rng, k);
        self.after_action()
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> String {
        self.inner.resize(cols, rows);
        self.inner.render()
    }

    pub fn is_over(&self) -> bool {
        self.inner.is_over()
    }
}

impl WasmTuiSession {
    fn after_action(&mut self) -> String {
        #[cfg(feature = "browser")]
        self.persist();
        self.inner.render()
    }

    #[cfg(feature = "browser")]
    fn persist(&mut self) {
        let storage = &persist::LocalStorage;
        if self.inner.is_over() {
            if !self.game_over_recorded {
                let victory = matches!(self.inner.tui.game, GameState::GameOver { victory } if victory);
                persist::on_run_end(storage, victory);
                self.game_over_recorded = true;
            }
        } else {
            persist::save_run(storage, &self.inner.tui.game, 0);
        }
    }
}

// Prevent duplicate impl block for WasmSession.
impl WasmSession {
}

/// A game session that renders via the ratatui TUI, returning ANSI sequences
/// suitable for display in xterm.js.
pub struct TuiSession {
    tui: slay_tui::tui::TuiState,
    rng: AnyRng,
    terminal: ratatui::Terminal<WasmBackend>,
}

impl TuiSession {
    pub fn new(state: GameState, rng: AnyRng, debug: bool) -> Self {
        let tui = slay_tui::tui::TuiState::new(state, debug);
        let backend = WasmBackend::new(120, 40);
        let terminal = ratatui::Terminal::new(backend).expect("terminal init");
        TuiSession { tui, rng, terminal }
    }

    pub fn process(&mut self, input: &str) {
        use slay_tui::key::Key;

        for c in input.chars() {
            let key = match c {
                '\n' | '\r' => Key::Enter,
                '\x08' | '\x7f' => Key::Backspace,
                '\x1b' => Key::Esc,
                _ => Key::Char(c),
            };
            slay_tui::tui::handle_key(&mut self.tui, &mut self.rng, key);
        }
    }

    pub fn send(&mut self, input: &str) -> String {
        self.process(input);
        self.render()
    }

    pub fn render(&mut self) -> String {
        self.terminal.draw(|f| slay_tui::tui::render_frame(f, &self.tui)).ok();
        self.terminal.backend_mut().take_output()
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.terminal.backend_mut().resize(cols, rows);
        let _ = self.terminal.resize(ratatui::layout::Rect::new(0, 0, cols, rows));
    }

    pub fn is_over(&self) -> bool {
        matches!(self.tui.game, GameState::GameOver { .. })
    }
}

fn player_from_state(state: &GameState) -> Option<&slay_core::Player> {
    match state {
        GameState::Map(m)               => Some(&m.player),
        GameState::Combat { state, .. } => Some(&state.player),
        GameState::RestSite(rs)         => Some(&rs.player),
        GameState::TreasureRoom(tr)     => Some(&tr.player),
        GameState::CardReward(cr)       => Some(&cr.player),
        GameState::Shop(shop)           => Some(&shop.player),
        GameState::EventRoom(er)        => Some(&er.player),
        GameState::Neow(neow)           => Some(&neow.player),
        GameState::GameOver { .. }      => None,
    }
}

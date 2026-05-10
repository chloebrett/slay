use std::io::Write;
use wasm_bindgen::prelude::*;
use slay_core::{AnyRng, GameState, NeowContext, NoOpRng, ThreadRng};

mod command;
mod engine;
mod render;

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

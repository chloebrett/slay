use slay_core::{AnyRng, GameState, NeowContext};
#[cfg(test)]
use std::cell::RefCell;
#[cfg(test)]
use std::collections::HashMap;

const SCHEMA_VERSION: u32 = 1;
const KEY_RUN: &str = "slay_run";
const KEY_META: &str = "slay_meta";

#[derive(serde::Serialize, serde::Deserialize)]
struct SaveFile {
    schema_version: u32,
    rng_seed: u64,
    state: GameState,
}

#[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
pub struct WasmMeta {
    pub runs_completed: u32,
    pub runs_won: u32,
    pub prev_run_reached_boss: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct MetaSaveFile {
    schema_version: u32,
    runs_completed: u32,
    runs_won: u32,
    prev_run_reached_boss: bool,
}

pub trait Storage {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&self, key: &str, value: &str);
    fn remove(&self, key: &str);
}

#[cfg(test)]
pub struct MemoryStorage(RefCell<HashMap<String, String>>);

#[cfg(test)]
impl MemoryStorage {
    pub fn new() -> Self {
        MemoryStorage(RefCell::new(HashMap::new()))
    }
}

#[cfg(test)]
impl Storage for MemoryStorage {
    fn get(&self, key: &str) -> Option<String> {
        self.0.borrow().get(key).cloned()
    }
    fn set(&self, key: &str, value: &str) {
        self.0.borrow_mut().insert(key.to_string(), value.to_string());
    }
    fn remove(&self, key: &str) {
        self.0.borrow_mut().remove(key);
    }
}

#[cfg(feature = "browser")]
pub struct LocalStorage;

#[cfg(feature = "browser")]
impl Storage for LocalStorage {
    fn get(&self, key: &str) -> Option<String> {
        web_sys::window()?.local_storage().ok()??.get_item(key).ok()?
    }
    fn set(&self, key: &str, value: &str) {
        if let Some(Ok(Some(s))) = web_sys::window().map(|w| w.local_storage()) {
            let _ = s.set_item(key, value);
        }
    }
    fn remove(&self, key: &str) {
        if let Some(Ok(Some(s))) = web_sys::window().map(|w| w.local_storage()) {
            let _ = s.remove_item(key);
        }
    }
}

pub fn save_run(storage: &impl Storage, state: &GameState, seed: u64) {
    let file = SaveFile { schema_version: SCHEMA_VERSION, rng_seed: seed, state: state.clone() };
    if let Ok(ron) = ron::ser::to_string_pretty(&file, ron::ser::PrettyConfig::default()) {
        storage.set(KEY_RUN, &ron);
    }
}

pub fn load_run(storage: &impl Storage) -> Option<(GameState, u64)> {
    let text = storage.get(KEY_RUN)?;
    let file: SaveFile = match ron::de::from_str(&text) {
        Ok(f) => f,
        Err(_) => { storage.remove(KEY_RUN); return None; }
    };
    if file.schema_version != SCHEMA_VERSION {
        storage.remove(KEY_RUN);
        return None;
    }
    Some((file.state, file.rng_seed))
}

pub fn delete_run(storage: &impl Storage) {
    storage.remove(KEY_RUN);
}

pub fn save_meta(storage: &impl Storage, meta: &WasmMeta) {
    let file = MetaSaveFile {
        schema_version: SCHEMA_VERSION,
        runs_completed: meta.runs_completed,
        runs_won: meta.runs_won,
        prev_run_reached_boss: meta.prev_run_reached_boss,
    };
    if let Ok(ron) = ron::ser::to_string_pretty(&file, ron::ser::PrettyConfig::default()) {
        storage.set(KEY_META, &ron);
    }
}

pub fn load_meta(storage: &impl Storage) -> WasmMeta {
    let text = match storage.get(KEY_META) {
        Some(t) => t,
        None => return WasmMeta::default(),
    };
    let file: MetaSaveFile = match ron::de::from_str(&text) {
        Ok(f) => f,
        Err(_) => return WasmMeta::default(),
    };
    if file.schema_version != SCHEMA_VERSION {
        return WasmMeta::default();
    }
    WasmMeta {
        runs_completed: file.runs_completed,
        runs_won: file.runs_won,
        prev_run_reached_boss: file.prev_run_reached_boss,
    }
}

pub fn on_run_end(storage: &impl Storage, victory: bool) {
    let mut meta = load_meta(storage);
    meta.runs_completed += 1;
    if victory {
        meta.runs_won += 1;
        meta.prev_run_reached_boss = true;
    }
    save_meta(storage, &meta);
    delete_run(storage);
}

pub fn neow_context(storage: &impl Storage) -> NeowContext {
    let meta = load_meta(storage);
    NeowContext {
        runs_completed: meta.runs_completed,
        prev_run_reached_boss: meta.prev_run_reached_boss,
    }
}

pub fn start_or_resume(
    storage: &impl Storage,
    rng: &mut AnyRng,
) -> GameState {
    if let Some((state, _seed)) = load_run(storage) {
        return state;
    }
    let ctx = neow_context(storage);
    slay_core::new_run(rng, &ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use slay_core::new_simple_run;

    fn mem() -> MemoryStorage { MemoryStorage::new() }

    #[test]
    fn save_and_load_run_round_trips() {
        let s = mem();
        let state = new_simple_run();
        save_run(&s, &state, 42);
        let (loaded, seed) = load_run(&s).expect("should load");
        assert_eq!(loaded, state);
        assert_eq!(seed, 42);
    }

    #[test]
    fn load_run_returns_none_when_storage_empty() {
        let s = mem();
        assert!(load_run(&s).is_none());
    }

    #[test]
    fn load_run_returns_none_and_clears_corrupt_data() {
        let s = mem();
        s.set(KEY_RUN, "not valid ron");
        assert!(load_run(&s).is_none());
        assert!(s.get(KEY_RUN).is_none(), "corrupt entry should be removed");
    }

    #[test]
    fn delete_run_removes_saved_run() {
        let s = mem();
        save_run(&s, &new_simple_run(), 0);
        delete_run(&s);
        assert!(load_run(&s).is_none());
    }

    #[test]
    fn save_and_load_meta_round_trips() {
        let s = mem();
        let meta = WasmMeta { runs_completed: 5, runs_won: 2, prev_run_reached_boss: true };
        save_meta(&s, &meta);
        let loaded = load_meta(&s);
        assert_eq!(loaded.runs_completed, 5);
        assert_eq!(loaded.runs_won, 2);
        assert!(loaded.prev_run_reached_boss);
    }

    #[test]
    fn load_meta_returns_defaults_when_empty() {
        let s = mem();
        let meta = load_meta(&s);
        assert_eq!(meta.runs_completed, 0);
        assert_eq!(meta.runs_won, 0);
        assert!(!meta.prev_run_reached_boss);
    }

    #[test]
    fn on_run_end_increments_runs_completed() {
        let s = mem();
        on_run_end(&s, false);
        assert_eq!(load_meta(&s).runs_completed, 1);
        on_run_end(&s, false);
        assert_eq!(load_meta(&s).runs_completed, 2);
    }

    #[test]
    fn on_run_end_increments_runs_won_on_victory() {
        let s = mem();
        on_run_end(&s, true);
        let meta = load_meta(&s);
        assert_eq!(meta.runs_won, 1);
        assert!(meta.prev_run_reached_boss);
    }

    #[test]
    fn on_run_end_does_not_increment_runs_won_on_defeat() {
        let s = mem();
        on_run_end(&s, false);
        let meta = load_meta(&s);
        assert_eq!(meta.runs_won, 0);
        assert!(!meta.prev_run_reached_boss);
    }

    #[test]
    fn on_run_end_deletes_run_save() {
        let s = mem();
        save_run(&s, &new_simple_run(), 0);
        on_run_end(&s, false);
        assert!(load_run(&s).is_none());
    }

    #[test]
    fn start_or_resume_returns_saved_run_when_present() {
        let s = mem();
        let state = new_simple_run();
        save_run(&s, &state, 0);
        let mut rng = slay_core::AnyRng::NoOp(slay_core::NoOpRng);
        let resumed = start_or_resume(&s, &mut rng);
        assert_eq!(resumed, state);
    }

    #[test]
    fn start_or_resume_starts_fresh_when_no_save() {
        let s = mem();
        let mut rng = slay_core::AnyRng::Thread(slay_core::ThreadRng::new());
        let state = start_or_resume(&s, &mut rng);
        assert!(!matches!(state, slay_core::GameState::GameOver { .. }));
    }

    #[test]
    fn neow_context_reflects_meta() {
        let s = mem();
        save_meta(&s, &WasmMeta { runs_completed: 3, runs_won: 1, prev_run_reached_boss: true });
        let ctx = neow_context(&s);
        assert_eq!(ctx.runs_completed, 3);
        assert!(ctx.prev_run_reached_boss);
    }
}

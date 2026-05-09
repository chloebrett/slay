use slay_core::GameState;
use std::path::{Path, PathBuf};

const SCHEMA_VERSION: u32 = 1;

#[derive(serde::Serialize, serde::Deserialize)]
struct SaveFile {
    schema_version: u32,
    rng_seed: u64,
    state: GameState,
}

#[derive(Default)]
pub struct MetaSave {
    pub runs_completed: u32,
    pub runs_won: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct MetaSaveFile {
    schema_version: u32,
    runs_completed: u32,
    runs_won: u32,
}

pub fn save_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("slay")
}

pub fn run_path() -> PathBuf { save_dir().join("run.ron") }
pub fn meta_path() -> PathBuf { save_dir().join("meta.ron") }

fn write_ron<T: serde::Serialize>(path: &Path, value: &T) {
    let ron = ron::ser::to_string_pretty(value, ron::ser::PrettyConfig::default())
        .expect("serialization failed");
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(path, ron);
}

pub fn save_run(state: &GameState, rng_seed: u64) {
    save_run_to(&run_path(), state, rng_seed);
}

pub fn save_run_to(path: &Path, state: &GameState, rng_seed: u64) {
    write_ron(path, &SaveFile { schema_version: SCHEMA_VERSION, rng_seed, state: state.clone() });
}

pub fn load_run() -> Option<(GameState, u64)> {
    load_run_from(&run_path())
}

pub fn load_run_from(path: &Path) -> Option<(GameState, u64)> {
    let text = std::fs::read_to_string(path).ok()?;
    let file: SaveFile = match ron::de::from_str(&text) {
        Ok(f) => f,
        Err(_) => { let _ = std::fs::remove_file(path); return None; }
    };
    if file.schema_version != SCHEMA_VERSION {
        let _ = std::fs::remove_file(path);
        return None;
    }
    Some((file.state, file.rng_seed))
}

pub fn delete_run() {
    let _ = std::fs::remove_file(run_path());
}

pub fn load_meta() -> MetaSave {
    load_meta_from(&meta_path())
}

pub fn load_meta_from(path: &Path) -> MetaSave {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => return MetaSave::default(),
    };
    let file: MetaSaveFile = match ron::de::from_str(&text) {
        Ok(f) => f,
        Err(_) => return MetaSave::default(),
    };
    if file.schema_version != SCHEMA_VERSION {
        return MetaSave::default();
    }
    MetaSave { runs_completed: file.runs_completed, runs_won: file.runs_won }
}

pub fn save_meta(meta: &MetaSave) {
    save_meta_to(&meta_path(), meta);
}

pub fn save_meta_to(path: &Path, meta: &MetaSave) {
    write_ron(path, &MetaSaveFile {
        schema_version: SCHEMA_VERSION,
        runs_completed: meta.runs_completed,
        runs_won: meta.runs_won,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use slay_core::new_simple_run;

    fn simple_state() -> GameState { new_simple_run() }

    #[test]
    fn game_state_round_trips_through_ron() {
        let original = simple_state();
        let ron = ron::ser::to_string_pretty(
            &SaveFile { schema_version: SCHEMA_VERSION, rng_seed: 0, state: original.clone() },
            ron::ser::PrettyConfig::default(),
        ).unwrap();
        let loaded: SaveFile = ron::de::from_str(&ron).unwrap();
        assert_eq!(loaded.state, original);
    }

    #[test]
    fn save_and_load_run_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("run.ron");
        let original = simple_state();
        save_run_to(&path, &original, 42);
        let (loaded, seed) = load_run_from(&path).expect("should load saved run");
        assert_eq!(loaded, original);
        assert_eq!(seed, 42);
    }

    #[test]
    fn load_run_returns_none_when_no_file_exists() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("run.ron");
        assert!(load_run_from(&path).is_none());
    }

    #[test]
    fn load_run_returns_none_and_deletes_corrupt_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("run.ron");
        std::fs::write(&path, "not valid ron").unwrap();
        assert!(load_run_from(&path).is_none());
        assert!(!path.exists());
    }

    #[test]
    fn load_run_returns_none_and_deletes_wrong_schema_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("run.ron");
        let stale = SaveFile { schema_version: 0, rng_seed: 0, state: simple_state() };
        write_ron(&path, &stale);
        assert!(load_run_from(&path).is_none());
        assert!(!path.exists());
    }

    #[test]
    fn meta_round_trips_through_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("meta.ron");
        let meta = MetaSave { runs_completed: 5, runs_won: 2 };
        save_meta_to(&path, &meta);
        let loaded = load_meta_from(&path);
        assert_eq!(loaded.runs_completed, 5);
        assert_eq!(loaded.runs_won, 2);
    }

    #[test]
    fn meta_returns_default_when_no_file_exists() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("meta.ron");
        let meta = load_meta_from(&path);
        assert_eq!(meta.runs_completed, 0);
        assert_eq!(meta.runs_won, 0);
    }
}

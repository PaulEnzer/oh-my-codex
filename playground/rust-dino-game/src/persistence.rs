use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

const HIGH_SCORE_FILE: &str = ".rust_dino_high_score";

#[derive(Debug, Clone)]
pub struct HighScoreStore {
    path: PathBuf,
}

impl HighScoreStore {
    pub fn from_env_or_default() -> Self {
        if let Ok(path) = env::var("DINO_HIGH_SCORE_PATH") {
            return Self::new(path);
        }

        let path = env::current_dir()
            .unwrap_or_else(|_| env::temp_dir())
            .join(HIGH_SCORE_FILE);

        Self { path }
    }

    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> u32 {
        fs::read_to_string(&self.path)
            .ok()
            .and_then(|contents| contents.trim().parse::<u32>().ok())
            .unwrap_or(0)
    }

    pub fn save(&self, score: u32) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&self.path, score.to_string())
    }

    pub fn store_if_higher(&self, previous_best: u32, candidate: u32) -> io::Result<u32> {
        let next_best = previous_best.max(candidate);
        if next_best > previous_best {
            self.save(next_best)?;
        }
        Ok(next_best)
    }
}

#[cfg(test)]
mod tests {
    use super::HighScoreStore;
    use std::{
        env, fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn unique_temp_file(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!("{name}-{nanos}.txt"))
    }

    #[test]
    fn load_defaults_to_zero_for_missing_file() {
        let path = unique_temp_file("missing-high-score");
        let store = HighScoreStore::new(&path);

        assert_eq!(store.load(), 0);
    }

    #[test]
    fn save_and_load_round_trip() {
        let path = unique_temp_file("round-trip-high-score");
        let store = HighScoreStore::new(&path);

        store.save(420).unwrap();

        assert_eq!(store.load(), 420);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn invalid_file_contents_fall_back_to_zero() {
        let path = unique_temp_file("invalid-high-score");
        fs::write(&path, "not-a-number").unwrap();
        let store = HighScoreStore::new(&path);

        assert_eq!(store.load(), 0);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn store_if_higher_only_overwrites_on_improvement() {
        let path = unique_temp_file("only-higher-high-score");
        let store = HighScoreStore::new(&path);
        store.save(300).unwrap();

        let best = store.store_if_higher(300, 250).unwrap();
        assert_eq!(best, 300);
        assert_eq!(store.load(), 300);

        let best = store.store_if_higher(300, 450).unwrap();
        assert_eq!(best, 450);
        assert_eq!(store.load(), 450);
        let _ = fs::remove_file(path);
    }
}

use crate::character::Character;
use crate::journal::JournalEntry;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct GameState {
    pub character: Character,
    pub journal: Vec<JournalEntry>,
    pub created_at: DateTime<Utc>,
    pub last_tick: DateTime<Utc>,
    /// Cached latest version from crates.io
    #[serde(default)]
    pub latest_version: Option<String>,
    /// When we last checked crates.io for a new version
    #[serde(default)]
    pub last_version_check: Option<DateTime<Utc>>,
    /// When the sage last appeared (to avoid spamming)
    #[serde(default)]
    pub last_sage_shown: Option<DateTime<Utc>>,
}

impl GameState {
    pub fn new(character: Character) -> Self {
        let now = Utc::now();
        GameState {
            character,
            journal: Vec::new(),
            created_at: now,
            last_tick: now,
            latest_version: None,
            last_version_check: None,
            last_sage_shown: None,
        }
    }

    pub fn add_journal(&mut self, entry: JournalEntry) {
        self.journal.push(entry);
        // Keep last 100 entries
        if self.journal.len() > 100 {
            self.journal.drain(0..self.journal.len() - 100);
        }
    }
}

pub fn save_dir() -> PathBuf {
    let mut path = dirs::home_dir().expect("Could not find home directory");
    path.push(".shellquest");
    path
}

pub fn save_path() -> PathBuf {
    save_dir().join("save.json")
}

pub fn save(state: &GameState) -> Result<(), String> {
    let dir = save_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create save dir: {}", e))?;

    // Set directory permissions to 0o700 (owner only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));
    }

    let json =
        serde_json::to_string_pretty(state).map_err(|e| format!("Failed to serialize: {}", e))?;

    // Atomic write: write to temp file then rename to prevent corruption from concurrent ticks
    let tmp_path = save_path().with_extension("json.tmp");
    let mut file =
        fs::File::create(&tmp_path).map_err(|e| format!("Failed to create temp file: {}", e))?;
    file.write_all(json.as_bytes())
        .map_err(|e| format!("Failed to write temp file: {}", e))?;
    file.sync_all()
        .map_err(|e| format!("Failed to sync temp file: {}", e))?;
    drop(file);
    fs::rename(&tmp_path, save_path()).map_err(|e| format!("Failed to rename save: {}", e))?;

    Ok(())
}

pub fn load() -> Result<GameState, String> {
    let path = save_path();
    if !path.exists() {
        return Err("No save file found. Run `sq init` to create a character.".to_string());
    }
    let data = fs::read_to_string(&path).map_err(|e| format!("Failed to read save: {}", e))?;
    serde_json::from_str(&data).map_err(|e| format!("Failed to parse save: {}", e))
}

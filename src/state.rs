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
    /// The last version we showed a first-time announcement for (so we only guarantee it once)
    #[serde(default)]
    pub last_announced_version: Option<String>,
    /// Cached shop items
    #[serde(default)]
    pub shop_items: Vec<crate::character::Item>,
    /// Date the shop was last refreshed (UTC midnight)
    #[serde(default)]
    pub shop_refreshed: Option<DateTime<Utc>>,
    #[serde(default)]
    pub quest_refreshed: Option<DateTime<Utc>>,
    #[serde(default)]
    pub quest_phrase: Option<String>,
    #[serde(default)]
    pub quest_scroll_path: Option<PathBuf>,
    #[serde(default)]
    pub quest_completed_today: bool,
    /// Last time the magical healer credited HP (UTC). Drives time-based passive heal.
    #[serde(default)]
    pub last_heal_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub active_boss: Option<crate::boss::Boss>,
    #[serde(default)]
    pub permadeath: bool,
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
            last_announced_version: None,
            shop_items: Vec::new(),
            shop_refreshed: None,
            quest_refreshed: None,
            quest_phrase: None,
            quest_scroll_path: None,
            quest_completed_today: false,
            last_heal_at: None,
            active_boss: None,
            permadeath: false,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::{Character, Class, Race};
    use crate::journal::{EventType, JournalEntry};

    fn make_character() -> Character {
        Character::new("Tester".to_string(), Class::Wizard, Race::Elf)
    }

    #[test]
    fn game_state_new_initializes_correctly() {
        let c = make_character();
        let state = GameState::new(c);
        assert!(state.journal.is_empty());
        assert!(state.latest_version.is_none());
        assert!(state.last_version_check.is_none());
        assert!(state.last_sage_shown.is_none());
        assert!(state.shop_items.is_empty());
        assert!(state.shop_refreshed.is_none());
        assert!(state.last_heal_at.is_none());
        assert_eq!(state.character.name, "Tester");
    }

    #[test]
    fn game_state_new_timestamps_near_now() {
        let before = chrono::Utc::now();
        let state = GameState::new(make_character());
        let after = chrono::Utc::now();
        assert!(state.created_at >= before);
        assert!(state.created_at <= after);
        assert!(state.last_tick >= before);
        assert!(state.last_tick <= after);
    }

    #[test]
    fn add_journal_appends_entry() {
        let mut state = GameState::new(make_character());
        let entry = JournalEntry::new(EventType::Combat, "A fight!".to_string());
        state.add_journal(entry);
        assert_eq!(state.journal.len(), 1);
        assert_eq!(state.journal[0].message, "A fight!");
    }

    #[test]
    fn add_journal_caps_at_100_entries() {
        let mut state = GameState::new(make_character());
        for i in 0..=110 {
            state.add_journal(JournalEntry::new(EventType::Travel, format!("entry {}", i)));
        }
        assert_eq!(state.journal.len(), 100);
        // The oldest entries were pruned; last entry should be the most recent
        assert_eq!(state.journal.last().unwrap().message, "entry 110");
    }

    #[test]
    fn save_dir_ends_with_shellquest() {
        let dir = save_dir();
        assert_eq!(dir.file_name().unwrap(), ".shellquest");
    }

    #[test]
    fn save_path_is_save_json_inside_save_dir() {
        let path = save_path();
        assert_eq!(path.file_name().unwrap(), "save.json");
        assert_eq!(path.parent().unwrap(), save_dir());
    }

    #[test]
    fn game_state_new_has_no_active_boss() {
        let state = GameState::new(make_character());
        assert!(state.active_boss.is_none());
    }

    #[test]
    fn game_state_new_has_no_active_quest() {
        let state = GameState::new(make_character());
        assert!(state.quest_refreshed.is_none());
        assert!(state.quest_phrase.is_none());
        assert!(state.quest_scroll_path.is_none());
        assert!(!state.quest_completed_today);
    }

    #[test]
    fn game_state_serializes_and_deserializes_boss() {
        use crate::boss::spawn_boss;
        let mut state = GameState::new(make_character());
        state.active_boss = Some(spawn_boss());
        let json = serde_json::to_string(&state).unwrap();
        let restored: GameState = serde_json::from_str(&json).unwrap();
        assert!(restored.active_boss.is_some());
    }

    #[test]
    fn game_state_serializes_and_deserializes_tournament_fields() {
        let mut state = GameState::new(make_character());
        state.character.tournament_wins = 7;
        state.character.best_tournament_round = 42;
        let json = serde_json::to_string(&state).unwrap();
        let restored: GameState = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.character.tournament_wins, 7);
        assert_eq!(restored.character.best_tournament_round, 42);
    }

    #[test]
    fn game_state_round_trips_quest_fields() {
        let mut state = GameState::new(make_character());
        let now = chrono::DateTime::parse_from_rfc3339("2026-05-30T12:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        state.quest_refreshed = Some(now);
        state.quest_phrase = Some("ashen root sigil".to_string());
        state.quest_scroll_path = Some(PathBuf::from("/tmp/sq-test/the_void/lost_scroll_0001.txt"));
        state.quest_completed_today = true;

        let json = serde_json::to_string(&state).unwrap();
        let restored: GameState = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.quest_refreshed, Some(now));
        assert_eq!(restored.quest_phrase.as_deref(), Some("ashen root sigil"));
        assert_eq!(
            restored.quest_scroll_path.as_deref(),
            Some(std::path::Path::new(
                "/tmp/sq-test/the_void/lost_scroll_0001.txt"
            ))
        );
        assert!(restored.quest_completed_today);
    }

    #[test]
    fn game_state_loads_old_save_without_quest_fields() {
        let state = GameState::new(make_character());
        let mut value = serde_json::to_value(&state).unwrap();
        let object = value.as_object_mut().unwrap();
        object.remove("quest_refreshed");
        object.remove("quest_phrase");
        object.remove("quest_scroll_path");
        object.remove("quest_completed_today");

        let restored: GameState = serde_json::from_value(value).unwrap();

        assert!(restored.quest_refreshed.is_none());
        assert!(restored.quest_phrase.is_none());
        assert!(restored.quest_scroll_path.is_none());
        assert!(!restored.quest_completed_today);
    }
}

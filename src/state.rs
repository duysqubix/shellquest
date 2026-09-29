use crate::character::Character;
use crate::journal::JournalEntry;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// How long `sq tick` waits for another sq process to finish with the save
/// before skipping this tick. It runs before every prompt, so it must never stall.
pub const TICK_LOCK_TIMEOUT: Duration = Duration::from_secs(1);
/// How long other read-modify-write commands wait for the save lock.
pub const COMMAND_LOCK_TIMEOUT: Duration = Duration::from_secs(10);

const SAVE_FILE: &str = "save.json";
const BACKUP_FILE: &str = "save.json.bak";
const LOCK_FILE: &str = "save.lock";
const NO_SAVE: &str = "No save file found. Run `sq init` to create a character.";
/// Prefix of temp and staging files (`.save.json.<pid>.<nanos>.<n>.tmp`).
const TEMP_PREFIX: &str = ".save.json.";
/// Temp files older than this were abandoned by a crashed process.
const STALE_TEMP_AGE: Duration = Duration::from_secs(60);

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
    /// Hook version the player was last told to upgrade from (see `hook::VERSION`).
    #[serde(default)]
    pub hook_notice_version: u32,
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
            hook_notice_version: 0,
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
    save_dir().join(SAVE_FILE)
}

/// Exclusive advisory lock on `~/.shellquest/save.lock` (`flock`-style, per open
/// file; on NFS it is only as good as the mount's lock support).
///
/// Hold it across a whole load → mutate → save so concurrent sq processes
/// (several shells ticking at once, a command racing a tick) can't lose each
/// other's updates. Never hold it across an interactive prompt or a network
/// call: every other shell's tick waits on it. Released on drop.
pub struct SaveLock {
    _file: fs::File,
}

#[derive(Debug)]
pub enum LockError {
    /// Another sq process held the lock for the whole timeout.
    Busy,
    /// The lock file or save directory can't be used (permissions, disk…).
    Io(String),
}

impl std::fmt::Display for LockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockError::Busy => {
                write!(
                    f,
                    "Another sq process is busy with your save; try again in a moment."
                )
            }
            LockError::Io(e) => write!(f, "{}", e),
        }
    }
}

pub fn lock(timeout: Duration) -> Result<SaveLock, LockError> {
    lock_in(&save_dir(), timeout)
}

fn lock_in(dir: &Path, timeout: Duration) -> Result<SaveLock, LockError> {
    ensure_dir(dir).map_err(LockError::Io)?;
    let path = dir.join(LOCK_FILE);
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options
        .open(&path)
        .map_err(|e| LockError::Io(format!("Failed to open {}: {}", path.display(), e)))?;
    // Best effort on purpose: the lock file is empty (no data to protect), and
    // failing here would stop the game on filesystems that reject chmod.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = file.set_permissions(fs::Permissions::from_mode(0o600));
    }
    let deadline = Instant::now() + timeout;
    loop {
        match file.try_lock() {
            Ok(()) => return Ok(SaveLock { _file: file }),
            Err(fs::TryLockError::WouldBlock) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(5))
            }
            Err(fs::TryLockError::WouldBlock) => return Err(LockError::Busy),
            Err(fs::TryLockError::Error(e)) => {
                return Err(LockError::Io(format!(
                    "Failed to lock {}: {}",
                    path.display(),
                    e
                )))
            }
        }
    }
}

/// Remove a file; a missing file is fine, any other failure is reported.
fn remove_if_exists(path: &Path) -> std::io::Result<()> {
    match fs::remove_file(path) {
        Err(e) if e.kind() != ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

/// Create the save directory (owner-only).
fn ensure_dir(dir: &Path) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| format!("Failed to create save dir: {}", e))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(dir, fs::Permissions::from_mode(0o700));
    }
    Ok(())
}

/// Save the character. The caller holds the save lock (see [`SaveLock`]); the
/// save being replaced is kept as the backup.
pub fn save(state: &GameState, _lock: &SaveLock) -> Result<(), String> {
    write_save(&save_dir(), state, true)
}

/// Save a brand-new character (`sq init`). The backup of any character this
/// replaces is removed first, so no later recovery can bring it back.
pub fn save_new(state: &GameState, _lock: &SaveLock) -> Result<(), String> {
    let dir = save_dir();
    remove_if_exists(&dir.join(BACKUP_FILE))
        .map_err(|e| format!("Failed to remove the old backup: {}", e))?;
    write_save(&dir, state, false)
}

/// Delete the character (`sq reset`, permadeath): the save and its backup.
/// Call it with the save lock held.
pub fn delete_save() -> std::io::Result<()> {
    delete_save_in(&save_dir())
}

fn delete_save_in(dir: &Path) -> std::io::Result<()> {
    // Delete the save even if the backup can't be removed; the next new character's
    // first save refuses to start while a stale backup survives.
    let backup = remove_if_exists(&dir.join(BACKUP_FILE));
    fs::remove_file(dir.join(SAVE_FILE))?;
    backup
}

fn write_save(dir: &Path, state: &GameState, keep_previous: bool) -> Result<(), String> {
    ensure_dir(dir)?;
    let json =
        serde_json::to_string_pretty(state).map_err(|e| format!("Failed to serialize: {}", e))?;

    // Atomic write: a temp file unique to this process, then rename over save.json.
    // (A shared temp name let concurrent processes truncate each other's writes.)
    reclaim_stale_temps(dir);
    let tmp = write_temp(dir, json.as_bytes())?;
    let current = dir.join(SAVE_FILE);
    if keep_previous && current.exists() {
        keep_backup(dir, &current);
    } else if !current.exists() {
        // A backup must belong to the current character (a fresh one has none).
        if let Err(e) = remove_if_exists(&dir.join(BACKUP_FILE)) {
            let _ = fs::remove_file(&tmp);
            return Err(format!("Failed to remove a stale backup: {}", e));
        }
    }
    fs::rename(&tmp, &current).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("Failed to rename save: {}", e)
    })
}

/// Write `bytes` to a new, uniquely named, owner-only file in `dir` and fsync it.
fn write_temp(dir: &Path, bytes: &[u8]) -> Result<PathBuf, String> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    for attempt in 0..16 {
        let path = dir.join(format!(
            "{}{}.{}.{}.tmp",
            TEMP_PREFIX,
            std::process::id(),
            stamp,
            attempt
        ));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = match options.open(&path) {
            Ok(file) => file,
            Err(e) if e.kind() == ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(format!("Failed to create temp file: {}", e)),
        };
        let written = file.write_all(bytes).and_then(|()| file.sync_all());
        drop(file);
        return match written {
            Ok(()) => Ok(path),
            Err(e) => {
                let _ = fs::remove_file(&path);
                Err(format!("Failed to write temp file: {}", e))
            }
        };
    }
    Err("Failed to create a unique temp file".to_string())
}

/// Remove temp/staging files a crashed sq left behind. Runs under the save lock;
/// the age threshold keeps it away from a write that is still in progress.
fn reclaim_stale_temps(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let Some(cutoff) = SystemTime::now().checked_sub(STALE_TEMP_AGE) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(TEMP_PREFIX) && name.ends_with(".tmp") {
            let modified = entry.metadata().and_then(|m| m.modified());
            if modified.is_ok_and(|t| t < cutoff) {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}

/// Keep the save being replaced as `save.json.bak`. A hard link means no data
/// copy, and the rename swaps the backup in atomically, so it is never half-written.
/// Best effort: without hard-link support the previous backup just stays.
fn keep_backup(dir: &Path, current: &Path) {
    let staging = dir.join(format!("{}bak.{}.tmp", TEMP_PREFIX, std::process::id()));
    let _ = fs::remove_file(&staging);
    if fs::hard_link(current, &staging).is_ok()
        && fs::rename(&staging, dir.join(BACKUP_FILE)).is_err()
    {
        let _ = fs::remove_file(&staging);
    }
}

enum SaveFile {
    Parsed(Box<GameState>),
    Unparseable { problem: String, bytes: Vec<u8> },
}

fn read_save(dir: &Path) -> Result<SaveFile, String> {
    match fs::read(dir.join(SAVE_FILE)) {
        Ok(bytes) => Ok(match serde_json::from_slice(&bytes) {
            Ok(state) => SaveFile::Parsed(Box::new(state)),
            Err(e) => SaveFile::Unparseable {
                problem: e.to_string(),
                bytes,
            },
        }),
        Err(e) if e.kind() == ErrorKind::NotFound => Err(NO_SAVE.to_string()),
        Err(e) => Err(format!("Failed to read save: {}", e)),
    }
}

/// Load for reading. A save that won't parse is repaired under the save lock,
/// so never call this while holding the lock (use [`load_locked`]).
pub fn load() -> Result<GameState, String> {
    let dir = save_dir();
    match read_save(&dir)? {
        SaveFile::Parsed(state) => Ok(*state),
        SaveFile::Unparseable { .. } => {
            let lock = lock_in(&dir, COMMAND_LOCK_TIMEOUT).map_err(|e| e.to_string())?;
            load_locked_in(&dir, &lock)
        }
    }
}

/// Load while holding the save lock (tick and read-modify-write commands).
pub fn load_locked(lock: &SaveLock) -> Result<GameState, String> {
    load_locked_in(&save_dir(), lock)
}

fn load_locked_in(dir: &Path, _lock: &SaveLock) -> Result<GameState, String> {
    match read_save(dir)? {
        SaveFile::Parsed(state) => Ok(*state),
        SaveFile::Unparseable { problem, bytes } => recover_from_backup(dir, &problem, &bytes),
    }
}

/// save.json exists but doesn't parse, and the caller holds the save lock: restore
/// the last good save, keep a copy of the damaged file, and tell the player once.
/// The replacement is written before anything else changes, so a crash at any
/// point leaves save.json and the backup in place and the next load retries.
fn recover_from_backup(dir: &Path, problem: &str, damaged: &[u8]) -> Result<GameState, String> {
    let failed = || Err(format!("Failed to parse save: {}", problem));
    let Ok(backup) = fs::read(dir.join(BACKUP_FILE)) else {
        return failed();
    };
    let Ok(state) = serde_json::from_slice::<GameState>(&backup) else {
        return failed();
    };
    let replacement = write_temp(dir, &backup)?;
    let kept = dir.join(format!(
        "save.json.corrupt-{}",
        Utc::now().format("%Y%m%dT%H%M%S%.3fZ")
    ));
    // Keep the damaged bytes (owner-only) before touching save.json; if that fails,
    // leave everything as it is and let a later load retry.
    let preserved = write_temp(dir, damaged).and_then(|copy| {
        fs::rename(&copy, &kept).map_err(|e| {
            let _ = fs::remove_file(&copy);
            e.to_string()
        })
    });
    if let Err(e) = preserved {
        let _ = fs::remove_file(&replacement);
        return Err(format!(
            "Failed to parse save: {} (not restoring from the backup: could not keep a copy of the damaged file: {})",
            problem, e
        ));
    }
    fs::rename(&replacement, dir.join(SAVE_FILE)).map_err(|e| {
        let _ = fs::remove_file(&replacement);
        format!("Failed to restore save from backup: {}", e)
    })?;
    eprintln!(
        "⚠️  sq: your save was unreadable ({}). Restored the last good copy; the damaged file is kept at {}",
        problem,
        kept.display()
    );
    Ok(state)
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

    // ── persistence: temp files, backup, recovery, locking (shellqeuest-x3p.1) ──
    // These use private *_in(dir) helpers on a scratch dir; they never touch $HOME.

    fn scratch_dir(name: &str) -> PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "sq-state-{}-{}-{}",
            name,
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn state_with_gold(gold: u32) -> GameState {
        let mut state = GameState::new(Character::new("Saver".into(), Class::Warrior, Race::Human));
        state.character.gold = gold;
        state
    }

    fn gold_in(path: &Path) -> u32 {
        let state: GameState = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        state.character.gold
    }

    fn load_in(dir: &Path) -> Result<GameState, String> {
        let lock = lock_in(dir, Duration::ZERO).unwrap();
        load_locked_in(dir, &lock)
    }

    fn names_in(dir: &Path) -> Vec<String> {
        fs::read_dir(dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect()
    }

    #[test]
    fn save_leaves_no_temp_files_behind() {
        let dir = scratch_dir("notemp");
        write_save(&dir, &state_with_gold(1), true).unwrap();
        write_save(&dir, &state_with_gold(2), true).unwrap();
        let leftovers: Vec<_> = names_in(&dir)
            .into_iter()
            .filter(|n| n.ends_with(".tmp"))
            .collect();
        assert!(
            leftovers.is_empty(),
            "temp files left behind: {:?}",
            leftovers
        );
        assert_eq!(gold_in(&dir.join(SAVE_FILE)), 2);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn backup_holds_the_previous_save() {
        let dir = scratch_dir("backup");
        write_save(&dir, &state_with_gold(10), true).unwrap();
        assert!(
            !dir.join(BACKUP_FILE).exists(),
            "a first save has nothing to back up"
        );
        write_save(&dir, &state_with_gold(20), true).unwrap();
        assert_eq!(gold_in(&dir.join(BACKUP_FILE)), 10);
        assert_eq!(gold_in(&dir.join(SAVE_FILE)), 20);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn corrupt_save_is_restored_from_backup_and_kept_aside() {
        let dir = scratch_dir("recover");
        write_save(&dir, &state_with_gold(10), true).unwrap();
        write_save(&dir, &state_with_gold(20), true).unwrap();
        fs::write(dir.join(SAVE_FILE), "{\"character\": trailing garbage").unwrap();

        let state = load_in(&dir).expect("recovers from the backup");
        assert_eq!(state.character.gold, 10);
        assert_eq!(
            gold_in(&dir.join(SAVE_FILE)),
            10,
            "save.json is repaired on disk"
        );
        assert_eq!(
            gold_in(&dir.join(BACKUP_FILE)),
            10,
            "the backup is still there"
        );
        let kept: Vec<_> = names_in(&dir)
            .into_iter()
            .filter(|n| n.starts_with("save.json.corrupt-"))
            .collect();
        assert_eq!(kept.len(), 1, "the damaged file is kept for inspection");
        assert_eq!(
            fs::read_to_string(dir.join(&kept[0])).unwrap(),
            "{\"character\": trailing garbage"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn invalid_utf8_save_still_reaches_recovery() {
        let dir = scratch_dir("utf8");
        write_save(&dir, &state_with_gold(7), true).unwrap();
        write_save(&dir, &state_with_gold(8), true).unwrap();
        fs::write(dir.join(SAVE_FILE), [0xff, 0xfe, 0x00, 0x7b]).unwrap();
        assert_eq!(load_in(&dir).unwrap().character.gold, 7);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn corrupt_save_without_backup_reports_a_parse_error() {
        let dir = scratch_dir("nobackup");
        fs::write(dir.join(SAVE_FILE), "not json").unwrap();
        let err = load_in(&dir).unwrap_err();
        assert!(err.starts_with("Failed to parse save"), "{}", err);
        assert_eq!(
            fs::read_to_string(dir.join(SAVE_FILE)).unwrap(),
            "not json",
            "left untouched"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_save_reports_no_save() {
        let dir = scratch_dir("missing");
        assert_eq!(load_in(&dir).unwrap_err(), NO_SAVE);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_lock_is_exclusive_until_dropped() {
        let dir = scratch_dir("lock");
        let held = lock_in(&dir, Duration::ZERO).unwrap();
        assert!(matches!(
            lock_in(&dir, Duration::from_millis(30)),
            Err(LockError::Busy)
        ));
        drop(held);
        assert!(lock_in(&dir, Duration::ZERO).is_ok());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn delete_removes_save_and_backup() {
        let dir = scratch_dir("delete");
        write_save(&dir, &state_with_gold(1), true).unwrap();
        write_save(&dir, &state_with_gold(2), true).unwrap();
        delete_save_in(&dir).unwrap();
        assert!(!dir.join(SAVE_FILE).exists());
        assert!(
            !dir.join(BACKUP_FILE).exists(),
            "a backup would resurrect a dead character"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn replacing_a_character_never_backs_up_the_old_one() {
        // What save_new does: drop the backup, then write without keeping the previous save.
        let dir = scratch_dir("new");
        write_save(&dir, &state_with_gold(1), true).unwrap();
        write_save(&dir, &state_with_gold(2), true).unwrap();
        let _ = fs::remove_file(dir.join(BACKUP_FILE));
        write_save(&dir, &state_with_gold(99), false).unwrap();
        assert!(!dir.join(BACKUP_FILE).exists());
        assert_eq!(gold_in(&dir.join(SAVE_FILE)), 99);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn first_save_after_a_delete_drops_a_stale_backup() {
        let dir = scratch_dir("stale");
        write_save(&dir, &state_with_gold(1), true).unwrap();
        write_save(&dir, &state_with_gold(2), true).unwrap();
        fs::remove_file(dir.join(SAVE_FILE)).unwrap(); // e.g. an older sq deleted only save.json
        write_save(&dir, &state_with_gold(3), true).unwrap();
        assert!(!dir.join(BACKUP_FILE).exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn delete_still_removes_the_save_when_the_backup_is_already_gone() {
        let dir = scratch_dir("delete-nobak");
        write_save(&dir, &state_with_gold(1), true).unwrap();
        assert!(!dir.join(BACKUP_FILE).exists());
        delete_save_in(&dir).unwrap();
        assert!(!dir.join(SAVE_FILE).exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn abandoned_temp_files_are_reclaimed_but_fresh_ones_are_not() {
        let dir = scratch_dir("reclaim");
        let old = dir.join(format!("{}999.1.0.tmp", TEMP_PREFIX));
        let fresh = dir.join(format!("{}999.2.0.tmp", TEMP_PREFIX));
        fs::write(&old, "left by a crash").unwrap();
        fs::write(&fresh, "being written right now").unwrap();
        let long_ago = SystemTime::now() - Duration::from_secs(3600);
        fs::File::options()
            .write(true)
            .open(&old)
            .unwrap()
            .set_modified(long_ago)
            .unwrap();
        write_save(&dir, &state_with_gold(1), true).unwrap();
        assert!(!old.exists(), "abandoned temp file reclaimed");
        assert!(
            fresh.exists(),
            "a temp file that may still be in use is left alone"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}

use rand::Rng;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const VOID_DIR_NAME: &str = "the_void";
const MAX_DEPTH: usize = 7;
const MIN_DIRS_PER_LEVEL: usize = 2;
const MAX_DIRS_PER_LEVEL: usize = 5;
const HIDDEN_FILE_PREFIX: &str = "lost_scroll";

const ROOM_NAMES: &[&str] = &[
    "ashen_gate",
    "null_gallery",
    "broken_prompt",
    "hollow_inode",
    "echo_vault",
    "shard_cache",
    "silent_pipe",
    "forgotten_tty",
    "black_stdin",
    "orphaned_lock",
];

const LEAF_MARKERS: &[&str] = &[
    "The prompt flickers, then forgets your name.",
    "Dust gathers in the shape of old commands.",
    "A directory breathes once and falls silent.",
    "Something here is listening to stderr.",
    "The walls are carved with paths that never existed.",
];

pub fn void_root() -> PathBuf {
    void_root_in(&crate::state::save_dir())
}

pub fn generate_void(rng: &mut impl Rng) -> io::Result<PathBuf> {
    generate_void_in(&crate::state::save_dir(), rng)
}

pub fn clear_void() -> io::Result<()> {
    clear_void_in(&crate::state::save_dir())
}

pub fn hide_file_in_void(root: &Path, contents: &str, rng: &mut impl Rng) -> io::Result<PathBuf> {
    let canonical_root = root.canonicalize()?;
    let mut leaves = leaf_dirs(root)?;
    let deepest = leaves
        .iter()
        .map(|(_, depth)| *depth)
        .max()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Void has no rooms"))?;
    leaves.retain(|(_, depth)| *depth == deepest);

    let leaf = &leaves[rng.gen_range(0..leaves.len())].0;
    for _ in 0..32 {
        let path = leaf.join(format!(
            "{}_{:04}.txt",
            HIDDEN_FILE_PREFIX,
            rng.gen_range(0..10_000)
        ));

        if path.exists() {
            continue;
        }

        fs::write(&path, contents)?;
        ensure_path_contained(&canonical_root, &path)?;
        return Ok(path);
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not find an empty Void leaf filename",
    ))
}

fn void_root_in(save_dir: &Path) -> PathBuf {
    save_dir.join(VOID_DIR_NAME)
}

fn generate_void_in(save_dir: &Path, rng: &mut impl Rng) -> io::Result<PathBuf> {
    clear_void_in(save_dir)?;

    let root = void_root_in(save_dir);
    fs::create_dir_all(&root)?;
    eprintln!("🕳️  The Void reshapes beneath {}...", root.display());

    let mut levels = vec![vec![root.clone()]];
    for depth in 1..=MAX_DEPTH {
        let parents = levels[depth - 1].clone();
        let room_count = rng.gen_range(MIN_DIRS_PER_LEVEL..=MAX_DIRS_PER_LEVEL);
        let mut rooms = Vec::with_capacity(room_count);

        for index in 0..room_count {
            let parent = &parents[rng.gen_range(0..parents.len())];
            let room = parent.join(room_name(depth, index, rng));
            fs::create_dir(&room)?;
            rooms.push(room);
        }

        levels.push(rooms);
    }

    write_leaf_markers(levels.last().unwrap(), rng)?;
    create_rifts(&levels, rng)?;
    ensure_void_contained(&root)?;

    Ok(root)
}

fn clear_void_in(save_dir: &Path) -> io::Result<()> {
    let root = void_root_in(save_dir);
    ensure_safe_void_root(&root)?;

    match fs::symlink_metadata(&root) {
        Ok(metadata) if metadata.file_type().is_symlink() || metadata.is_file() => {
            fs::remove_file(&root)
        }
        Ok(_) => fs::remove_dir_all(&root),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

fn room_name(depth: usize, index: usize, rng: &mut impl Rng) -> String {
    let name = ROOM_NAMES[rng.gen_range(0..ROOM_NAMES.len())];
    format!("depth_{:02}_room_{:02}_{}", depth, index, name)
}

fn write_leaf_markers(leaves: &[PathBuf], rng: &mut impl Rng) -> io::Result<()> {
    for leaf in leaves {
        let marker = LEAF_MARKERS[rng.gen_range(0..LEAF_MARKERS.len())];
        fs::write(leaf.join(".void_whisper"), format!("{}\n", marker))?;
    }

    Ok(())
}

/// Symlink acyclicity invariant: every rift lives in a room at depth N and points only
/// to a real room at depth greater than N. Real directory edges also move from a
/// parent to a deeper child, so any traversal that follows directories and rifts has
/// monotonically increasing depth and cannot loop back to an earlier room.
fn create_rifts(levels: &[Vec<PathBuf>], rng: &mut impl Rng) -> io::Result<()> {
    let mut created = 0;

    for source_depth in 0..MAX_DEPTH {
        let target_depth = rng.gen_range(source_depth + 1..=MAX_DEPTH);
        let targets = &levels[target_depth];

        for (source_index, source) in levels[source_depth].iter().enumerate() {
            if created > 0 && !rng.gen_ratio(1, 2) {
                continue;
            }

            let target = &targets[rng.gen_range(0..targets.len())];
            let link = source.join(format!(
                "rift_to_depth_{:02}_{}",
                target_depth, source_index
            ));
            symlink_dir(target, &link)?;
            created += 1;
        }
    }

    Ok(())
}

fn leaf_dirs(root: &Path) -> io::Result<Vec<(PathBuf, usize)>> {
    let mut leaves = Vec::new();
    let mut stack = vec![(root.to_path_buf(), 0)];

    while let Some((dir, depth)) = stack.pop() {
        let mut child_dirs = Vec::new();
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                child_dirs.push(entry.path());
            }
        }

        if child_dirs.is_empty() {
            leaves.push((dir, depth));
        } else {
            for child in child_dirs {
                stack.push((child, depth + 1));
            }
        }
    }

    Ok(leaves)
}

fn ensure_void_contained(root: &Path) -> io::Result<()> {
    let canonical_root = root.canonicalize()?;
    ensure_path_contained(&canonical_root, &canonical_root)?;

    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_type = entry.file_type()?;

            ensure_path_contained(&canonical_root, &path)?;
            if file_type.is_dir() {
                stack.push(path);
            }
        }
    }

    Ok(())
}

fn ensure_path_contained(canonical_root: &Path, path: &Path) -> io::Result<()> {
    let canonical_path = path.canonicalize()?;
    if canonical_path.starts_with(canonical_root) {
        return Ok(());
    }

    Err(io::Error::new(
        io::ErrorKind::PermissionDenied,
        format!(
            "Void path escaped root: {} is outside {}",
            canonical_path.display(),
            canonical_root.display()
        ),
    ))
}

fn ensure_safe_void_root(root: &Path) -> io::Result<()> {
    if root.as_os_str().is_empty()
        || root == Path::new("/")
        || root.parent().is_none()
        || root.file_name().and_then(|name| name.to_str()) != Some(VOID_DIR_NAME)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "refusing to clear unsafe Void root",
        ));
    }

    Ok(())
}

fn symlink_dir(target: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static NEXT_TEMP_ID: AtomicUsize = AtomicUsize::new(0);

    struct TempSaveRoot {
        path: PathBuf,
    }

    impl TempSaveRoot {
        fn new() -> Self {
            let unique = NEXT_TEMP_ID.fetch_add(1, Ordering::SeqCst);
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "shellquest_void_test_{}_{}_{}",
                std::process::id(),
                nanos,
                unique
            ));
            fs::create_dir_all(&path).unwrap();

            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempSaveRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn seeded_rng() -> StdRng {
        StdRng::seed_from_u64(0x51e11)
    }

    fn real_dir_depths(root: &Path) -> io::Result<Vec<(PathBuf, usize)>> {
        let mut dirs = Vec::new();
        let mut stack = vec![(root.to_path_buf(), 0)];

        while let Some((dir, depth)) = stack.pop() {
            dirs.push((dir.clone(), depth));
            for entry in fs::read_dir(&dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    stack.push((entry.path(), depth + 1));
                }
            }
        }

        Ok(dirs)
    }

    fn symlink_paths(root: &Path) -> io::Result<Vec<PathBuf>> {
        let mut links = Vec::new();
        let mut stack = vec![root.to_path_buf()];

        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(&dir)? {
                let entry = entry?;
                let file_type = entry.file_type()?;
                if file_type.is_symlink() {
                    links.push(entry.path());
                } else if file_type.is_dir() {
                    stack.push(entry.path());
                }
            }
        }

        Ok(links)
    }

    fn assert_path_under_root(root: &Path, path: &Path) {
        let canonical_root = root.canonicalize().unwrap();
        let canonical_path = path.canonicalize().unwrap();
        assert!(
            canonical_path.starts_with(&canonical_root),
            "{} escaped {}",
            canonical_path.display(),
            canonical_root.display()
        );
    }

    #[test]
    fn generate_creates_bounded_void() {
        let save_root = TempSaveRoot::new();
        let mut rng = seeded_rng();

        let root = generate_void_in(save_root.path(), &mut rng).unwrap();

        assert!(root.exists());
        let dirs = real_dir_depths(&root).unwrap();
        let max_depth = dirs.iter().map(|(_, depth)| *depth).max().unwrap();
        assert!(max_depth <= MAX_DEPTH);

        let mut dirs_per_level: HashMap<usize, usize> = HashMap::new();
        for (_, depth) in dirs {
            *dirs_per_level.entry(depth).or_insert(0) += 1;
        }

        for (depth, count) in dirs_per_level {
            let cap = if depth == 0 { 1 } else { MAX_DIRS_PER_LEVEL };
            assert!(count <= cap, "depth {} had {} dirs", depth, count);
        }
    }

    #[test]
    fn symlink_targets_stay_inside_void_root() {
        let save_root = TempSaveRoot::new();
        let mut rng = seeded_rng();

        let root = generate_void_in(save_root.path(), &mut rng).unwrap();

        for (dir, _) in real_dir_depths(&root).unwrap() {
            assert_path_under_root(&root, &dir);
        }

        let links = symlink_paths(&root).unwrap();
        assert!(
            !links.is_empty(),
            "Void should include at least one symlink rift"
        );
        for link in links {
            assert_path_under_root(&root, &link);
        }
    }

    #[test]
    fn symlink_traversal_terminates() {
        let save_root = TempSaveRoot::new();
        let mut rng = seeded_rng();

        let root = generate_void_in(save_root.path(), &mut rng).unwrap();
        assert!(!symlink_paths(&root).unwrap().is_empty());

        let mut stack = vec![root.clone()];
        let mut visited_entries = 0;

        while let Some(path) = stack.pop() {
            visited_entries += 1;
            assert!(visited_entries < 10_000, "traversal did not terminate");

            let metadata = fs::symlink_metadata(&path).unwrap();
            if metadata.is_file() {
                continue;
            }

            let read_dir = if metadata.file_type().is_symlink() {
                path.canonicalize().unwrap()
            } else {
                path
            };

            for entry in fs::read_dir(read_dir).unwrap() {
                stack.push(entry.unwrap().path());
            }
        }
    }

    #[test]
    fn clear_void_removes_only_the_void_and_allows_absent_root() {
        let save_root = TempSaveRoot::new();
        let root = void_root_in(save_root.path());
        let sibling = save_root.path().join("outside_void.txt");
        fs::create_dir_all(root.join("nested")).unwrap();
        fs::write(root.join("nested").join("room.txt"), "lost").unwrap();
        fs::write(&sibling, "keep").unwrap();

        clear_void_in(save_root.path()).unwrap();

        assert!(!root.exists());
        assert!(sibling.exists());
        clear_void_in(save_root.path()).unwrap();
        assert!(sibling.exists());
    }

    #[test]
    fn hide_file_in_void_places_readable_file_at_leaf() {
        let save_root = TempSaveRoot::new();
        let mut rng = seeded_rng();
        let root = generate_void_in(save_root.path(), &mut rng).unwrap();

        let hidden = hide_file_in_void(&root, "the scroll remembers", &mut rng).unwrap();

        assert_path_under_root(&root, &hidden);
        assert_eq!(fs::read_to_string(&hidden).unwrap(), "the scroll remembers");
        let parent = hidden.parent().unwrap();
        let depth = parent.strip_prefix(&root).unwrap().components().count();
        assert!(depth >= MAX_DEPTH.saturating_sub(1));
        assert!(!fs::read_dir(parent).unwrap().any(|entry| entry
            .unwrap()
            .file_type()
            .unwrap()
            .is_dir()));
    }

    #[test]
    #[ignore]
    fn manual_void_harness_generates_temp_maze() {
        let save_root = std::env::var("SQ_VOID_MANUAL_SAVE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir().join("shellquest_void_manual_save"));
        let _ = fs::remove_dir_all(&save_root);
        fs::create_dir_all(&save_root).unwrap();

        let mut rng = seeded_rng();
        let root = generate_void_in(&save_root, &mut rng).unwrap();

        println!("VOID_ROOT={}", root.display());
    }
}

#[cfg(test)]
mod reshuffle_tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static NEXT_RESHUFFLE_ID: AtomicUsize = AtomicUsize::new(0);

    struct TempSaveRoot {
        path: PathBuf,
    }

    impl TempSaveRoot {
        fn new() -> Self {
            let unique = NEXT_RESHUFFLE_ID.fetch_add(1, Ordering::SeqCst);
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "shellquest_reshuffle_test_{}_{}_{}",
                std::process::id(),
                nanos,
                unique
            ));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempSaveRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn seeded_rng() -> StdRng {
        StdRng::seed_from_u64(0xDA117EED)
    }

    #[test]
    fn double_generate_leaves_exactly_one_void_dir_no_orphans() {
        let save_root = TempSaveRoot::new();
        let mut rng = seeded_rng();

        // First daily reshuffle
        let root1 = generate_void_in(save_root.path(), &mut rng).unwrap();
        assert!(root1.exists(), "first void must exist after generation");

        // Second daily reshuffle (simulates UTC-midnight rollover)
        let root2 = generate_void_in(save_root.path(), &mut rng).unwrap();
        assert!(root2.exists(), "second void must exist after regeneration");

        // Both calls return the same canonical path (the_void under save_root)
        assert_eq!(
            root1, root2,
            "both reshuffles must produce the same root path"
        );

        // Exactly one the_void directory exists — no orphan siblings
        let void_siblings: Vec<_> = fs::read_dir(save_root.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with(VOID_DIR_NAME)
            })
            .collect();
        assert_eq!(
            void_siblings.len(),
            1,
            "expected exactly 1 the_void dir after two reshuffles, found {}: {:?}",
            void_siblings.len(),
            void_siblings.iter().map(|e| e.path()).collect::<Vec<_>>()
        );
    }
}

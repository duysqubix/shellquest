mod arena;
mod boss;
mod character;
mod display;
mod events;
mod help;
mod hook;
mod journal;
mod loot;
mod messages;
mod sage;
mod state;
mod telemetry;
pub mod void;
mod zones;

use character::{Class, Race};
use clap::{Parser, Subcommand};
use colored::*;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "sq",
    version,
    about = "A passive RPG that lives in your terminal",
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// View the in-game manual; pass an optional topic for a specific command guide
    Help {
        /// Topic name (e.g. arena, status, journal); omit for the index
        topic: Option<String>,
    },
    /// Create a new character
    Init,
    /// View your character sheet (includes inventory)
    #[clap(visible_alias = "stat")]
    Status,
    /// Check your inventory
    #[clap(visible_alias = "inv")]
    Inventory,
    /// View your adventure journal
    Journal,
    /// Print the full static item catalog
    Items {
        /// Emit JSON output
        #[arg(long)]
        json: bool,
    },
    /// Print the static boss and monster bestiary
    Bestiary {
        /// Emit JSON output
        #[arg(long)]
        json: bool,
    },
    /// Process a terminal command (called by shell hook)
    Tick {
        /// The command that was run
        #[arg(long, default_value = "")]
        cmd: String,
        /// Current working directory
        #[arg(long, default_value = ".")]
        cwd: String,
        /// Exit code of the command
        #[arg(long, default_value_t = 0)]
        exit_code: i32,
        /// Force the update sage to appear (for testing)
        #[arg(long, hide = true)]
        test_sage: bool,
        /// Version of the shell hook calling (`hook::VERSION`); older hooks pass none
        #[arg(long, hide = true, default_value_t = 0)]
        hook: u32,
    },
    /// Print or install the shell hook
    Hook {
        /// Shell type: bash, zsh, or fish
        #[arg(long, default_value = "zsh")]
        shell: String,
        /// Install hook directly to a file (default: ~/.zshrc, ~/.bashrc, or fish config)
        #[arg(long)]
        install: bool,
        /// Custom file to install the hook to (implies --install)
        #[arg(long)]
        file: Option<String>,
    },
    /// Equip armor or ring from inventory
    #[clap(visible_alias = "wear")]
    Equip {
        /// Item name (or partial match)
        name: Vec<String>,
    },
    /// Wield a weapon from inventory
    Wield {
        /// Item name (or partial match)
        name: Vec<String>,
    },
    /// Remove equipped weapon, armor, or ring (puts it back in inventory)
    #[clap(visible_alias = "unequip")]
    Remove {
        /// Item name, partial match, or slot keyword (weapon/armor/ring)
        name: Vec<String>,
    },
    /// Drop an item from inventory permanently
    Drop {
        /// Item name (or partial match)
        name: Vec<String>,
    },
    /// Browse the shop (must be in home directory)
    Shop,
    /// Meet the daily quest-giver in your home directory
    Quest {
        #[command(subcommand)]
        action: Option<QuestAction>,
    },
    /// Buy an item from the shop by number (see `sq shop` for numbered list)
    Buy {
        /// Item number from the shop list
        number: usize,
    },
    /// Sell an inventory item at the shop by number or name; `sq sell junk` sweeps all Common/Uncommon items
    Sell {
        /// Item number (1-indexed), partial name match, or the literal word `junk`
        item: Vec<String>,
    },
    /// Enchant an equipped item (+1 power, max +5). Wizard class can enchant anywhere; others must be in $HOME.
    Enchant {
        /// Equipped item name (partial match)
        name: Vec<String>,
    },
    /// Show full stats and modifiers for an inventory or equipped item
    #[clap(visible_alias = "id")]
    Identify {
        /// Item name (or partial match); append `.N` to pick the N-th match
        name: Vec<String>,
    },
    /// Drink a potion from inventory to restore HP
    Drink {
        /// Item name (or partial match)
        name: Vec<String>,
    },
    /// Flee from the active world boss (costs 10% of your gold)
    Flee,
    /// Prestige: reset to level 1 with a subclass and bonus stats
    Prestige,
    /// Reset your character (start over)
    Reset,
    /// Update sq to the latest version
    Update,
    /// Enter the Arena (interactive combat gauntlet)
    Arena,
    /// Enter the Terminal Gauntlet tournament (deprecated; use `arena`)
    Tournament,
}

#[derive(Subcommand)]
enum QuestAction {
    /// Speak the hidden scroll phrase back to the quest-giver
    Answer {
        /// Phrase found in the Void scroll
        phrase: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Help { topic } => cmd_help(topic.as_deref()),
        Commands::Init => cmd_init(),
        Commands::Status => cmd_status(),
        Commands::Inventory => cmd_inventory(),
        Commands::Journal => cmd_journal(),
        Commands::Items { json } => cmd_items(json),
        Commands::Bestiary { json } => cmd_bestiary(json),
        Commands::Tick {
            cmd,
            cwd,
            exit_code,
            test_sage,
            hook,
        } => cmd_tick(&cmd, &cwd, exit_code, test_sage, hook),
        Commands::Hook {
            shell,
            install,
            file,
        } => cmd_hook(&shell, install || file.is_some(), file),
        Commands::Shop => cmd_shop(),
        Commands::Quest { action } => cmd_quest(action),
        Commands::Buy { number } => cmd_buy(number),
        Commands::Sell { item } => cmd_sell(&item.join(" ")),
        Commands::Enchant { name } => cmd_enchant(&name.join(" ")),
        Commands::Identify { name } => cmd_identify(&name.join(" ")),
        Commands::Equip { name } => cmd_equip(&name.join(" ")),
        Commands::Wield { name } => cmd_wield(&name.join(" ")),
        Commands::Remove { name } => cmd_remove(&name.join(" ")),
        Commands::Drop { name } => cmd_drop_item(&name.join(" ")),
        Commands::Drink { name } => cmd_drink(&name.join(" ")),
        Commands::Flee => cmd_flee(),
        Commands::Prestige => cmd_prestige(),
        Commands::Reset => cmd_reset(),
        Commands::Update => cmd_update(),
        Commands::Arena => cmd_arena(false),
        Commands::Tournament => cmd_arena(true),
    }
}

fn prompt(msg: &str) -> String {
    print!("{}", msg);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn cmd_init() {
    let replacing = state::save_path().exists();
    if replacing {
        let answer = prompt(&format!(
            "{} A character already exists! Overwrite? [y/N] ",
            "⚠️".yellow()
        ));
        if answer.to_lowercase() != "y" {
            println!("{}", "Cancelled.".dimmed());
            return;
        }
    }

    println!();
    println!(
        "{}",
        "⚔️  Welcome to sq — The Passive Terminal RPG ⚔️"
            .bold()
            .cyan()
    );
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!();

    // Name
    let name = loop {
        let n = prompt(&format!("{} What is your name, adventurer? ", "📝".bold()));
        if !n.is_empty() {
            break n;
        }
        println!("{}", "  Please enter a name.".red());
    };

    println!();

    // Class
    println!("{}", "Choose your class:".bold().yellow());
    println!(
        "  {} {} — High INT, arcane power",
        "1.".dimmed(),
        "Wizard".blue().bold()
    );
    println!(
        "  {} {} — High STR, melee combat",
        "2.".dimmed(),
        "Warrior".red().bold()
    );
    println!(
        "  {} {} — High DEX, critical strikes",
        "3.".dimmed(),
        "Rogue".green().bold()
    );
    println!(
        "  {} {} — Balanced DEX/STR, versatile",
        "4.".dimmed(),
        "Ranger".yellow().bold()
    );
    println!(
        "  {} {} — Highest INT, dark arts",
        "5.".dimmed(),
        "Necromancer".magenta().bold()
    );

    let class = loop {
        let c = prompt(&format!("{} Choose [1-5]: ", "🎭".bold()));
        match c.as_str() {
            "1" => break Class::Wizard,
            "2" => break Class::Warrior,
            "3" => break Class::Rogue,
            "4" => break Class::Ranger,
            "5" => break Class::Necromancer,
            _ => println!("{}", "  Pick 1-5.".red()),
        }
    };

    println!();

    // Race
    println!("{}", "Choose your race:".bold().yellow());
    println!(
        "  {} {} — Balanced stats (+1/+1/+1)",
        "1.".dimmed(),
        "Human".white().bold()
    );
    println!(
        "  {} {} — Agile & wise (+0/+2/+2)",
        "2.".dimmed(),
        "Elf".cyan().bold()
    );
    println!(
        "  {} {} — Tough & sturdy (+3/+0/+1)",
        "3.".dimmed(),
        "Dwarf".yellow().bold()
    );
    println!(
        "  {} {} — Raw strength (+4/+1/-1)",
        "4.".dimmed(),
        "Orc".red().bold()
    );
    println!(
        "  {} {} — Quick & clever (-1/+3/+1)",
        "5.".dimmed(),
        "Goblin".green().bold()
    );

    let race = loop {
        let r = prompt(&format!("{} Choose [1-5]: ", "🧬".bold()));
        match r.as_str() {
            "1" => break Race::Human,
            "2" => break Race::Elf,
            "3" => break Race::Dwarf,
            "4" => break Race::Orc,
            "5" => break Race::Goblin,
            _ => println!("{}", "  Pick 1-5.".red()),
        }
    };

    let character = character::Character::new(name.clone(), class, race);
    let mut game_state = state::GameState::new(character);

    println!();
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".red().bold());
    println!("{} {}", "💀".bold(), "PERMADEATH MODE".red().bold());
    println!("  If you die, your character is gone forever. All is lost.");
    println!("  In standard mode, death resets your XP to 0 and costs 15% gold.");
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".red().bold());
    let pd_answer = prompt("Enable permadeath? [y/N] ");
    game_state.permadeath = pd_answer.trim().to_lowercase() == "y";

    if game_state.permadeath {
        println!(
            "{} {}",
            "☠".red().bold(),
            "Permadeath enabled. May the void be merciful."
                .red()
                .dimmed()
        );
    } else {
        println!(
            "{} {}",
            "✓".green(),
            "Standard mode. Death is a setback, not the end.".dimmed()
        );
    }

    let saved = state::lock(state::COMMAND_LOCK_TIMEOUT)
        .map_err(|e| e.to_string())
        .and_then(|lock| {
            if !replacing && state::save_path().exists() {
                // Created in another terminal while this one was prompting.
                return Err("A character appeared while you were choosing; not overwriting it. Run `sq init` again to replace it.".to_string());
            }
            state::save_new(&game_state, &lock)
        });
    match saved {
        Ok(()) => {
            println!();
            println!(
                "{}",
                "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
            );
            println!(
                "{} {} has entered the terminal realm!",
                "🎉".bold(),
                name.bold().green()
            );
            println!();
            println!(
                "  Run {} to install the shell hook.",
                "sq hook --shell zsh --install".cyan()
            );
            println!("  Run {} to see your character.", "sq status".cyan());
            println!(
                "{}",
                "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
            );
            println!();
        }
        Err(e) => {
            eprintln!("{} Failed to save: {}", "❌".bold(), e.red());
        }
    }
}

fn cmd_status() {
    match state::load() {
        Ok(game) => {
            display::print_status(&game.character, game.permadeath);
            display::print_inventory(&game.character);
        }
        Err(e) => eprintln!("{} {}", "❌".bold(), e.red()),
    }
}

fn cmd_inventory() {
    match state::load() {
        Ok(game) => display::print_inventory(&game.character),
        Err(e) => eprintln!("{} {}", "❌".bold(), e.red()),
    }
}

fn cmd_journal() {
    match state::load() {
        Ok(game) => display::print_journal(&game.journal),
        Err(e) => eprintln!("{} {}", "❌".bold(), e.red()),
    }
}

fn cmd_items(_json: bool) {
    let weights = loot::rarity_weights();
    let rarity_weights: serde_json::Map<String, serde_json::Value> = weights
        .iter()
        .map(|(rarity, weight)| (rarity.to_string(), serde_json::json!(*weight)))
        .collect();
    let rarity_multipliers: serde_json::Map<String, serde_json::Value> = weights
        .iter()
        .map(|(rarity, _)| {
            (
                rarity.to_string(),
                serde_json::json!(loot::rarity_multiplier(*rarity)),
            )
        })
        .collect();

    let value = serde_json::json!({
        "items": loot::catalog(),
        "rarity_weights": rarity_weights,
        "rarity_multipliers": rarity_multipliers,
    });

    match serde_json::to_string_pretty(&value) {
        Ok(json) => println!("{json}"),
        Err(e) => {
            eprintln!("{} Failed to serialize item catalog: {}", "❌".bold(), e);
            std::process::exit(1);
        }
    }
}

fn cmd_bestiary(_json: bool) {
    let tier_danger: serde_json::Map<String, serde_json::Value> = events::tier_danger()
        .into_iter()
        .map(|(danger, tiers)| (danger.to_string(), serde_json::json!(tiers)))
        .collect();

    let value = serde_json::json!({
        "bosses": boss::boss_roster(),
        "monsters": events::monster_bestiary(),
        "meta": {
            "tier_danger": tier_danger,
            "tier_order": events::monster_tier_order(),
            "elite": events::elite_modifiers(),
            "boss_spawn_rate": boss::BOSS_SPAWN_RATE,
            "boss_min_level": boss::BOSS_MIN_LEVEL,
            "boss_scale_offset": boss::BOSS_SCALE_OFFSET,
        },
    });

    match serde_json::to_string_pretty(&value) {
        Ok(json) => println!("{json}"),
        Err(e) => {
            eprintln!("{} Failed to serialize bestiary: {}", "❌".bold(), e);
            std::process::exit(1);
        }
    }
}

fn format_help(topic: Option<&str>) -> String {
    match topic {
        None => help::render_index(),
        Some(name) => match help::lookup_topic(name) {
            help::LookupResult::Found(t) => help::render_topic(t),
            help::LookupResult::Suggestions(s) => help::render_no_match(name, &s),
            help::LookupResult::NoMatch => help::render_no_match(name, &[]),
        },
    }
}

fn cmd_help(topic: Option<&str>) {
    print!("{}", format_help(topic));
}

fn sq_debug() -> bool {
    telemetry::sq_debug_enabled()
}

fn cmd_tick(cmd: &str, cwd: &str, exit_code: i32, test_sage: bool, hook_version: u32) {
    // A crates.io result fetched before (re)taking the save lock: the daily
    // network call must never hold the lock other shells' ticks wait on.
    let mut prefetched: Option<Option<String>> = None;
    if !sq_debug() && !state::save_path().exists() {
        return; // no character: stay silent and create nothing
    }
    loop {
        let lock = match state::lock(state::TICK_LOCK_TIMEOUT) {
            Ok(lock) => lock,
            Err(e) => {
                if sq_debug() {
                    eprintln!("{} Tick lock failure: {}", "❌".bold(), e.to_string().red());
                    std::process::exit(1);
                }
                if let state::LockError::Io(_) = e {
                    // Not contention: a broken lock file would otherwise stop the game silently.
                    eprintln!("{} sq: {}", "⚠️".yellow(), e);
                }
                return; // another sq holds the save; skip this tick rather than stall the prompt
            }
        };
        let mut game = match state::load_locked(&lock) {
            Ok(g) => g,
            Err(e) => {
                if sq_debug() {
                    eprintln!("{} Tick load failure: {}", "❌".bold(), e.red());
                    std::process::exit(1);
                }
                if state::save_path().exists() {
                    // Unreadable (not missing): say so instead of silently doing nothing.
                    eprintln!("{} sq: {}", "⚠️".yellow(), e);
                }
                return; // Silently skip if no character
            }
        };

        if prefetched.is_none() && sage::update_check_due(&game) {
            drop(lock);
            prefetched = Some(sage::fetch_latest_version());
            continue; // re-lock and reload: another shell may have saved meanwhile
        }

        if hook_version < hook::VERSION
            && game.hook_notice_version < hook::VERSION
            && io::stdin().is_terminal()
            && io::stderr().is_terminal()
        {
            // Called by a hook that older versions appended to the rc file verbatim.
            // Told once, where the player can see it (sq 1.0's hook discards stderr).
            print_hook_upgrade_notice();
            game.hook_notice_version = hook::VERSION;
        }

        events::tick(&mut game, cmd, cwd, exit_code);
        if test_sage {
            sage::force_show_sage(&mut game, prefetched.take());
        } else {
            sage::maybe_show_sage(&mut game, prefetched.take());
        }
        game.last_tick = chrono::Utc::now();

        if let Err(e) = state::save(&game, &lock) {
            eprintln!("{} Failed to save: {}", "❌".bold(), e.red());
            if sq_debug() {
                std::process::exit(1);
            }
        }
        return;
    }
}

/// Take the save lock and load the character for a read-modify-write command.
/// Keep the returned lock alive until after `state::save`, and don't use this for
/// commands that prompt the player (the lock would stall every shell's tick).
fn load_for_update() -> Option<(state::SaveLock, state::GameState)> {
    let lock = match state::lock(state::COMMAND_LOCK_TIMEOUT) {
        Ok(lock) => lock,
        Err(e) => {
            eprintln!("{} {}", "❌".bold(), e.to_string().red());
            return None;
        }
    };
    match state::load_locked(&lock) {
        Ok(game) => Some((lock, game)),
        Err(e) => {
            eprintln!("{} {}", "❌".bold(), e.red());
            None
        }
    }
}

/// One-time notice for players whose rc file still holds a hook older versions
/// appended verbatim (it re-ticks the previous command on an empty Enter).
fn print_hook_upgrade_notice() {
    let shell = std::env::var("SHELL")
        .ok()
        .and_then(|s| s.rsplit('/').next().map(str::to_string))
        .filter(|s| hook::code(s).is_some())
        .unwrap_or_else(|| "<bash|zsh|fish>".to_string());
    eprintln!(
        "{} Your shellquest hook is out of date: it counts an empty Enter as a repeat of your last command.\n   Run {} to upgrade it (your rc file is backed up first).",
        "🪝".bold(),
        format!("sq hook --shell {} --install", shell).cyan()
    );
}

/// The rc files a shell reads, in the order `--install` considers them.
fn rc_candidates(shell: &str) -> Vec<PathBuf> {
    let Some(home) = dirs::home_dir() else {
        return Vec::new();
    };
    match shell {
        "zsh" => vec![home.join(".zshrc"), home.join(".zshrc_local")],
        "bash" => vec![home.join(".bashrc"), home.join(".bash_profile")],
        "fish" => vec![home.join(".config/fish/config.fish")],
        _ => Vec::new(),
    }
}

/// Create a file that doesn't exist yet, trying `name(0)`, `name(1)`, ... in turn.
/// It starts out readable only by the player; callers set its final permissions.
fn create_unique(name: impl Fn(u32) -> PathBuf) -> std::io::Result<(PathBuf, std::fs::File)> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    for n in 0..100 {
        let path = name(n);
        match options.open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "no free file name",
    ))
}

/// Replace an rc file's contents, which were `original` when the change was planned,
/// keeping a copy of the original next to it. The new text goes to a temp file beside
/// the file's real path and is renamed over it, so a crash can't leave the file
/// half-written and a symlinked rc file (a dotfiles repo) stays a symlink. The file
/// keeps its permissions. Returns the backup's path.
fn rewrite_rc(path: &Path, original: &[u8], contents: &str) -> Result<PathBuf, String> {
    use std::io::Write;
    let fail =
        |what: &str, e: std::io::Error| format!("could not {} {}: {}", what, path.display(), e);
    let real = std::fs::canonicalize(path).map_err(|e| fail("resolve", e))?;
    if std::fs::read(&real).map_err(|e| fail("read", e))? != original {
        return Err(format!(
            "{} changed while sq was reading it; run this again",
            path.display()
        ));
    }
    let permissions = std::fs::metadata(&real)
        .map_err(|e| fail("read", e))?
        .permissions();

    let stamp = chrono::Local::now().format("%Y%m%d%H%M%S");
    let (backup, mut file) = create_unique(|n| {
        let suffix = if n == 0 {
            String::new()
        } else {
            format!("-{}", n)
        };
        PathBuf::from(format!("{}.sq-backup-{}{}", path.display(), stamp, suffix))
    })
    .map_err(|e| fail("back up", e))?;
    file.write_all(original)
        .and_then(|()| file.sync_all())
        .and_then(|()| file.set_permissions(permissions.clone()))
        .map_err(|e| fail("back up", e))?;

    let dir = real.parent().unwrap_or(Path::new("."));
    let name = real
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let (tmp, mut file) =
        create_unique(|n| dir.join(format!("{}.sq-tmp-{}-{}", name, std::process::id(), n)))
            .map_err(|e| fail("write", e))?;
    let written = file
        .write_all(contents.as_bytes())
        .and_then(|()| file.sync_all())
        .and_then(|()| file.set_permissions(permissions))
        .and_then(|()| std::fs::rename(&tmp, &real));
    if let Err(e) = written {
        let _ = std::fs::remove_file(&tmp);
        return Err(fail("write", e));
    }
    Ok(backup)
}

fn cmd_hook(shell: &str, install: bool, file: Option<String>) {
    let (Some(code), Some(loader)) = (hook::code(shell), hook::loader(shell)) else {
        eprintln!(
            "{} Unknown shell: {}. Supported: bash, zsh, fish",
            "❌".bold(),
            shell.red()
        );
        std::process::exit(2);
    };

    if !install {
        // Printed for `eval "$(sq hook --shell <sh>)"` (what --install writes) or pasting.
        print!("{}", code);
        return;
    }

    let fail = |e: String| -> ! {
        eprintln!("{} {}", "❌".bold(), e.red());
        std::process::exit(1);
    };

    // The explicit file first, then the shell's usual rc files: a hook from an older
    // version may live in any of them. Each file that has one is upgraded in place
    // (login and interactive shells may read different files); the hook registers
    // itself once per shell however many files load it.
    let mut files: Vec<PathBuf> = file.iter().map(PathBuf::from).collect();
    files.extend(rc_candidates(shell));
    let Some(target) = files.first().cloned() else {
        fail(format!("Could not determine an rc file for {}", shell));
    };

    let mut seen: Vec<PathBuf> = Vec::new();
    let mut upgrades = Vec::new();
    let mut installed_in = Vec::new();
    let target_identity = std::fs::canonicalize(&target).unwrap_or_else(|_| target.clone());
    let mut target_has_hook = false;
    for path in &files {
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => fail(format!("Could not read {}: {}", path.display(), e)),
        };
        // The same file under two names (symlink, relative path) is handled once.
        let identity = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());
        if seen.contains(&identity) {
            continue;
        }
        let is_target = identity == target_identity;
        seen.push(identity);
        // Hooks are plain ASCII, so a file with other bytes elsewhere can still be
        // searched and appended to; it just isn't rewritten from a lossy copy.
        match hook::plan_rc(shell, &String::from_utf8_lossy(&bytes)) {
            hook::RcPlan::Absent => continue,
            hook::RcPlan::Unrecognized => fail(format!(
                "{} has a shellquest hook this version doesn't recognize (edited by hand?).\n   Remove it, then run this command again, or replace it with these lines yourself:\n\n{}",
                path.display(),
                loader
            )),
            hook::RcPlan::Present { current: true, .. } => installed_in.push(path.clone()),
            hook::RcPlan::Present {
                current: false,
                with_loader,
            } => {
                if std::str::from_utf8(&bytes).is_err() {
                    fail(format!(
                        "{} isn't UTF-8 text, so sq won't rewrite it. Replace the old hook in it with these lines yourself:\n\n{}",
                        path.display(),
                        loader
                    ));
                }
                upgrades.push((path.clone(), bytes, with_loader));
            }
        }
        target_has_hook |= is_target;
    }

    for path in &installed_in {
        println!(
            "{} Hook already installed in {}",
            "✓".green().bold(),
            path.display().to_string().cyan()
        );
    }
    for (path, original, contents) in &upgrades {
        match rewrite_rc(path, original, contents) {
            Ok(backup) => println!(
                "{} Upgraded the shellquest hook in {} (backup: {})",
                "✓".green().bold(),
                path.display().to_string().cyan(),
                backup.display()
            ),
            Err(e) => fail(format!(
                "Failed to upgrade the hook: {}\n   Replace the old hook in {} with these lines yourself:\n\n{}",
                e,
                path.display(),
                loader
            )),
        }
    }

    // An explicit --file always ends up with the loader; otherwise the default rc
    // file gets one only when no file has a hook yet.
    let append = if file.is_some() {
        !target_has_hook
    } else {
        installed_in.is_empty() && upgrades.is_empty()
    };
    let activate = if append {
        if let Some(dir) = target.parent() {
            let _ = std::fs::create_dir_all(dir); // fish's config dir may not exist yet
        }
        let existing = std::fs::read(&target).unwrap_or_default();
        let separator = if existing.is_empty() {
            ""
        } else if existing.ends_with(b"\n") {
            "\n"
        } else {
            "\n\n"
        };
        use std::io::Write;
        let appended = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&target)
            .and_then(|mut f| f.write_all(format!("{}{}", separator, loader).as_bytes()));
        if let Err(e) = appended {
            fail(format!("Failed to write {}: {}", target.display(), e));
        }
        println!(
            "{} Hook installed to {}",
            "✓".green().bold(),
            target.display().to_string().cyan()
        );
        Some(target.clone())
    } else {
        upgrades.first().map(|(path, ..)| path.clone())
    };
    if let Some(path) = activate {
        println!(
            "  Open a new terminal to activate it (or run {}).",
            format!("source {}", path.display()).dimmed()
        );
    }
}

fn cmd_prestige() {
    let mut game = match state::load() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{} {}", "❌".bold(), e.red());
            return;
        }
    };

    if !game.character.can_prestige() {
        println!(
            "{} You must reach level {} to prestige. Current level: {}",
            "⚠️".yellow(),
            format!("{}", character::MAX_LEVEL).cyan().bold(),
            format!("{}", game.character.level).white().bold()
        );
        return;
    }

    println!();
    println!("{}", "✨ PRESTIGE ✨".yellow().bold().on_black());
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".yellow()
    );
    println!();
    println!(
        "  You will {} to level {} but gain:",
        "reset".red().bold(),
        "1".white().bold()
    );
    println!(
        "  {} {} to all stats per prestige tier",
        "•".yellow(),
        "+2".green().bold()
    );
    println!(
        "  {} A {} with unique stat bonuses",
        "•".yellow(),
        "subclass".magenta().bold()
    );
    println!(
        "  {} {} HP per prestige tier",
        "•".yellow(),
        "+10".green().bold()
    );
    println!(
        "  {} You {} your gold, gear, kills, and inventory",
        "•".yellow(),
        "keep".green().bold()
    );
    println!();

    let subclasses = character::Subclass::available_for(&game.character.class);
    println!("{}", "Choose your subclass:".bold().yellow());
    for (i, sub) in subclasses.iter().enumerate() {
        let (s, d, int) = sub.stat_bonus();
        println!(
            "  {} {} — STR:{} DEX:{} INT:{}",
            format!("{}.", i + 1).dimmed(),
            format!("{}", sub).magenta().bold(),
            format!("+{}", s).red(),
            format!("+{}", d).green(),
            format!("+{}", int).blue()
        );
    }

    let subclass = loop {
        let choice = prompt(&format!(
            "{} Choose [1-{}]: ",
            "🎭".bold(),
            subclasses.len()
        ));
        if let Ok(n) = choice.parse::<usize>() {
            if n >= 1 && n <= subclasses.len() {
                break subclasses[n - 1].clone();
            }
        }
        println!("{}", format!("  Pick 1-{}.", subclasses.len()).red());
    };

    let confirm = prompt(&format!(
        "{} Prestige as {}? This resets your level! [y/N] ",
        "⚠️".yellow(),
        format!("{}", subclass).magenta().bold()
    ));

    if confirm.to_lowercase() != "y" {
        println!("{}", "Cancelled.".dimmed());
        return;
    }

    let sub_name = format!("{}", subclass);
    game.character.prestige(subclass);

    // Locks only the final write; reloading to keep other shells' progress is x3p.2.
    let saved = state::lock(state::COMMAND_LOCK_TIMEOUT)
        .map_err(|e| e.to_string())
        .and_then(|lock| state::save(&game, &lock));
    match saved {
        Ok(()) => {
            println!();
            println!(
                "{}",
                "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".yellow()
            );
            println!(
                "{} {} has ascended as a {} {}! Prestige tier: {}",
                "✨".bold(),
                game.character.name.bold().green(),
                sub_name.magenta().bold(),
                format!("{}", game.character.class).cyan(),
                format!("{}", game.character.prestige).yellow().bold()
            );
            println!(
                "{}",
                "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".yellow()
            );
            println!();
        }
        Err(e) => {
            eprintln!("{} Failed to save: {}", "❌".bold(), e.red());
        }
    }
}

fn refresh_shop_if_needed(game: &mut state::GameState) {
    use chrono::Utc;

    let now = Utc::now();
    let today_midnight = now.date_naive().and_hms_opt(0, 0, 0).unwrap();

    let needs_refresh = match game.shop_refreshed {
        None => true,
        Some(last) => last.date_naive() < today_midnight.date(),
    };

    if needs_refresh {
        game.shop_items.clear();
        for _ in 0..6 {
            game.shop_items.push(loot::roll_shop_loot());
        }
        game.shop_refreshed = Some(now);
    }
}

const QUEST_PHRASE_WORDS: &[&str] = &[
    "ashen", "root", "sigil", "hollow", "prompt", "ember", "null", "rune", "echo", "vault",
    "cursor", "shadow", "inode", "glyph", "midnight", "pipe", "shell", "cipher", "kernel",
    "static", "oracle", "thread", "daemon", "cache",
];

const QUEST_BASE_XP: u32 = 75;
const QUEST_XP_PER_LEVEL: u32 = 10;
const QUEST_BASE_GOLD: u32 = 40;
const QUEST_GOLD_PER_LEVEL: u32 = 5;

#[derive(Debug, PartialEq, Eq)]
enum QuestAnswerStatus {
    Correct,
    Wrong,
    AlreadyCompleted,
    NoActiveQuest,
}

#[derive(Debug)]
struct QuestAnswerResult {
    status: QuestAnswerStatus,
    xp: u32,
    gold: u32,
    item_name: Option<String>,
    item_rarity: Option<character::Rarity>,
}

fn normalize_quest_answer(answer: &str) -> String {
    answer
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn quest_is_current(game: &state::GameState, now: chrono::DateTime<chrono::Utc>) -> bool {
    matches!(game.quest_refreshed, Some(last) if last.date_naive() == now.date_naive())
}

fn quest_needs_refresh(game: &state::GameState, now: chrono::DateTime<chrono::Utc>) -> bool {
    !quest_is_current(game, now) || game.quest_phrase.is_none()
}

fn refresh_quest_state_if_needed(
    game: &mut state::GameState,
    now: chrono::DateTime<chrono::Utc>,
    phrase: String,
    scroll_path: Option<std::path::PathBuf>,
) -> bool {
    if !quest_needs_refresh(game, now) {
        return false;
    }

    game.quest_refreshed = Some(now);
    game.quest_phrase = Some(phrase);
    game.quest_scroll_path = scroll_path;
    game.quest_completed_today = false;
    true
}

fn quest_seed(game: &state::GameState, now: chrono::DateTime<chrono::Utc>) -> u64 {
    use chrono::Datelike;
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    now.date_naive().year().hash(&mut hasher);
    now.date_naive().ordinal().hash(&mut hasher);
    game.character.name.hash(&mut hasher);
    game.created_at.timestamp().hash(&mut hasher);
    hasher.finish()
}

fn quest_phrase_for_day(game: &state::GameState, now: chrono::DateTime<chrono::Utc>) -> String {
    use rand::{Rng, SeedableRng};

    let mut rng = rand::rngs::StdRng::seed_from_u64(quest_seed(game, now));
    let mut words = Vec::new();
    while words.len() < 3 {
        let word = QUEST_PHRASE_WORDS[rng.gen_range(0..QUEST_PHRASE_WORDS.len())];
        if !words.contains(&word) {
            words.push(word);
        }
    }
    words.join(" ")
}

fn quest_scroll_contents(phrase: &str) -> String {
    format!(
        "A vellum curl hums with terminal-static.\n\nSpeak this phrase back to the quest-giver:\n{}\n",
        phrase
    )
}

fn refresh_quest_if_needed_at(
    game: &mut state::GameState,
    now: chrono::DateTime<chrono::Utc>,
    rng: &mut impl rand::Rng,
) -> Result<bool, String> {
    if !quest_needs_refresh(game, now) {
        return Ok(false);
    }

    let phrase = quest_phrase_for_day(game, now);
    let contents = quest_scroll_contents(&phrase);
    let root = void::generate_void(rng).map_err(|e| format!("Failed to shape the Void: {}", e))?;
    let scroll_path = void::hide_file_in_void(&root, &contents, rng)
        .map_err(|e| format!("Failed to hide the quest scroll: {}", e))?;
    Ok(refresh_quest_state_if_needed(
        game,
        now,
        phrase,
        Some(scroll_path),
    ))
}

fn cleanup_quest_scroll(game: &mut state::GameState) {
    let Some(path) = game.quest_scroll_path.take() else {
        return;
    };

    match std::fs::remove_file(&path) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => eprintln!(
            "{} Could not clean up quest scroll {}: {}",
            "⚠️".yellow(),
            path.display(),
            e.to_string().red()
        ),
    }
}

fn quest_reward_amounts(game: &state::GameState) -> (u32, u32) {
    let level = game.character.level;
    (
        character::scale_xp_gain(QUEST_BASE_XP + level * QUEST_XP_PER_LEVEL),
        QUEST_BASE_GOLD + level * QUEST_GOLD_PER_LEVEL,
    )
}

fn apply_quest_answer(
    game: &mut state::GameState,
    answer: &str,
    now: chrono::DateTime<chrono::Utc>,
    reward_item: character::Item,
) -> QuestAnswerResult {
    if game.quest_completed_today && quest_is_current(game, now) {
        return QuestAnswerResult {
            status: QuestAnswerStatus::AlreadyCompleted,
            xp: 0,
            gold: 0,
            item_name: None,
            item_rarity: None,
        };
    }

    let Some(expected) = game.quest_phrase.clone() else {
        return QuestAnswerResult {
            status: QuestAnswerStatus::NoActiveQuest,
            xp: 0,
            gold: 0,
            item_name: None,
            item_rarity: None,
        };
    };

    if !quest_is_current(game, now) {
        return QuestAnswerResult {
            status: QuestAnswerStatus::NoActiveQuest,
            xp: 0,
            gold: 0,
            item_name: None,
            item_rarity: None,
        };
    }

    if normalize_quest_answer(answer) != normalize_quest_answer(&expected) {
        return QuestAnswerResult {
            status: QuestAnswerStatus::Wrong,
            xp: 0,
            gold: 0,
            item_name: None,
            item_rarity: None,
        };
    }

    let (xp, gold) = quest_reward_amounts(game);
    let item_name = reward_item.name.clone();
    let item_rarity = reward_item.rarity;
    let leveled = game.character.gain_xp(xp);
    game.character.gold += gold;
    events::add_to_inventory_pub(game, reward_item);
    game.quest_completed_today = true;
    cleanup_quest_scroll(game);

    let (plain, _) = messages::quest_reward(&game.character.class, &item_name, xp, gold);
    game.add_journal(journal::JournalEntry::new(journal::EventType::Quest, plain));
    if leveled {
        events::emit_level_up(game);
    }

    QuestAnswerResult {
        status: QuestAnswerStatus::Correct,
        xp,
        gold,
        item_name: Some(item_name),
        item_rarity: Some(item_rarity),
    }
}

fn ensure_quest_home() -> bool {
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let home = dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    if cwd != home {
        eprintln!(
            "{} The quest-giver only receives visitors in your {}. You are in {}",
            "🏠".bold(),
            "home directory".cyan().bold(),
            cwd.dimmed()
        );
        eprintln!("  Run {} to return home first.", "cd ~".cyan());
        return false;
    }

    true
}

fn cmd_quest(action: Option<QuestAction>) {
    if !ensure_quest_home() {
        return;
    }

    if let Some(QuestAction::Answer { phrase }) = &action {
        if phrase.is_empty() {
            eprintln!(
                "{} Usage: {}",
                "❌".bold(),
                "sq quest answer \"<phrase>\"".cyan()
            );
            return;
        }
    }

    let Some((lock, mut game)) = load_for_update() else {
        return;
    };

    let now = chrono::Utc::now();
    let mut rng = rand::thread_rng();
    if let Err(e) = refresh_quest_if_needed_at(&mut game, now, &mut rng) {
        eprintln!("{} {}", "❌".bold(), e.red());
        return;
    }

    match action {
        None => {
            if game.quest_completed_today && quest_is_current(&game, now) {
                display::print_quest(
                    "You've already claimed today's boon. Return after midnight UTC.",
                );
            } else {
                let (_, colored) = messages::quest_giver(&game.character.class);
                display::print_quest(&colored);
                eprintln!(
                    "  Bring it back with {}.",
                    "sq quest answer \"<phrase>\"".cyan()
                );
            }
        }
        Some(QuestAction::Answer { phrase }) => {
            let answer = phrase.join(" ");
            let reward = loot::roll_loot_with_min_rarity(character::Rarity::Rare);
            let result = apply_quest_answer(&mut game, &answer, now, reward);
            match result.status {
                QuestAnswerStatus::Correct => {
                    if let (Some(item_name), Some(item_rarity)) =
                        (&result.item_name, result.item_rarity)
                    {
                        let (_, colored) = messages::quest_reward(
                            &game.character.class,
                            item_name,
                            result.xp,
                            result.gold,
                        );
                        display::print_quest(&colored);
                        display::print_loot(&format!("Quest boon: {}", item_name), &item_rarity);
                    }
                }
                QuestAnswerStatus::Wrong => {
                    display::print_quest(
                        "The quest-giver shakes their head. That is not the phrase.",
                    );
                }
                QuestAnswerStatus::AlreadyCompleted => {
                    display::print_quest(
                        "You've already claimed today's boon. Return after midnight UTC.",
                    );
                }
                QuestAnswerStatus::NoActiveQuest => {
                    display::print_quest(
                        "No active scroll is waiting. Ask for today's quest first.",
                    );
                }
            }
        }
    }

    if let Err(e) = state::save(&game, &lock) {
        eprintln!("{} Failed to save: {}", "❌".bold(), e.red());
    }
}

fn cmd_shop() {
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let home = dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    if cwd != home {
        println!(
            "{} The shop is only accessible from your {}. You are in {}",
            "🏠".bold(),
            "home directory".cyan().bold(),
            cwd.dimmed()
        );
        println!("  Run {} to return home first.", "cd ~".cyan());
        return;
    }

    let Some((lock, mut game)) = load_for_update() else {
        return;
    };

    refresh_shop_if_needed(&mut game);

    println!();
    println!("{}", "🏪 The Terminal Bazaar".bold().yellow());
    println!("{}", "─".repeat(50).dimmed());
    println!(
        "  {} {}",
        "Your gold:".bold(),
        format!("{}", game.character.gold).yellow().bold()
    );
    println!("{}", "─".repeat(50).dimmed());

    if game.shop_items.is_empty() {
        println!("{}", "  The shop is empty... come back tomorrow.".dimmed());
    } else {
        for (i, item) in game.shop_items.iter().enumerate() {
            let price = loot::item_price(item);
            let rarity_str = match item.rarity {
                character::Rarity::Common => format!("{}", "[Common]".dimmed()),
                character::Rarity::Uncommon => format!("{}", "[Uncommon]".dimmed().bold()),
                character::Rarity::Rare => format!("{}", "[Rare]".green().bold()),
                _ => format!("{}", item.rarity),
            };
            let affordable = if game.character.gold >= price {
                "".to_string()
            } else {
                format!(" {}", "(can't afford)".red().dimmed())
            };
            println!(
                "  {}. {} (+{} {}) {} — {} gold{}",
                format!("{}", i + 1).dimmed(),
                item.name.white().bold(),
                item.power,
                format!("{}", item.slot).dimmed(),
                rarity_str,
                format!("{}", price).yellow().bold(),
                affordable
            );
        }
    }

    println!("{}", "─".repeat(50).dimmed());
    println!("  Use {} to purchase an item.", "sq buy <number>".cyan());
    println!("  Shop refreshes daily at {}.", "midnight UTC".dimmed());
    println!();

    if let Err(e) = state::save(&game, &lock) {
        eprintln!("{} Failed to save: {}", "❌".bold(), e.red());
    }
}

fn cmd_buy(number: usize) {
    if number == 0 {
        eprintln!(
            "{} Usage: {} (see {} for numbered list)",
            "❌".bold(),
            "sq buy <number>".cyan(),
            "sq shop".cyan()
        );
        return;
    }

    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let home = dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    if cwd != home {
        println!(
            "{} The shop is only accessible from your {}.",
            "🏠".bold(),
            "home directory".cyan().bold()
        );
        return;
    }

    let Some((lock, mut game)) = load_for_update() else {
        return;
    };

    refresh_shop_if_needed(&mut game);

    let idx = number - 1;
    if idx >= game.shop_items.len() {
        println!(
            "{} Invalid item number {}. The shop has {} items. Run {} to see the list.",
            "⚠️".yellow(),
            format!("{}", number).white().bold(),
            format!("{}", game.shop_items.len()).white().bold(),
            "sq shop".cyan()
        );
        return;
    }

    let price = loot::item_price(&game.shop_items[idx]);

    if game.character.gold < price {
        println!(
            "{} Not enough gold! {} costs {} gold, you have {}.",
            "⚠️".yellow(),
            game.shop_items[idx].name.white().bold(),
            format!("{}", price).yellow().bold(),
            format!("{}", game.character.gold).yellow()
        );
        return;
    }

    let item = game.shop_items.remove(idx);
    let item_name = item.name.clone();
    game.character.gold -= price;
    game.character.inventory.push(item);

    println!(
        "{} Purchased {} for {} gold! ({} gold remaining)",
        "💰".bold(),
        item_name.green().bold(),
        format!("{}", price).yellow().bold(),
        format!("{}", game.character.gold).yellow()
    );

    if let Err(e) = state::save(&game, &lock) {
        eprintln!("{} Failed to save: {}", "❌".bold(), e.red());
    }
}

fn cmd_sell(query: &str) {
    if query.is_empty() {
        eprintln!(
            "{} Usage: {} or {}",
            "❌".bold(),
            "sq sell <number>".cyan(),
            "sq sell <item name>".cyan()
        );
        return;
    }

    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let home = dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    if cwd != home {
        println!(
            "{} The shop is only accessible from your {}.",
            "🏠".bold(),
            "home directory".cyan().bold()
        );
        return;
    }

    let Some((lock, mut game)) = load_for_update() else {
        return;
    };

    if game.character.inventory.is_empty() {
        println!(
            "{} Nothing to sell. Check {}.",
            "⚠️".yellow(),
            "sq inventory".cyan()
        );
        return;
    }

    if query.eq_ignore_ascii_case("junk") {
        cmd_sell_junk(&mut game, &lock);
        return;
    }

    let idx = if let Ok(n) = query.parse::<usize>() {
        if n == 0 || n > game.character.inventory.len() {
            println!(
                "{} No item at slot {}. You have {} item{}. Check {}.",
                "⚠️".yellow(),
                format!("{}", n).white().bold(),
                format!("{}", game.character.inventory.len()).white().bold(),
                if game.character.inventory.len() == 1 {
                    ""
                } else {
                    "s"
                },
                "sq inventory".cyan()
            );
            return;
        }
        // N is the number `sq inventory` shows, not the position in the save file.
        display::inventory_display_order(&game.character.inventory)[n - 1]
    } else {
        match find_inventory_item(&game, query) {
            Ok(Some(i)) => i,
            Ok(None) => {
                println!(
                    "{} No item matching {} in your inventory.",
                    "⚠️".yellow(),
                    format!("\"{}\"", query).white().bold()
                );
                return;
            }
            Err(msg) => {
                println!("{} {}", "⚠️".yellow(), msg);
                return;
            }
        }
    };

    let sell_price = loot::sell_price(&game.character.inventory[idx]);
    let item = game.character.inventory.remove(idx);
    let old_gold = game.character.gold;
    game.character.gold += sell_price;

    println!(
        "{} Sold {} (+{} {}) [{}] for {} gold.",
        "💰".bold(),
        item.name.white().bold(),
        item.power,
        format!("{}", item.slot).dimmed(),
        format!("{}", item.rarity).dimmed(),
        format!("{}", sell_price).yellow().bold(),
    );
    println!(
        "   Gold: {} → {}",
        format!("{}", old_gold).dimmed(),
        format!("{}", game.character.gold).yellow().bold(),
    );

    if let Err(e) = state::save(&game, &lock) {
        eprintln!("{} Failed to save: {}", "❌".bold(), e.red());
    }
}

fn cmd_sell_junk(game: &mut state::GameState, lock: &state::SaveLock) {
    let inv = std::mem::take(&mut game.character.inventory);
    let result = sweep_junk(inv);

    if result.sold_count == 0 {
        game.character.inventory = result.kept;
        println!(
            "{} No junk in your inventory. {} and up are kept.",
            "⚠️".yellow(),
            "Rare".green().bold()
        );
        return;
    }

    let old_gold = game.character.gold;
    game.character.inventory = result.kept;
    game.character.gold += result.total_price;

    println!(
        "{} Sold {} junk item{} for {} gold.",
        "💰".bold(),
        format!("{}", result.sold_count).white().bold(),
        if result.sold_count == 1 { "" } else { "s" },
        format!("{}", result.total_price).yellow().bold(),
    );
    println!(
        "   Gold: {} → {}",
        format!("{}", old_gold).dimmed(),
        format!("{}", game.character.gold).yellow().bold(),
    );

    if let Err(e) = state::save(game, lock) {
        eprintln!("{} Failed to save: {}", "❌".bold(), e.red());
    }
}

struct SweepResult {
    kept: Vec<character::Item>,
    sold_count: usize,
    total_price: u32,
}

fn sweep_junk(items: Vec<character::Item>) -> SweepResult {
    let mut kept = Vec::new();
    let mut sold_count = 0;
    let mut total_price: u32 = 0;
    for item in items {
        if matches!(
            item.rarity,
            character::Rarity::Common | character::Rarity::Uncommon
        ) {
            total_price += loot::sell_price(&item);
            sold_count += 1;
        } else {
            kept.push(item);
        }
    }
    SweepResult {
        kept,
        sold_count,
        total_price,
    }
}

#[derive(Debug, PartialEq)]
enum EquippedSlot {
    Weapon,
    Armor,
    Ring,
}

#[derive(Debug, PartialEq)]
enum ItemLookup {
    Equipped(EquippedSlot),
    Inventory(usize),
}

fn find_equipped_slot_to_enchant(
    game: &state::GameState,
    query: &str,
) -> Result<EquippedSlot, &'static str> {
    let q = query.to_lowercase();
    let mut matches: Vec<EquippedSlot> = Vec::new();
    if let Some(w) = &game.character.weapon {
        if w.name.to_lowercase().contains(&q) || fuzzy_match_name(&w.name, query) {
            matches.push(EquippedSlot::Weapon);
        }
    }
    if let Some(a) = &game.character.armor {
        if a.name.to_lowercase().contains(&q) || fuzzy_match_name(&a.name, query) {
            matches.push(EquippedSlot::Armor);
        }
    }
    if let Some(r) = &game.character.ring {
        if r.name.to_lowercase().contains(&q) || fuzzy_match_name(&r.name, query) {
            matches.push(EquippedSlot::Ring);
        }
    }
    match matches.len() {
        0 => Err("no_match"),
        1 => Ok(matches.remove(0)),
        _ => Err("ambiguous"),
    }
}

fn cmd_enchant(query: &str) {
    if query.is_empty() {
        eprintln!(
            "{} Usage: {} (enchant an equipped item, +1 power per enchant, max +5)",
            "❌".bold(),
            "sq enchant <item name>".cyan()
        );
        return;
    }

    let Some((lock, mut game)) = load_for_update() else {
        return;
    };

    let is_wizard = matches!(game.character.class, character::Class::Wizard);
    if !is_wizard {
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let home = dirs::home_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        if cwd != home {
            println!(
                "{} The enchanting workbench is only accessible from your {}. (Wizards can enchant anywhere.)",
                "🏠".bold(),
                "home directory".cyan().bold()
            );
            return;
        }
    }

    let slot = match find_equipped_slot_to_enchant(&game, query) {
        Ok(s) => s,
        Err("no_match") => {
            println!(
                "{} No equipped item matching {}. Equip the item first with {} / {}.",
                "⚠️".yellow(),
                format!("\"{}\"", query).white().bold(),
                "sq wield".cyan(),
                "sq equip".cyan()
            );
            return;
        }
        Err("ambiguous") => {
            println!(
                "{} Multiple equipped items match {}. Be more specific.",
                "⚠️".yellow(),
                format!("\"{}\"", query).white().bold()
            );
            return;
        }
        Err(_) => return,
    };

    let item_ref = match slot {
        EquippedSlot::Weapon => game.character.weapon.as_ref().unwrap(),
        EquippedSlot::Armor => game.character.armor.as_ref().unwrap(),
        EquippedSlot::Ring => game.character.ring.as_ref().unwrap(),
    };

    if !loot::is_enchantable(item_ref) {
        println!("{} That item cannot be enchanted.", "⚠️".yellow());
        return;
    }

    if !loot::can_enchant_further(item_ref) {
        println!(
            "{} {} is already at maximum enchantment ({}).",
            "⚠️".yellow(),
            item_ref.name.white().bold(),
            format!("+{}", loot::MAX_ENCHANT_LEVEL).yellow().bold()
        );
        return;
    }

    let cost = loot::enchant_cost(item_ref);
    if game.character.gold < cost {
        println!(
            "{} Not enough gold. Need {} gold (you have {}).",
            "⚠️".yellow(),
            format!("{}", cost).yellow().bold(),
            format!("{}", game.character.gold).yellow()
        );
        return;
    }

    let item_mut = match slot {
        EquippedSlot::Weapon => game.character.weapon.as_mut().unwrap(),
        EquippedSlot::Armor => game.character.armor.as_mut().unwrap(),
        EquippedSlot::Ring => game.character.ring.as_mut().unwrap(),
    };
    item_mut.enchant_level += 1;
    let new_level = item_mut.enchant_level;
    let item_name = item_mut.name.clone();

    let old_gold = game.character.gold;
    game.character.gold -= cost;

    println!(
        "{} {} is now {} (cost: {} gold).",
        "✨".bold(),
        item_name.white().bold(),
        display::enchant_tag(new_level).trim_start(),
        format!("{}", cost).yellow().bold()
    );
    println!(
        "   Gold: {} → {}",
        format!("{}", old_gold).dimmed(),
        format!("{}", game.character.gold).yellow().bold()
    );

    if let Err(e) = state::save(&game, &lock) {
        eprintln!("{} Failed to save: {}", "❌".bold(), e.red());
    }
}

fn cmd_identify(query: &str) {
    if query.is_empty() {
        eprintln!(
            "{} Usage: {} (alias: {})",
            "❌".bold(),
            "sq identify <item name>".cyan(),
            "sq id <item name>".cyan()
        );
        return;
    }

    let game = match state::load() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{} {}", "❌".bold(), e.red());
            return;
        }
    };

    let lookup = match find_inventory_or_equipped_item(&game, query) {
        Ok(Some(l)) => l,
        Ok(None) => {
            println!(
                "{} No item matching {} in your inventory or equipped slots.",
                "⚠️".yellow(),
                format!("\"{}\"", query).white().bold()
            );
            return;
        }
        Err(msg) => {
            println!("{} {}", "⚠️".yellow(), msg);
            return;
        }
    };

    let total_inv = game.character.inventory.len();
    let (item_ref, source) = match lookup {
        ItemLookup::Equipped(EquippedSlot::Weapon) => (
            game.character.weapon.as_ref().unwrap(),
            display::ItemSource::Equipped,
        ),
        ItemLookup::Equipped(EquippedSlot::Armor) => (
            game.character.armor.as_ref().unwrap(),
            display::ItemSource::Equipped,
        ),
        ItemLookup::Equipped(EquippedSlot::Ring) => (
            game.character.ring.as_ref().unwrap(),
            display::ItemSource::Equipped,
        ),
        ItemLookup::Inventory(idx) => (
            &game.character.inventory[idx],
            display::ItemSource::Inventory {
                index: display::inventory_display_number(&game.character.inventory, idx)
                    .unwrap_or(idx + 1),
                total: total_inv,
            },
        ),
    };

    display::print_item_detail(item_ref, source);
}

fn fuzzy_match_name(item_name: &str, query: &str) -> bool {
    let name_lower = item_name.to_lowercase();
    query
        .to_lowercase()
        .split_whitespace()
        .all(|token| name_lower.contains(token))
}

/// Storage indices of inventory items matching `query`, in the order `sq inventory`
/// lists them, so "first match" and `name.N` mean what the player sees.
fn find_inventory_items(game: &state::GameState, query: &str) -> Vec<usize> {
    let query_lower = query.to_lowercase();
    let inv = &game.character.inventory;
    let mut matched: Vec<usize> = display::inventory_display_order(inv)
        .into_iter()
        .filter(|&i| {
            let name_lower = inv[i].name.to_lowercase();
            name_lower == query_lower
                || name_lower.contains(&query_lower)
                || fuzzy_match_name(&inv[i].name, query)
        })
        .collect();
    matched.dedup();
    matched
}

fn find_inventory_item(game: &state::GameState, name: &str) -> Result<Option<usize>, String> {
    let (query, n) = if let Some(dot_pos) = name.rfind('.') {
        let suffix = &name[dot_pos + 1..];
        match suffix.parse::<usize>() {
            Ok(0) => {
                return Err("Item index must be 1 or higher (e.g. potion.1)".to_string());
            }
            Ok(n) => (&name[..dot_pos], n),
            Err(_) => (name, 1usize),
        }
    } else {
        (name, 1usize)
    };

    let matches = find_inventory_items(game, query);

    if matches.is_empty() {
        return Ok(None);
    }

    match matches.get(n - 1) {
        Some(&idx) => Ok(Some(idx)),
        None => Err(format!(
            "Only {} '{}' item(s) found — use {}.1 … {}.{}",
            matches.len(),
            query,
            query,
            query,
            matches.len()
        )),
    }
}

fn equipped_item_name_matches(item: &character::Item, query: &str) -> bool {
    let q = query.to_lowercase();
    let name_lower = item.name.to_lowercase();
    name_lower == q || name_lower.contains(&q) || fuzzy_match_name(&item.name, query)
}

fn find_inventory_or_equipped_item(
    game: &state::GameState,
    name: &str,
) -> Result<Option<ItemLookup>, String> {
    let (query, n) = if let Some(dot_pos) = name.rfind('.') {
        let suffix = &name[dot_pos + 1..];
        match suffix.parse::<usize>() {
            Ok(0) => {
                return Err("Item index must be 1 or higher (e.g. scythe.1)".to_string());
            }
            Ok(n) => (&name[..dot_pos], n),
            Err(_) => (name, 1usize),
        }
    } else {
        (name, 1usize)
    };

    let mut candidates: Vec<ItemLookup> = Vec::new();
    if let Some(w) = &game.character.weapon {
        if equipped_item_name_matches(w, query) {
            candidates.push(ItemLookup::Equipped(EquippedSlot::Weapon));
        }
    }
    if let Some(a) = &game.character.armor {
        if equipped_item_name_matches(a, query) {
            candidates.push(ItemLookup::Equipped(EquippedSlot::Armor));
        }
    }
    if let Some(r) = &game.character.ring {
        if equipped_item_name_matches(r, query) {
            candidates.push(ItemLookup::Equipped(EquippedSlot::Ring));
        }
    }
    for idx in find_inventory_items(game, query) {
        candidates.push(ItemLookup::Inventory(idx));
    }

    if candidates.is_empty() {
        return Ok(None);
    }

    match candidates.into_iter().nth(n - 1) {
        Some(c) => Ok(Some(c)),
        None => Err(format!(
            "Only matching items below '{}.{}' index — use a lower number.",
            query, n
        )),
    }
}

#[cfg(test)]
#[test]
fn registry_covers_all_canonical_topics() {
    let names: Vec<&str> = help::all_topics().iter().map(|t| t.name).collect();
    let expected: Vec<&str> = help::CANONICAL_TOPIC_ORDER.to_vec();
    assert_eq!(
        names, expected,
        "registry must contain exactly the canonical topics in canonical order"
    );
}

#[cfg(test)]
#[test]
fn registry_related_topics_are_known() {
    use std::collections::HashSet;
    let canonical: HashSet<&str> = help::CANONICAL_TOPIC_ORDER.iter().copied().collect();
    for topic in help::all_topics() {
        for related in topic.related {
            assert!(
                canonical.contains(*related),
                "topic '{}' references unknown related topic '{}'",
                topic.name,
                related
            );
        }
    }
}

#[cfg(test)]
#[test]
fn lookup_prefers_primary_then_alias() {
    use help::{lookup_topic, LookupResult};

    match lookup_topic("status") {
        LookupResult::Found(t) => assert_eq!(t.name, "status"),
        other => panic!("expected Found(status) for canonical name, got {:?}", other),
    }

    match lookup_topic("stat") {
        LookupResult::Found(t) => assert_eq!(
            t.name, "status",
            "alias 'stat' must resolve to canonical 'status'"
        ),
        other => panic!("expected Found(status) via alias, got {:?}", other),
    }
}

#[cfg(test)]
#[test]
fn lookup_typo_and_gibberish_paths() {
    use help::{lookup_topic, LookupResult};

    match lookup_topic("jounral") {
        LookupResult::Suggestions(s) => {
            assert!(
                !s.is_empty(),
                "expected at least one suggestion for 'jounral'"
            );
            assert_eq!(
                s[0].name, "journal",
                "first suggestion for 'jounral' must be 'journal'"
            );
        }
        other => panic!("expected Suggestions for 'jounral', got {:?}", other),
    }

    match lookup_topic("xyzzy") {
        LookupResult::NoMatch => {}
        other => panic!("expected NoMatch for 'xyzzy', got {:?}", other),
    }
}

#[cfg(test)]
#[test]
fn render_index_lists_topics_in_order() {
    colored::control::set_override(false);

    let out = help::render_index();

    let mut search_start = 0usize;
    for name in help::CANONICAL_TOPIC_ORDER {
        let needle = format!("  {}", name);
        let rel_pos = out[search_start..].find(&needle).unwrap_or_else(|| {
            panic!(
                "topic row '{}' not found at or after offset {} in render_index() output:\n{}",
                name, search_start, out
            )
        });
        search_start += rel_pos + needle.len();
    }

    assert!(
        out.contains(help::INDEX_FOOTER),
        "render_index must include the exact footer '{}'; got:\n{}",
        help::INDEX_FOOTER,
        out
    );
}

#[cfg(test)]
#[test]
fn render_no_match_shows_suggestions() {
    use help::{lookup_topic, LookupResult};

    colored::control::set_override(false);

    let suggestions = match lookup_topic("jounral") {
        LookupResult::Suggestions(s) => s,
        other => panic!("expected Suggestions for 'jounral', got {:?}", other),
    };

    let out = help::render_no_match("jounral", &suggestions);

    assert!(
        out.contains("jounral"),
        "no-match output must echo the original query 'jounral':\n{}",
        out
    );
    assert!(
        out.contains("journal"),
        "no-match output must surface 'journal' as a close match:\n{}",
        out
    );
    assert!(
        out.to_lowercase().contains("did you mean"),
        "no-match output must clearly suggest close matches (looked for 'did you mean'):\n{}",
        out
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::{Character, Class, Item, ItemSlot, Race, Rarity};

    fn make_state_with_items(items: Vec<Item>) -> state::GameState {
        let mut s =
            state::GameState::new(Character::new("T".to_string(), Class::Rogue, Race::Human));
        s.character.inventory = items;
        s
    }

    fn item(name: &str) -> Item {
        Item {
            name: name.to_string(),
            slot: ItemSlot::Weapon,
            power: 1,
            rarity: Rarity::Common,
            enchant_level: 0,
        }
    }

    #[test]
    fn sq_debug_enabled_when_env_is_present() {
        assert!(telemetry::sq_debug_value_enabled(Some("")));
        assert!(telemetry::sq_debug_value_enabled(Some("1")));
        assert!(!telemetry::sq_debug_value_enabled(None));
    }

    #[test]
    fn fuzzy_match_two_tokens_both_present() {
        assert!(fuzzy_match_name("Big Sword of Awesome", "big of"));
    }

    #[test]
    fn fuzzy_match_partial_word_token() {
        assert!(fuzzy_match_name("Big Sword of Awesome", "big sw"));
    }

    #[test]
    fn fuzzy_match_case_insensitive() {
        assert!(fuzzy_match_name("Big Sword of Awesome", "BIG SWORD"));
    }

    #[test]
    fn fuzzy_match_single_token_prefix() {
        assert!(fuzzy_match_name("Big Sword of Awesome", "awe"));
    }

    #[test]
    fn fuzzy_match_full_name_exact() {
        assert!(fuzzy_match_name(
            "Big Sword of Awesome",
            "Big Sword of Awesome"
        ));
    }

    #[test]
    fn fuzzy_match_token_missing_returns_false() {
        assert!(!fuzzy_match_name("Big Sword of Awesome", "xyz"));
    }

    #[test]
    fn fuzzy_match_one_token_absent_returns_false() {
        assert!(!fuzzy_match_name("Big Sword of Awesome", "big xyz"));
    }

    #[test]
    fn fuzzy_match_empty_query_returns_true() {
        assert!(fuzzy_match_name("Big Sword of Awesome", ""));
    }

    #[test]
    fn find_inventory_item_exact_match() {
        let state = make_state_with_items(vec![item("Big Sword of Awesome")]);
        assert_eq!(
            find_inventory_item(&state, "Big Sword of Awesome"),
            Ok(Some(0))
        );
    }

    #[test]
    fn find_inventory_item_case_insensitive_exact() {
        let state = make_state_with_items(vec![item("Big Sword of Awesome")]);
        assert_eq!(
            find_inventory_item(&state, "big sword of awesome"),
            Ok(Some(0))
        );
    }

    #[test]
    fn find_inventory_item_substring_match() {
        let state = make_state_with_items(vec![item("Big Sword of Awesome")]);
        assert_eq!(find_inventory_item(&state, "big sw"), Ok(Some(0)));
    }

    #[test]
    fn find_inventory_item_fuzzy_non_contiguous_tokens() {
        let state = make_state_with_items(vec![item("Big Sword of Awesome")]);
        assert_eq!(find_inventory_item(&state, "big of"), Ok(Some(0)));
    }

    #[test]
    fn find_inventory_item_fuzzy_case_insensitive() {
        let state = make_state_with_items(vec![item("Big Sword of Awesome")]);
        assert_eq!(find_inventory_item(&state, "BIG OF"), Ok(Some(0)));
    }

    #[test]
    fn find_inventory_item_no_match_returns_none() {
        let state = make_state_with_items(vec![item("Big Sword of Awesome")]);
        assert_eq!(find_inventory_item(&state, "hammer"), Ok(None));
    }

    #[test]
    fn find_inventory_item_exact_wins_over_fuzzy() {
        let state = make_state_with_items(vec![item("Small Shield"), item("Big Sword of Awesome")]);
        assert_eq!(
            find_inventory_item(&state, "Big Sword of Awesome"),
            Ok(Some(1))
        );
    }

    #[test]
    fn find_inventory_item_fuzzy_picks_first_among_multiple() {
        let state = make_state_with_items(vec![
            item("Big Dagger of Doom"),
            item("Big Sword of Awesome"),
        ]);
        assert_eq!(find_inventory_item(&state, "big of"), Ok(Some(0)));
    }

    #[test]
    fn find_all_empty_inventory_returns_empty() {
        let state = make_state_with_items(vec![]);
        assert_eq!(find_inventory_items(&state, "potion"), Vec::<usize>::new());
    }

    #[test]
    fn find_all_single_word_matches_substring() {
        let state = make_state_with_items(vec![
            item("Potion of Coffee"),
            item("Rusty Pipe"),
            item("Potion of Sorrow"),
        ]);
        assert_eq!(find_inventory_items(&state, "potion"), vec![0, 2]);
    }

    #[test]
    fn find_all_multi_token_requires_all_tokens() {
        let state = make_state_with_items(vec![
            item("Big Sword of Awesome"),
            item("Small Dagger"),
            item("Big Shield"),
        ]);
        assert_eq!(find_inventory_items(&state, "big sword"), vec![0]);
    }

    #[test]
    fn find_all_case_insensitive() {
        let state = make_state_with_items(vec![item("Potion of Coffee"), item("Rusty Pipe")]);
        assert_eq!(find_inventory_items(&state, "POTION"), vec![0]);
    }

    #[test]
    fn find_all_no_match_returns_empty() {
        let state = make_state_with_items(vec![item("Rusty Pipe")]);
        assert_eq!(find_inventory_items(&state, "xyz"), Vec::<usize>::new());
    }

    #[test]
    fn find_all_exact_and_partial_both_included_in_order() {
        let state = make_state_with_items(vec![
            item("Rusty Pipe"),
            item("Pipewright Gauntlets"),
            item("Sword"),
        ]);
        let result = find_inventory_items(&state, "pipe");
        assert_eq!(result, vec![0, 1]);
    }

    #[test]
    fn find_all_returns_stable_inventory_order() {
        let state = make_state_with_items(vec![
            item("Potion of Sorrow"),
            item("Potion of Coffee"),
            item("Rusty Pipe"),
        ]);
        let result = find_inventory_items(&state, "potion");
        assert_eq!(result, vec![0, 1]);
    }

    #[test]
    fn selector_no_suffix_returns_first_match() {
        let state = make_state_with_items(vec![item("Potion of Coffee"), item("Potion of Sorrow")]);
        assert_eq!(find_inventory_item(&state, "potion"), Ok(Some(0)));
    }

    #[test]
    fn selector_explicit_dot_one_returns_first_match() {
        let state = make_state_with_items(vec![item("Potion of Coffee"), item("Potion of Sorrow")]);
        assert_eq!(find_inventory_item(&state, "potion.1"), Ok(Some(0)));
    }

    #[test]
    fn selector_dot_two_returns_second_match() {
        let state = make_state_with_items(vec![item("Potion of Coffee"), item("Potion of Sorrow")]);
        assert_eq!(find_inventory_item(&state, "potion.2"), Ok(Some(1)));
    }

    #[test]
    fn selector_dot_zero_returns_err() {
        let state = make_state_with_items(vec![item("Potion of Coffee")]);
        let result = find_inventory_item(&state, "potion.0");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("1 or higher"));
    }

    #[test]
    fn selector_n_exceeds_match_count_returns_err() {
        let state = make_state_with_items(vec![item("Potion of Coffee"), item("Potion of Sorrow")]);
        let result = find_inventory_item(&state, "potion.5");
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("Only 2"), "expected 'Only 2' in: {msg}");
        assert!(msg.contains("potion"), "expected query name in: {msg}");
    }

    #[test]
    fn selector_non_numeric_suffix_treated_as_query() {
        let state = make_state_with_items(vec![item("Potion of Coffee")]);
        assert_eq!(find_inventory_item(&state, "Potion.of.Coffee"), Ok(None));
    }

    #[test]
    fn selector_no_match_no_suffix_returns_ok_none() {
        let state = make_state_with_items(vec![item("Rusty Pipe")]);
        assert_eq!(find_inventory_item(&state, "xyz"), Ok(None));
    }

    #[test]
    fn selector_no_match_with_valid_suffix_returns_ok_none() {
        let state = make_state_with_items(vec![item("Rusty Pipe")]);
        assert_eq!(find_inventory_item(&state, "xyz.2"), Ok(None));
    }

    #[test]
    fn selector_dot_n_on_exact_match_works() {
        let state = make_state_with_items(vec![item("Rusty Pipe"), item("Rusty Sword")]);
        assert_eq!(find_inventory_item(&state, "rusty.2"), Ok(Some(1)));
    }

    #[test]
    fn parser_help_no_topic_yields_help_variant() {
        let cli = Cli::try_parse_from(["sq", "help"]).expect("'sq help' must parse");
        match cli.command {
            Commands::Help { topic } => assert!(
                topic.is_none(),
                "'sq help' must parse with topic == None, got {:?}",
                topic
            ),
            _ => panic!("expected Commands::Help, got a different variant"),
        }
    }

    #[test]
    fn parser_help_with_arena_topic_yields_help_variant() {
        let cli = Cli::try_parse_from(["sq", "help", "arena"]).expect("'sq help arena' must parse");
        match cli.command {
            Commands::Help { topic } => assert_eq!(
                topic.as_deref(),
                Some("arena"),
                "'sq help arena' must carry topic Some(\"arena\")"
            ),
            _ => panic!("expected Commands::Help"),
        }
    }

    #[test]
    fn parser_help_help_topic_yields_help_variant() {
        let cli = Cli::try_parse_from(["sq", "help", "help"]).expect("'sq help help' must parse");
        match cli.command {
            Commands::Help { topic } => assert_eq!(
                topic.as_deref(),
                Some("help"),
                "'sq help help' must carry topic Some(\"help\"), proving we own the help name"
            ),
            _ => panic!("expected Commands::Help"),
        }
    }

    #[test]
    fn parser_identify_canonical_yields_identify_variant() {
        let cli = Cli::try_parse_from(["sq", "identify", "scythe"])
            .expect("'sq identify scythe' must parse");
        match cli.command {
            Commands::Identify { name } => {
                assert_eq!(name, vec!["scythe".to_string()]);
            }
            _ => panic!("expected Commands::Identify"),
        }
    }

    #[test]
    fn parser_id_alias_routes_to_identify_variant() {
        let cli = Cli::try_parse_from(["sq", "id", "ring", "of", "fortune"])
            .expect("'sq id ring of fortune' must parse via the `id` alias");
        match cli.command {
            Commands::Identify { name } => {
                assert_eq!(
                    name,
                    vec!["ring".to_string(), "of".to_string(), "fortune".to_string()]
                );
            }
            _ => panic!("expected Commands::Identify via alias"),
        }
    }

    #[test]
    fn parser_quest_answer_routes_phrase_to_quest_subcommand() {
        let cli = Cli::try_parse_from(["sq", "quest", "answer", "ashen", "root", "sigil"])
            .expect("'sq quest answer <phrase>' must parse");
        match cli.command {
            Commands::Quest {
                action: Some(QuestAction::Answer { phrase }),
            } => assert_eq!(phrase, vec!["ashen", "root", "sigil"]),
            _ => panic!("expected quest answer variant"),
        }
    }

    #[test]
    fn quest_answer_normalization_is_case_insensitive_and_whitespace_insensitive() {
        assert_eq!(
            normalize_quest_answer("  Ashen   Root\nSigil "),
            "ashen root sigil"
        );
    }

    #[test]
    fn quest_correct_answer_awards_once() {
        let mut state = make_state_with_items(vec![]);
        let now = chrono::DateTime::parse_from_rfc3339("2026-05-30T12:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        state.quest_refreshed = Some(now);
        state.quest_phrase = Some("ashen root sigil".to_string());
        state.quest_completed_today = false;
        state.character.gold = 10;
        let reward = item_full("Rare Quest Blade", 12, Rarity::Rare);

        let first = apply_quest_answer(&mut state, " ASHEN   root sigil ", now, reward.clone());
        assert_eq!(first.status, QuestAnswerStatus::Correct);
        assert!(state.quest_completed_today);
        assert!(state.character.gold > 10);
        assert!(state
            .character
            .inventory
            .iter()
            .any(|i| i.name == reward.name));

        let gold_after_first = state.character.gold;
        let inventory_after_first = state.character.inventory.len();
        let second = apply_quest_answer(&mut state, "ashen root sigil", now, reward);
        assert_eq!(second.status, QuestAnswerStatus::AlreadyCompleted);
        assert_eq!(state.character.gold, gold_after_first);
        assert_eq!(state.character.inventory.len(), inventory_after_first);
    }

    #[test]
    fn quest_wrong_answer_does_not_penalize_or_complete() {
        let mut state = make_state_with_items(vec![]);
        let now = chrono::DateTime::parse_from_rfc3339("2026-05-30T12:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        state.quest_refreshed = Some(now);
        state.quest_phrase = Some("ashen root sigil".to_string());
        state.character.gold = 25;
        let reward = item_full("Rare Quest Blade", 12, Rarity::Rare);

        let result = apply_quest_answer(&mut state, "wrong words", now, reward);
        assert_eq!(result.status, QuestAnswerStatus::Wrong);
        assert!(!state.quest_completed_today);
        assert_eq!(state.character.gold, 25);
        assert!(state.character.inventory.is_empty());
    }

    #[test]
    fn quest_refresh_keeps_same_day_and_replaces_after_utc_midnight() {
        let mut state = make_state_with_items(vec![]);
        let day_one = chrono::DateTime::parse_from_rfc3339("2026-05-30T23:59:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let same_day = chrono::DateTime::parse_from_rfc3339("2026-05-30T23:59:59Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let day_two = chrono::DateTime::parse_from_rfc3339("2026-05-31T00:00:01Z")
            .unwrap()
            .with_timezone(&chrono::Utc);

        assert!(refresh_quest_state_if_needed(
            &mut state,
            day_one,
            "first phrase".to_string(),
            None,
        ));
        assert_eq!(state.quest_phrase.as_deref(), Some("first phrase"));
        state.quest_completed_today = true;

        assert!(!refresh_quest_state_if_needed(
            &mut state,
            same_day,
            "same-day phrase".to_string(),
            None,
        ));
        assert_eq!(state.quest_phrase.as_deref(), Some("first phrase"));
        assert!(state.quest_completed_today);

        assert!(refresh_quest_state_if_needed(
            &mut state,
            day_two,
            "second phrase".to_string(),
            None,
        ));
        assert_eq!(state.quest_phrase.as_deref(), Some("second phrase"));
        assert!(!state.quest_completed_today);
    }

    #[test]
    fn help_topic_identify_resolves_canonical_and_id_alias() {
        use help::{lookup_topic, LookupResult};

        match lookup_topic("identify") {
            LookupResult::Found(t) => assert_eq!(t.name, "identify"),
            other => panic!("expected Found(identify), got {:?}", other),
        }
        match lookup_topic("id") {
            LookupResult::Found(t) => assert_eq!(
                t.name, "identify",
                "alias 'id' must resolve to canonical 'identify'"
            ),
            other => panic!("expected Found(identify) via 'id', got {:?}", other),
        }
    }

    #[test]
    fn parser_dash_dash_help_still_routes_to_clap() {
        let err = match Cli::try_parse_from(["sq", "--help"]) {
            Ok(_) => panic!("'sq --help' must short-circuit on clap's DisplayHelp path, not parse"),
            Err(e) => e,
        };
        assert_eq!(
            err.kind(),
            clap::error::ErrorKind::DisplayHelp,
            "--help must trigger clap's auto-generated help, not our custom Help variant"
        );
    }

    #[test]
    fn format_help_no_topic_emits_index() {
        colored::control::set_override(false);
        let out = format_help(None);
        assert!(
            out.contains("sq Manual"),
            "index header missing from no-topic output:\n{}",
            out
        );
        assert!(
            out.contains(help::INDEX_FOOTER),
            "index footer '{}' missing:\n{}",
            help::INDEX_FOOTER,
            out
        );
    }

    #[test]
    fn format_help_arena_emits_authored_topic() {
        colored::control::set_override(false);
        let out = format_help(Some("arena"));
        assert!(
            out.contains("sq arena"),
            "topic header missing from arena help:\n{}",
            out
        );
        assert!(
            out.contains("Usage:"),
            "Usage section missing from arena help:\n{}",
            out
        );
        assert!(
            out.contains("Examples:"),
            "Examples section missing from arena help:\n{}",
            out
        );
    }

    #[test]
    fn format_help_help_emits_authored_topic_not_clap_help() {
        colored::control::set_override(false);
        let out = format_help(Some("help"));
        assert!(
            out.contains("sq help"),
            "header 'sq help' missing — our Help variant must own the topic:\n{}",
            out
        );
        assert!(
            out.contains("Browse the in-game manual"),
            "authored summary missing — output looks like clap's auto-help instead of our topic:\n{}",
            out
        );
    }

    #[test]
    fn format_help_typo_emits_no_match_with_suggestion() {
        colored::control::set_override(false);
        let out = format_help(Some("jounral"));
        assert!(
            out.contains("jounral"),
            "no-match output must echo the misspelled query:\n{}",
            out
        );
        assert!(
            out.contains("journal"),
            "no-match output must surface 'journal' as a close suggestion:\n{}",
            out
        );
    }

    fn item_full(name: &str, power: i32, rarity: Rarity) -> Item {
        Item {
            name: name.to_string(),
            slot: ItemSlot::Weapon,
            power,
            rarity,
            enchant_level: 0,
        }
    }

    #[test]
    fn name_matches_and_dot_n_follow_the_listed_order() {
        // Stored worst-first; `sq inventory` lists the Legendary first.
        let state = make_state_with_items(vec![
            item_full("Old Blade", 4, Rarity::Common),
            item_full("Kernel Blade", 40, Rarity::Legendary),
        ]);
        assert_eq!(find_inventory_items(&state, "blade"), vec![1, 0]);
        assert_eq!(find_inventory_item(&state, "blade"), Ok(Some(1)));
        assert_eq!(find_inventory_item(&state, "blade.2"), Ok(Some(0)));
    }

    #[test]
    fn sweep_junk_empty_inventory_returns_empty_result() {
        let r = sweep_junk(vec![]);
        assert_eq!(r.sold_count, 0);
        assert_eq!(r.total_price, 0);
        assert!(r.kept.is_empty());
    }

    #[test]
    fn sweep_junk_preserves_order_of_kept_items_and_sums_prices() {
        let inv = vec![
            item_full("Common Stick", 2, Rarity::Common),
            item_full("Rare One", 10, Rarity::Rare),
            item_full("Uncommon Tonic", 5, Rarity::Uncommon),
            item_full("Epic Blade", 25, Rarity::Epic),
            item_full("Rare Two", 12, Rarity::Rare),
        ];
        let expected_price = loot::item_price(&inv[0]) / 2 + loot::item_price(&inv[2]) / 2;
        let r = sweep_junk(inv);
        assert_eq!(r.sold_count, 2);
        assert_eq!(r.total_price, expected_price);
        let kept_names: Vec<&str> = r.kept.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(kept_names, vec!["Rare One", "Epic Blade", "Rare Two"]);
    }

    #[test]
    fn sweep_junk_never_sells_epic_or_legendary() {
        let epic = item_full("Doombringer", 20, Rarity::Epic);
        let legendary = item_full("Worldslayer", 50, Rarity::Legendary);
        let r = sweep_junk(vec![epic, legendary]);
        assert_eq!(r.sold_count, 0);
        assert_eq!(r.total_price, 0);
        assert_eq!(r.kept.len(), 2);
        let names: Vec<&str> = r.kept.iter().map(|i| i.name.as_str()).collect();
        assert!(names.contains(&"Doombringer"));
        assert!(names.contains(&"Worldslayer"));
    }

    #[test]
    fn sweep_junk_sells_an_uncommon_item_too() {
        let item = item_full("Decent Mace", 6, Rarity::Uncommon);
        let expected_price = loot::item_price(&item) / 2;
        let r = sweep_junk(vec![item]);
        assert_eq!(r.sold_count, 1);
        assert!(r.kept.is_empty());
        assert_eq!(r.total_price, expected_price);
    }

    #[test]
    fn sweep_junk_keeps_a_single_rare_item() {
        let rare = item_full("Rare Blade", 10, Rarity::Rare);
        let r = sweep_junk(vec![rare]);
        assert_eq!(r.sold_count, 0);
        assert_eq!(r.total_price, 0);
        assert_eq!(r.kept.len(), 1);
        assert_eq!(r.kept[0].name, "Rare Blade");
    }

    #[test]
    fn sweep_junk_sells_a_single_common_item() {
        let item = item_full("Rusty Spoon", 4, Rarity::Common);
        let expected_price = loot::item_price(&item) / 2;
        let r = sweep_junk(vec![item]);
        assert_eq!(r.sold_count, 1);
        assert!(r.kept.is_empty());
        assert_eq!(r.total_price, expected_price);
    }

    fn state_with_loadout(
        weapon: Option<Item>,
        armor: Option<Item>,
        ring: Option<Item>,
        inventory: Vec<Item>,
    ) -> state::GameState {
        let mut s = make_state_with_items(inventory);
        s.character.weapon = weapon;
        s.character.armor = armor;
        s.character.ring = ring;
        s
    }

    #[test]
    fn identify_lookup_matches_equipped_weapon_by_exact_name() {
        let s = state_with_loadout(Some(item("Scythe of Segfault")), None, None, vec![]);
        assert_eq!(
            find_inventory_or_equipped_item(&s, "Scythe of Segfault"),
            Ok(Some(ItemLookup::Equipped(EquippedSlot::Weapon))),
        );
    }

    #[test]
    fn identify_lookup_matches_equipped_armor_by_substring() {
        let s = state_with_loadout(None, Some(item("Leather Cuirass")), None, vec![]);
        assert_eq!(
            find_inventory_or_equipped_item(&s, "leather"),
            Ok(Some(ItemLookup::Equipped(EquippedSlot::Armor))),
        );
    }

    #[test]
    fn identify_lookup_matches_equipped_ring_via_fuzzy_tokens() {
        let s = state_with_loadout(None, None, Some(item("Ring of Fortune")), vec![]);
        assert_eq!(
            find_inventory_or_equipped_item(&s, "ring fortune"),
            Ok(Some(ItemLookup::Equipped(EquippedSlot::Ring))),
        );
    }

    #[test]
    fn identify_lookup_matches_inventory_item_by_substring() {
        let s = make_state_with_items(vec![item("Rusty Pipe")]);
        assert_eq!(
            find_inventory_or_equipped_item(&s, "rusty"),
            Ok(Some(ItemLookup::Inventory(0))),
        );
    }

    #[test]
    fn identify_lookup_matches_inventory_via_fuzzy_tokens() {
        let s = make_state_with_items(vec![item("Big Sword of Awesome")]);
        assert_eq!(
            find_inventory_or_equipped_item(&s, "big of"),
            Ok(Some(ItemLookup::Inventory(0))),
        );
    }

    #[test]
    fn identify_lookup_returns_none_on_no_match() {
        let s = state_with_loadout(
            Some(item("Pike of Ping")),
            None,
            None,
            vec![item("Rusty Pipe")],
        );
        assert_eq!(find_inventory_or_equipped_item(&s, "hammer"), Ok(None),);
    }

    #[test]
    fn identify_lookup_equipped_wins_over_inventory_by_default() {
        let s = state_with_loadout(
            Some(item("Scythe of Segfault")),
            None,
            None,
            vec![item("Scythe of Segfault")],
        );
        assert_eq!(
            find_inventory_or_equipped_item(&s, "scythe"),
            Ok(Some(ItemLookup::Equipped(EquippedSlot::Weapon))),
        );
    }

    #[test]
    fn identify_lookup_dot_two_picks_inventory_after_equipped() {
        let s = state_with_loadout(
            Some(item("Scythe of Segfault")),
            None,
            None,
            vec![item("Scythe of Segfault")],
        );
        assert_eq!(
            find_inventory_or_equipped_item(&s, "scythe.2"),
            Ok(Some(ItemLookup::Inventory(0))),
        );
    }

    #[test]
    fn identify_lookup_dot_three_with_only_two_matches_returns_err() {
        let s = state_with_loadout(
            Some(item("Pike of Ping")),
            None,
            None,
            vec![item("Pike of Ping")],
        );
        let result = find_inventory_or_equipped_item(&s, "pike.3");
        assert!(result.is_err());
    }

    #[test]
    fn identify_lookup_dot_zero_returns_err() {
        let s = make_state_with_items(vec![item("Rusty Pipe")]);
        let result = find_inventory_or_equipped_item(&s, "rusty.0");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("1 or higher"));
    }

    #[test]
    fn identify_lookup_orders_equipped_before_inventory_across_slots() {
        let s = state_with_loadout(
            Some(item("Spike Mace")),
            Some(item("Spike Plate")),
            Some(item("Spike Ring")),
            vec![item("Spike Dagger")],
        );
        assert_eq!(
            find_inventory_or_equipped_item(&s, "spike"),
            Ok(Some(ItemLookup::Equipped(EquippedSlot::Weapon))),
        );
        assert_eq!(
            find_inventory_or_equipped_item(&s, "spike.2"),
            Ok(Some(ItemLookup::Equipped(EquippedSlot::Armor))),
        );
        assert_eq!(
            find_inventory_or_equipped_item(&s, "spike.3"),
            Ok(Some(ItemLookup::Equipped(EquippedSlot::Ring))),
        );
        assert_eq!(
            find_inventory_or_equipped_item(&s, "spike.4"),
            Ok(Some(ItemLookup::Inventory(0))),
        );
    }
}

fn cmd_equip(name: &str) {
    if name.is_empty() {
        eprintln!(
            "{} Usage: {} or {}",
            "❌".bold(),
            "sq equip <armor name>".cyan(),
            "sq equip <ring name>".cyan()
        );
        return;
    }

    let Some((lock, mut game)) = load_for_update() else {
        return;
    };

    let idx = match find_inventory_item(&game, name) {
        Ok(Some(i)) => i,
        Ok(None) => {
            println!(
                "{} No item matching {} in your inventory.",
                "⚠️".yellow(),
                format!("\"{}\"", name).white().bold()
            );
            return;
        }
        Err(msg) => {
            println!("{} {}", "⚠️".yellow(), msg);
            return;
        }
    };

    let item = &game.character.inventory[idx];
    match item.slot {
        character::ItemSlot::Weapon => {
            println!(
                "{} {} is a weapon. Use {} instead.",
                "⚠️".yellow(),
                item.name.cyan().bold(),
                "sq wield".cyan()
            );
            return;
        }
        character::ItemSlot::Potion => {
            println!(
                "{} {} is a potion and cannot be equipped.",
                "⚠️".yellow(),
                item.name.cyan().bold()
            );
            return;
        }
        character::ItemSlot::Armor | character::ItemSlot::Ring => {}
    }

    let item = game.character.inventory.remove(idx);
    let item_name = item.name.clone();
    let slot_name = format!("{}", item.slot);

    if let Some(old) = game.character.equip(item) {
        let old_name = old.name.clone();
        game.character.inventory.push(old);
        println!(
            "{} Equipped {}! (replaced {})",
            "🛡️".bold(),
            item_name.green().bold(),
            old_name.dimmed()
        );
    } else {
        println!(
            "{} Equipped {} in {} slot!",
            "🛡️".bold(),
            item_name.green().bold(),
            slot_name.cyan()
        );
    }

    if let Err(e) = state::save(&game, &lock) {
        eprintln!("{} Failed to save: {}", "❌".bold(), e.red());
    }
}

fn cmd_wield(name: &str) {
    if name.is_empty() {
        eprintln!("{} Usage: {}", "❌".bold(), "sq wield <weapon name>".cyan());
        return;
    }

    let Some((lock, mut game)) = load_for_update() else {
        return;
    };

    let idx = match find_inventory_item(&game, name) {
        Ok(Some(i)) => i,
        Ok(None) => {
            println!(
                "{} No item matching {} in your inventory.",
                "⚠️".yellow(),
                format!("\"{}\"", name).white().bold()
            );
            return;
        }
        Err(msg) => {
            println!("{} {}", "⚠️".yellow(), msg);
            return;
        }
    };

    let item = &game.character.inventory[idx];
    if item.slot != character::ItemSlot::Weapon {
        println!(
            "{} {} is not a weapon. Use {} to wear armor or rings.",
            "⚠️".yellow(),
            item.name.cyan().bold(),
            "sq equip".cyan()
        );
        return;
    }

    let item = game.character.inventory.remove(idx);
    let item_name = item.name.clone();

    if let Some(old) = game.character.equip(item) {
        let old_name = old.name.clone();
        game.character.inventory.push(old);
        println!(
            "{} Now wielding {}! (sheathed {})",
            "⚔️".bold(),
            item_name.green().bold(),
            old_name.dimmed()
        );
    } else {
        println!("{} Now wielding {}!", "⚔️".bold(), item_name.green().bold());
    }

    if let Err(e) = state::save(&game, &lock) {
        eprintln!("{} Failed to save: {}", "❌".bold(), e.red());
    }
}

fn cmd_remove(name: &str) {
    if name.is_empty() {
        eprintln!(
            "{} Usage: {} (or use slot keyword: weapon, armor, ring)",
            "❌".bold(),
            "sq remove <item name>".cyan()
        );
        return;
    }

    let Some((lock, mut game)) = load_for_update() else {
        return;
    };

    let query = name.to_lowercase();

    let slot = if fuzzy_match_name(game.character.weapon.as_ref().map_or("", |i| &i.name), name)
        || query == "weapon"
    {
        "weapon"
    } else if fuzzy_match_name(game.character.armor.as_ref().map_or("", |i| &i.name), name)
        || query == "armor"
        || query == "armour"
    {
        "armor"
    } else if fuzzy_match_name(game.character.ring.as_ref().map_or("", |i| &i.name), name)
        || query == "ring"
    {
        "ring"
    } else {
        let equipped: Vec<&str> = [
            game.character.weapon.as_ref().map(|i| i.name.as_str()),
            game.character.armor.as_ref().map(|i| i.name.as_str()),
            game.character.ring.as_ref().map(|i| i.name.as_str()),
        ]
        .iter()
        .filter_map(|x| *x)
        .collect();

        if equipped.is_empty() {
            println!(
                "{} Nothing equipped. Use {} to see your gear.",
                "⚠️".yellow(),
                "sq status".cyan()
            );
        } else {
            println!(
                "{} No equipped item matching {}. Equipped: {}",
                "⚠️".yellow(),
                format!("\"{}\"", name).white().bold(),
                equipped.join(", ").dimmed()
            );
        }
        return;
    };

    if game.character.inventory.len() >= 20 {
        println!(
            "{} Inventory full (20/20). Drop an item first with {}.",
            "⚠️".yellow(),
            "sq drop <name>".cyan()
        );
        return;
    }

    let item = match slot {
        "weapon" => game.character.weapon.take(),
        "armor" => game.character.armor.take(),
        "ring" => game.character.ring.take(),
        _ => None,
    };

    if let Some(item) = item {
        let item_name = item.name.clone();
        let slot_name = format!("{}", item.slot);
        game.character.inventory.push(item);
        println!(
            "{} Removed {} from {} slot → moved to inventory.",
            "📦".bold(),
            item_name.white().bold(),
            slot_name.dimmed()
        );
    }

    if let Err(e) = state::save(&game, &lock) {
        eprintln!("{} Failed to save: {}", "❌".bold(), e.red());
    }
}

fn cmd_flee() {
    let Some((lock, mut game)) = load_for_update() else {
        return;
    };
    let Some(flight) = boss::flee(&mut game) else {
        println!("{} There is no world boss to flee from.", "⚠️".yellow());
        return;
    };
    if let Err(e) = state::save(&game, &lock) {
        eprintln!("{} Failed to save: {}", "❌".bold(), e.red());
        return;
    }
    match flight {
        boss::Flight::Fled { boss, cost } => println!(
            "{} You flee from {}, dropping {} gold as you run. It won't follow you.",
            "🏃".bold(),
            boss.red().bold(),
            format!("{}", cost).yellow().bold()
        ),
        boss::Flight::AlreadyLeaving { boss } => println!(
            "{} {} had already lost interest in you. It leaves, and you keep your gold.",
            "👻".bold(),
            boss.red().bold()
        ),
    }
}

fn cmd_drink(name: &str) {
    if name.is_empty() {
        eprintln!("{} Usage: {}", "❌".bold(), "sq drink <potion name>".cyan());
        return;
    }

    let Some((lock, mut game)) = load_for_update() else {
        return;
    };

    let idx = match find_inventory_item(&game, name) {
        Ok(Some(i)) => i,
        Ok(None) => {
            println!(
                "{} No item matching {} in your inventory.",
                "⚠️".yellow(),
                format!("\"{}\"", name).white().bold()
            );
            return;
        }
        Err(msg) => {
            println!("{} {}", "⚠️".yellow(), msg);
            return;
        }
    };

    let item = &game.character.inventory[idx];
    if item.slot != character::ItemSlot::Potion {
        println!(
            "{} {} is not drinkable.",
            "⚠️".yellow(),
            item.name.cyan().bold()
        );
        return;
    }

    let item = game.character.inventory.remove(idx);
    let heal = character::potion_heal_amount(item.power, game.character.max_hp);
    let item_name = item.name.clone();
    game.character.heal(heal);

    println!(
        "{} You drink the {}! Restored {} HP. HP: {}/{}",
        "🧪".bold(),
        item_name.green().bold(),
        format!("+{}", heal).green().bold(),
        format!("{}", game.character.hp).white().bold(),
        game.character.max_hp
    );

    if let Err(e) = state::save(&game, &lock) {
        eprintln!("{} Failed to save: {}", "❌".bold(), e.red());
    }
}

fn cmd_drop_item(name: &str) {
    if name.is_empty() {
        eprintln!("{} Usage: {}", "❌".bold(), "sq drop <item name>".cyan());
        return;
    }

    let Some((lock, mut game)) = load_for_update() else {
        return;
    };

    let idx = match find_inventory_item(&game, name) {
        Ok(Some(i)) => i,
        Ok(None) => {
            println!(
                "{} No item matching {} in your inventory.",
                "⚠️".yellow(),
                format!("\"{}\"", name).white().bold()
            );
            return;
        }
        Err(msg) => {
            println!("{} {}", "⚠️".yellow(), msg);
            return;
        }
    };

    let item = game.character.inventory.remove(idx);
    println!(
        "{} Dropped {} forever. It vanishes into the void.",
        "🗑️".bold(),
        item.name.red().bold()
    );

    if let Err(e) = state::save(&game, &lock) {
        eprintln!("{} Failed to save: {}", "❌".bold(), e.red());
    }
}

fn cmd_reset() {
    let answer = prompt(&format!(
        "{} This will delete your character permanently! Are you sure? [y/N] ",
        "💀".red().bold()
    ));
    if answer.to_lowercase() == "y" {
        let _lock = match state::lock(state::COMMAND_LOCK_TIMEOUT) {
            Ok(lock) => lock,
            Err(e) => {
                eprintln!("{} {}", "❌".bold(), e.to_string().red());
                return;
            }
        };
        let path = state::save_path();
        if path.exists() {
            match state::delete_save() {
                Ok(()) => println!(
                    "{} Character deleted. Run {} to start over.",
                    "🗑️".bold(),
                    "sq init".cyan()
                ),
                Err(e) => eprintln!("{} Failed to delete: {}", "❌".bold(), e.to_string().red()),
            }
        } else {
            println!("{}", "No character found.".dimmed());
        }
    } else {
        println!("{}", "Cancelled.".dimmed());
    }
}

fn cmd_update() {
    use std::process::Command;

    println!();
    println!("{}", "⬆️  Updating shellquest...".bold().cyan());
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );

    // Try cargo install from crates.io first (simplest path)
    println!(
        "  {} Installing latest version from {}...",
        "📦".bold(),
        "crates.io".cyan()
    );

    let status = Command::new("cargo")
        .args(["install", "shellquest", "--force"])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!();
            println!(
                "{}",
                "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
            );
            println!(
                "{} {} Restart your shell or run {} to use the new version.",
                "✅".bold(),
                "Update complete!".green().bold(),
                "sq status".cyan()
            );
            println!(
                "{}",
                "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
            );
            println!();
        }
        Ok(_) => {
            eprintln!(
                "{} {} Try manually: {}",
                "❌".bold(),
                "Update failed.".red(),
                "cargo install shellquest --force".dimmed()
            );
        }
        Err(e) => {
            eprintln!(
                "{} Failed to run cargo: {}",
                "❌".bold(),
                e.to_string().red()
            );
            eprintln!(
                "  Make sure {} is installed: {}",
                "cargo".bold(),
                "https://rustup.rs".dimmed()
            );
        }
    }
}

fn cmd_arena(from_deprecated: bool) {
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        eprintln!("Arena requires an interactive terminal.");
        std::process::exit(1);
    }

    if from_deprecated {
        println!(
            "{}",
            "⚠️  The `tournament` command is deprecated. Use `sq arena` instead.".yellow()
        );
    }

    let mut game = match state::load() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{} {}", "❌".bold(), e.red());
            return;
        }
    };

    let tier = match select_arena_tier(&game.character) {
        Some(t) => t,
        None => return,
    };

    if !tier.is_unlocked(&game.character) {
        let req = if tier.or_unlock {
            format!(
                "Requires level {} or prestige {}.",
                tier.min_level, tier.min_prestige
            )
        } else {
            format!(
                "Requires level {} and prestige {}.",
                tier.min_level, tier.min_prestige
            )
        };
        println!(
            "{} {} is locked. {}",
            "🔒".bold(),
            tier.name.yellow().bold(),
            req
        );
        return;
    }

    let entry = arena::ArenaEntrySnapshot::from_character(&game.character);
    let fee = tier.compute_fee(&entry);

    if game.character.gold < fee {
        println!(
            "{} Not enough gold! {} entry fee is {} gold, you have {}.",
            "⚠️".yellow(),
            tier.name,
            format!("{}", fee).yellow().bold(),
            format!("{}", game.character.gold).yellow()
        );
        return;
    }

    let confirm = prompt(&format!(
        "{} Enter {} for {} gold? [y/N] ",
        "🏟️".bold(),
        tier.name.yellow().bold(),
        format!("{}", fee).yellow().bold()
    ));
    if confirm.to_lowercase() != "y" {
        println!("{}", "Cancelled.".dimmed());
        return;
    }

    match arena::run_arena_session(&game.character, tier, fee) {
        Some(commit) => {
            // T7 transaction boundary: apply mutates state and returns deferred
            // reward output (level-up, overflow, inventory replacement). Nothing
            // user-visible is rendered until `state::save()` succeeds, so a save
            // failure leaves no stale prints behind.
            let deferred = arena::apply_arena_commit(&mut game, &commit);
            // Locks only the final write; reloading to keep other shells' progress is x3p.2.
            let saved = state::lock(state::COMMAND_LOCK_TIMEOUT)
                .map_err(|e| e.to_string())
                .and_then(|lock| state::save(&game, &lock));
            if let Err(e) = saved {
                eprintln!("{} Failed to save arena results: {}", "❌".bold(), e.red());
                return;
            }
            arena::render_arena_deferred_output(&deferred);

            let (label, rounds) = match commit.outcome {
                arena::ArenaOutcome::Defeat { rounds_cleared } => ("Knocked out", rounds_cleared),
                arena::ArenaOutcome::CashOut { rounds_cleared } => ("Cashed out", rounds_cleared),
                arena::ArenaOutcome::Victory { rounds_cleared } => ("Victory", rounds_cleared),
            };
            println!("{} {} after {} rounds.", "🏁".bold(), label, rounds);
        }
        None => {
            println!("{}", "Arena run cancelled before round 1.".dimmed());
        }
    }
}

fn select_arena_tier(character: &character::Character) -> Option<arena::ArenaTier> {
    println!();
    println!("{}", "🏟️  Arena Tiers".bold().yellow());
    println!("{}", "─".repeat(50).dimmed());

    for (i, tier) in arena::ARENA_TIERS.iter().enumerate() {
        let unlocked = tier.is_unlocked(character);
        let status = if unlocked {
            "✓".green().bold()
        } else {
            "🔒".dimmed()
        };
        let name = if unlocked {
            tier.name.white().bold()
        } else {
            tier.name.dimmed()
        };
        let req = if tier.or_unlock {
            format!("lvl {} or prestige {}", tier.min_level, tier.min_prestige)
        } else {
            format!("lvl {} & prestige {}", tier.min_level, tier.min_prestige)
        };
        println!(
            "  {}. {} {} — {} rounds — {}",
            format!("{}", i + 1).dimmed(),
            status,
            name,
            tier.max_rounds,
            req.dimmed()
        );
    }

    println!("{}", "─".repeat(50).dimmed());

    loop {
        let choice = prompt("   Select tier [1-5] (or press Enter to cancel): ");
        if choice.is_empty() {
            return None;
        }
        match choice.as_str() {
            "1" => return Some(arena::TIER_PIT),
            "2" => return Some(arena::TIER_GAUNTLET),
            "3" => return Some(arena::TIER_COLOSSEUM),
            "4" => return Some(arena::TIER_ABYSSAL),
            "5" => return Some(arena::TIER_GODSLAYER),
            _ => println!("   Invalid choice. Pick 1-5 or press Enter to cancel."),
        }
    }
}

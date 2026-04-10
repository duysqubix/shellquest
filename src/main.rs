mod boss;
mod character;
mod display;
mod events;
mod journal;
mod loot;
mod sage;
mod state;
mod zones;

use character::{Class, Race};
use clap::{Parser, Subcommand};
use colored::*;
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "sq", version, about = "A passive RPG that lives in your terminal")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new character
    Init,
    /// View your character sheet
    Status,
    /// Check your inventory
    #[clap(alias = "inv")]
    Inventory,
    /// View your adventure journal
    Journal,
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
    Equip {
        /// Item name (or partial match)
        name: Vec<String>,
    },
    /// Wield a weapon from inventory
    Wield {
        /// Item name (or partial match)
        name: Vec<String>,
    },
    /// Drop an item from inventory permanently
    Drop {
        /// Item name (or partial match)
        name: Vec<String>,
    },
    /// Browse the shop (must be in home directory)
    Shop,
    /// Buy an item from the shop by number (see `sq shop` for numbered list)
    Buy {
        /// Item number from the shop list
        number: usize,
    },
    /// Drink a potion from inventory to restore HP
    Drink {
        /// Item name (or partial match)
        name: Vec<String>,
    },
    /// Prestige: reset to level 1 with a subclass and bonus stats
    Prestige,
    /// Reset your character (start over)
    Reset,
    /// Update sq to the latest version
    Update,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => cmd_init(),
        Commands::Status => cmd_status(),
        Commands::Inventory => cmd_inventory(),
        Commands::Journal => cmd_journal(),
        Commands::Tick {
            cmd,
            cwd,
            exit_code,
            test_sage,
        } => cmd_tick(&cmd, &cwd, exit_code, test_sage),
        Commands::Hook { shell, install, file } => cmd_hook(&shell, install || file.is_some(), file),
        Commands::Shop => cmd_shop(),
        Commands::Buy { number } => cmd_buy(number),
        Commands::Equip { name } => cmd_equip(&name.join(" ")),
        Commands::Wield { name } => cmd_wield(&name.join(" ")),
        Commands::Drop { name } => cmd_drop_item(&name.join(" ")),
        Commands::Drink { name } => cmd_drink(&name.join(" ")),
        Commands::Prestige => cmd_prestige(),
        Commands::Reset => cmd_reset(),
        Commands::Update => cmd_update(),
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
    if state::save_path().exists() {
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
    let game_state = state::GameState::new(character);

    match state::save(&game_state) {
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
            println!("  Run {} to install the shell hook.", "sq hook --shell zsh".cyan());
            println!(
                "  Run {} to see your character.",
                "sq status".cyan()
            );
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
        Ok(game) => display::print_status(&game.character),
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

fn cmd_tick(cmd: &str, cwd: &str, exit_code: i32, test_sage: bool) {
    let mut game = match state::load() {
        Ok(g) => g,
        Err(_) => return, // Silently skip if no character
    };

    events::tick(&mut game, cmd, cwd, exit_code);
    if test_sage {
        sage::force_show_sage(&mut game);
    } else {
        sage::maybe_show_sage(&mut game);
    }
    game.last_tick = chrono::Utc::now();

    if let Err(e) = state::save(&game) {
        eprintln!("{} Failed to save: {}", "❌".bold(), e.red());
    }
}

fn hook_code(shell: &str) -> Option<String> {
    match shell {
        "bash" => Some(r#"
# shellquest (sq) — passive terminal RPG hook
__sq_hook() {
    local exit_code=$?
    local cmd=$(HISTTIMEFORMAT= history 1 | sed 's/^ *[0-9]* *//')
    sq tick --cmd "$cmd" --cwd "$PWD" --exit-code "$exit_code"
}
PROMPT_COMMAND="__sq_hook;$PROMPT_COMMAND"
"#.to_string()),
        "zsh" => Some(r#"
# shellquest (sq) — passive terminal RPG hook
__sq_hook() {
    local exit_code=$?
    local cmd=$(fc -ln -1)
    sq tick --cmd "$cmd" --cwd "$PWD" --exit-code "$exit_code"
}
precmd_functions+=(__sq_hook)
"#.to_string()),
        "fish" => Some(r#"
# shellquest (sq) — passive terminal RPG hook
function __sq_hook --on-event fish_postexec
    set -l cmd $argv[1]
    set -l exit_code $status
    sq tick --cmd "$cmd" --cwd "$PWD" --exit-code "$exit_code"
end
"#.to_string()),
        _ => None,
    }
}

fn default_rc_file(shell: &str) -> Option<String> {
    let home = dirs::home_dir()?;
    match shell {
        "bash" => Some(home.join(".bashrc").to_string_lossy().to_string()),
        "zsh" => Some(home.join(".zshrc").to_string_lossy().to_string()),
        "fish" => Some(home.join(".config/fish/config.fish").to_string_lossy().to_string()),
        _ => None,
    }
}

fn cmd_hook(shell: &str, install: bool, file: Option<String>) {
    let code = match hook_code(shell) {
        Some(c) => c,
        None => {
            eprintln!(
                "{} Unknown shell: {}. Supported: bash, zsh, fish",
                "❌".bold(),
                shell.red()
            );
            return;
        }
    };

    if !install {
        // Just print the hook code
        print!("{}", code);
        return;
    }

    // Install mode: write to file
    let target = file.or_else(|| default_rc_file(shell));
    let target = match target {
        Some(t) => t,
        None => {
            eprintln!("{} Could not determine rc file for shell: {}", "❌".bold(), shell.red());
            return;
        }
    };

    // Check if hook already installed
    if let Ok(contents) = std::fs::read_to_string(&target) {
        if contents.contains("__sq_hook") {
            println!(
                "{} Hook already installed in {}",
                "✓".green().bold(),
                target.cyan()
            );
            return;
        }
    }

    // Append hook
    use std::fs::OpenOptions;
    match OpenOptions::new().create(true).append(true).open(&target) {
        Ok(mut f) => {
            use std::io::Write;
            if let Err(e) = f.write_all(code.as_bytes()) {
                eprintln!("{} Failed to write hook: {}", "❌".bold(), e.to_string().red());
                return;
            }
            println!(
                "{} Hook installed to {}",
                "✓".green().bold(),
                target.cyan()
            );
            println!(
                "  Run {} or restart your terminal to activate.",
                format!("source {}", target).dimmed()
            );
        }
        Err(e) => {
            eprintln!("{} Failed to open {}: {}", "❌".bold(), target, e.to_string().red());
        }
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
    println!(
        "{}",
        "✨ PRESTIGE ✨"
            .yellow()
            .bold()
            .on_black()
    );
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
        let choice = prompt(&format!("{} Choose [1-{}]: ", "🎭".bold(), subclasses.len()));
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

    match state::save(&game) {
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
        println!(
            "  Run {} to return home first.",
            "cd ~".cyan()
        );
        return;
    }

    let mut game = match state::load() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{} {}", "❌".bold(), e.red());
            return;
        }
    };

    refresh_shop_if_needed(&mut game);

    println!();
    println!(
        "{}",
        "🏪 The Terminal Bazaar".bold().yellow()
    );
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
    println!(
        "  Use {} to purchase an item.",
        "sq buy <number>".cyan()
    );
    println!(
        "  Shop refreshes daily at {}.",
        "midnight UTC".dimmed()
    );
    println!();

    if let Err(e) = state::save(&game) {
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

    let mut game = match state::load() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{} {}", "❌".bold(), e.red());
            return;
        }
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

    if let Err(e) = state::save(&game) {
        eprintln!("{} Failed to save: {}", "❌".bold(), e.red());
    }
}

fn fuzzy_match_name(item_name: &str, query: &str) -> bool {
    let name_lower = item_name.to_lowercase();
    query
        .to_lowercase()
        .split_whitespace()
        .all(|token| name_lower.contains(token))
}

fn find_inventory_items(game: &state::GameState, query: &str) -> Vec<usize> {
    let query_lower = query.to_lowercase();
    let inv = &game.character.inventory;
    let mut matched: Vec<usize> = (0..inv.len())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::{Character, Class, Item, ItemSlot, Race, Rarity};

    fn make_state_with_items(items: Vec<Item>) -> state::GameState {
        let mut s = state::GameState::new(Character::new("T".to_string(), Class::Rogue, Race::Human));
        s.character.inventory = items;
        s
    }

    fn item(name: &str) -> Item {
        Item { name: name.to_string(), slot: ItemSlot::Weapon, power: 1, rarity: Rarity::Common }
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
        assert!(fuzzy_match_name("Big Sword of Awesome", "Big Sword of Awesome"));
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
        assert_eq!(find_inventory_item(&state, "Big Sword of Awesome"), Ok(Some(0)));
    }

    #[test]
    fn find_inventory_item_case_insensitive_exact() {
        let state = make_state_with_items(vec![item("Big Sword of Awesome")]);
        assert_eq!(find_inventory_item(&state, "big sword of awesome"), Ok(Some(0)));
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
        let state = make_state_with_items(vec![
            item("Small Shield"),
            item("Big Sword of Awesome"),
        ]);
        assert_eq!(find_inventory_item(&state, "Big Sword of Awesome"), Ok(Some(1)));
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

    let mut game = match state::load() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{} {}", "❌".bold(), e.red());
            return;
        }
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

    if let Err(e) = state::save(&game) {
        eprintln!("{} Failed to save: {}", "❌".bold(), e.red());
    }
}

fn cmd_wield(name: &str) {
    if name.is_empty() {
        eprintln!(
            "{} Usage: {}",
            "❌".bold(),
            "sq wield <weapon name>".cyan()
        );
        return;
    }

    let mut game = match state::load() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{} {}", "❌".bold(), e.red());
            return;
        }
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
        println!(
            "{} Now wielding {}!",
            "⚔️".bold(),
            item_name.green().bold()
        );
    }

    if let Err(e) = state::save(&game) {
        eprintln!("{} Failed to save: {}", "❌".bold(), e.red());
    }
}

fn cmd_drink(name: &str) {
    if name.is_empty() {
        eprintln!(
            "{} Usage: {}",
            "❌".bold(),
            "sq drink <potion name>".cyan()
        );
        return;
    }

    let mut game = match state::load() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{} {}", "❌".bold(), e.red());
            return;
        }
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
    let heal = item.power;
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

    if let Err(e) = state::save(&game) {
        eprintln!("{} Failed to save: {}", "❌".bold(), e.red());
    }
}

fn cmd_drop_item(name: &str) {
    if name.is_empty() {
        eprintln!(
            "{} Usage: {}",
            "❌".bold(),
            "sq drop <item name>".cyan()
        );
        return;
    }

    let mut game = match state::load() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{} {}", "❌".bold(), e.red());
            return;
        }
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

    if let Err(e) = state::save(&game) {
        eprintln!("{} Failed to save: {}", "❌".bold(), e.red());
    }
}

fn cmd_reset() {
    let answer = prompt(&format!(
        "{} This will delete your character permanently! Are you sure? [y/N] ",
        "💀".red().bold()
    ));
    if answer.to_lowercase() == "y" {
        let path = state::save_path();
        if path.exists() {
            match std::fs::remove_file(&path) {
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
    println!(
        "{}",
        "⬆️  Updating shellquest...".bold().cyan()
    );
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

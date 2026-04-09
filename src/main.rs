mod character;
mod display;
mod events;
mod journal;
mod loot;
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
    /// Prestige: reset to level 1 with a subclass and bonus stats
    Prestige,
    /// Reset your character (start over)
    Reset,
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
        } => cmd_tick(&cmd, &cwd, exit_code),
        Commands::Hook { shell, install, file } => cmd_hook(&shell, install || file.is_some(), file),
        Commands::Prestige => cmd_prestige(),
        Commands::Reset => cmd_reset(),
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

fn cmd_tick(cmd: &str, cwd: &str, exit_code: i32) {
    let mut game = match state::load() {
        Ok(g) => g,
        Err(_) => return, // Silently skip if no character
    };

    events::tick(&mut game, cmd, cwd, exit_code);
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

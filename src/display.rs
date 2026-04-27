use crate::character::{Character, Rarity};
use crate::journal::{EventType, JournalEntry};
use crate::zones::Zone;
use colored::*;

// ── Rich inline color helpers (MUD-style) ──

pub fn color_damage(n: i32) -> String { format!("{}", format!("{}", n).red().bold()) }
pub fn color_xp(n: u32) -> String { format!("+{} {}", format!("{}", n).cyan().bold(), "XP".cyan()) }
pub fn color_gold(n: u32) -> String { format!("+{} {}", format!("{}", n).yellow().bold(), "gold".yellow()) }
pub fn color_hp(hp: i32, max_hp: i32) -> String {
    let pct = hp as f32 / max_hp as f32;
    let hp_str = format!("{}/{}", hp, max_hp);
    if pct > 0.6 { format!("{}: {}", "HP".bold(), hp_str.green()) }
    else if pct > 0.3 { format!("{}: {}", "HP".bold(), hp_str.yellow()) }
    else { format!("{}: {}", "HP".bold(), hp_str.red().bold()) }
}
pub fn color_monster(name: &str) -> String { format!("{}", name.red().bold()) }
pub fn color_item_inline(name: &str, rarity: &Rarity) -> String {
    match rarity {
        Rarity::Common => format!("{}", name.white()),
        Rarity::Uncommon => format!("{}", name.dimmed().bold()),
        Rarity::Rare => format!("{}", name.green().bold()),
        Rarity::Epic => format!("{}{}{}", "★".magenta(), name.magenta().bold(), "★".magenta()),
        Rarity::Legendary => format!("{}{}{}", "✦".yellow().bold(), name.yellow().bold().on_black(), "✦".yellow().bold()),
    }
}
pub fn color_zone(name: &str, zone: &Zone) -> String {
    use crate::zones::ZoneColor;
    match zone.color {
        ZoneColor::Green => format!("{}", name.green().bold()),
        ZoneColor::Yellow => format!("{}", name.yellow().bold()),
        ZoneColor::Red => format!("{}", name.red().bold()),
        ZoneColor::Blue => format!("{}", name.blue().bold()),
        ZoneColor::Magenta => format!("{}", name.magenta().bold()),
        ZoneColor::Cyan => format!("{}", name.cyan().bold()),
    }
}

// ── Print functions (accept pre-colored or plain strings) ──

pub fn print_combat_win(msg: &str) {
    eprintln!("{} {}", "⚔️ ".bold(), msg);
}

pub fn print_combat_tough(msg: &str, died: bool) {
    if died {
        eprintln!("{} {}", "💀".bold(), msg);
    } else {
        eprintln!("{} {}", "⚔️ ".bold(), msg);
    }
}

pub fn print_combat_lose(msg: &str, died: bool) {
    if died {
        eprintln!("{} {}", "💀".bold(), msg);
    } else {
        eprintln!("{} {}", "🩸".bold(), msg);
    }
}

pub fn print_combat_draw(msg: &str) {
    eprintln!("{} {}", "👻".bold(), msg.dimmed());
}

pub fn print_trap(msg: &str) {
    eprintln!("{} {}", "🪤".bold(), msg);
}

pub fn print_travel(msg: &str, _zone: &Zone) {
    eprintln!("{} {}", "🗺️ ".bold(), msg);
}

pub fn print_craft(msg: &str) {
    eprintln!("{} {}", "🔨".bold(), msg);
}

pub fn print_quest(msg: &str) {
    eprintln!("{} {}", "🏆".bold(), msg);
}

pub fn print_discovery(msg: &str) {
    eprintln!("{} {}", "🔮".bold(), msg);
}

pub fn print_loot(msg: &str, rarity: &Rarity) {
    match rarity {
        Rarity::Common => {
            eprintln!("{} {}", "📦".bold(), msg.white());
        }
        Rarity::Uncommon => {
            eprintln!("{} {} {}", "📦".bold(), "~".dimmed(), msg.dimmed().bold());
        }
        Rarity::Rare => {
            eprintln!("{} {} {} {}", "📦".bold(), "~~".green().bold(), msg.green().bold(), "~~".green().bold());
        }
        Rarity::Epic => {
            eprintln!("{} {} {} {}", "💎".bold(), "★·.·".magenta(), msg.magenta().bold().italic(), "·.·★".magenta());
        }
        Rarity::Legendary => {
            eprintln!("{}", "╔═══════════════════════════════════════════╗".yellow().bold());
            eprintln!("{} {} {} {}", "║".yellow().bold(), "✦✦✦".yellow().bold().on_black(), msg.yellow().bold().on_black(), "✦✦✦".yellow().bold().on_black());
            eprintln!("{}", "╚═══════════════════════════════════════════╝".yellow().bold());
        }
    }
}

fn format_item_rarity(name: &str, rarity: &Rarity) -> (String, String) {
    match rarity {
        Rarity::Common => (
            name.white().to_string(),
            "[Common]".dimmed().to_string(),
        ),
        Rarity::Uncommon => (
            name.dimmed().bold().to_string(),
            "[Uncommon]".dimmed().to_string(),
        ),
        Rarity::Rare => (
            format!("{}", name.green().bold()),
            format!("{}", "[Rare]".green().bold()),
        ),
        Rarity::Epic => (
            format!("{}{}{}", "★ ".magenta(), name.magenta().bold().italic(), " ★".magenta()),
            format!("{}", "[Epic]".magenta().bold()),
        ),
        Rarity::Legendary => (
            format!("{}{}{}", "✦ ".yellow().bold(), name.yellow().bold().on_black(), " ✦".yellow().bold()),
            format!("{}", "[LEGENDARY]".yellow().bold().on_black()),
        ),
    }
}

pub fn print_gold(msg: &str) {
    eprintln!("{} {}", "💰".bold(), msg);
}

pub fn print_familiar(msg: &str) {
    eprintln!("{} {}", "🐾".bold(), msg);
}

pub fn print_portal(msg: &str) {
    eprintln!("{} {}", "🌀".bold(), msg);
}

pub fn print_power(msg: &str) {
    eprintln!("{} {}", "⚡".bold(), msg);
}

pub fn print_level_up(msg: &str) {
    eprintln!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".yellow().bold());
    eprintln!("{} {}", "🎉".bold(), msg);
    eprintln!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".yellow().bold());
}

pub fn print_status(char: &Character, permadeath: bool) {
    let class_colored = format!("{}", char.class).cyan().bold();
    let race_colored = format!("{}", char.race).magenta();

    println!();
    println!("{}", "┌──────────────────────────────────────────┐".dimmed());
    let subclass_str = char.subclass.as_ref().map_or(String::from(" "), |s| {
        format!(" {} ", format!("{}", s).magenta().bold())
    });
    let prestige_str = if char.prestige > 0 {
        format!(" [{}{}]", "P".yellow().bold(), format!("{}", char.prestige).yellow().bold())
    } else {
        String::new()
    };
    println!(
        "{}  {} {} {}{}{}  (Lvl {}{})",
        "│".dimmed(),
        char.name.bold().white(),
        "the".dimmed(),
        race_colored,
        subclass_str,
        class_colored,
        format!("{}", char.level).yellow().bold(),
        prestige_str
    );
    println!("{}", "│".dimmed());

    // HP bar
    let hp_pct = char.hp as f32 / char.max_hp as f32;
    let bar_len = 20;
    let filled = (hp_pct * bar_len as f32) as usize;
    let empty = bar_len - filled;
    let hp_color = if hp_pct > 0.6 {
        "green"
    } else if hp_pct > 0.3 {
        "yellow"
    } else {
        "red"
    };
    let bar = format!("{}{}",
        "█".repeat(filled),
        "░".repeat(empty)
    );
    let bar_colored = match hp_color {
        "green" => bar.green(),
        "yellow" => bar.yellow(),
        _ => bar.red(),
    };
    println!(
        "{}  {} {} {}/{}",
        "│".dimmed(),
        "HP:".bold(),
        bar_colored,
        format!("{}", char.hp).bold(),
        char.max_hp
    );

    // XP bar
    let xp_pct = char.xp as f32 / char.xp_to_next as f32;
    let xp_filled = ((xp_pct * bar_len as f32) as usize).min(bar_len);
    let xp_empty = bar_len - xp_filled;
    let xp_bar = format!("{}{}",
        "█".repeat(xp_filled),
        "░".repeat(xp_empty)
    );
    println!(
        "{}  {} {} {}/{}",
        "│".dimmed(),
        "XP:".bold(),
        xp_bar.cyan(),
        char.xp,
        char.xp_to_next
    );

    println!("{}", "│".dimmed());
    println!(
        "{}  {} {}  {} {}  {} {}",
        "│".dimmed(),
        "STR:".bold(),
        format!("{}", char.strength).red(),
        "DEX:".bold(),
        format!("{}", char.dexterity).green(),
        "INT:".bold(),
        format!("{}", char.intelligence).blue()
    );
    println!(
        "{}  {} {}",
        "│".dimmed(),
        "Gold:".bold(),
        format!("{}", char.gold).yellow()
    );
    println!("{}", "│".dimmed());

    // Equipment
    let weapon_str = char.weapon.as_ref().map_or("(none)".dimmed().to_string(), |w| {
        let (name, rarity) = format_item_rarity(&w.name, &w.rarity);
        format!("{} (+{}) {}", name, w.power, rarity)
    });
    let armor_str = char.armor.as_ref().map_or("(none)".dimmed().to_string(), |a| {
        let (name, rarity) = format_item_rarity(&a.name, &a.rarity);
        format!("{} (+{}) {}", name, a.power, rarity)
    });
    let ring_str = char.ring.as_ref().map_or("(none)".dimmed().to_string(), |r| {
        let (name, rarity) = format_item_rarity(&r.name, &r.rarity);
        format!("{} (+{}) {}", name, r.power, rarity)
    });

    println!("{}  {} {}", "│".dimmed(), "Weapon:".bold(), weapon_str);
    println!("{}  {} {}", "│".dimmed(), "Armor: ".bold(), armor_str);
    println!("{}  {} {}", "│".dimmed(), "Ring:  ".bold(), ring_str);
    println!("{}", "│".dimmed());

    println!(
        "{}  {} {}  {} {}  {} {}",
        "│".dimmed(),
        "Kills:".bold(),
        format!("{}", char.kills).green(),
        "Deaths:".bold(),
        format!("{}", char.deaths).red(),
        "Cmds:".bold(),
        format!("{}", char.commands_run).cyan()
    );
    println!(
        "{}  {} {}",
        "│".dimmed(),
        "Title:".bold(),
        char.title.yellow().italic()
    );
    if char.tournament_wins > 0 || char.best_tournament_round > 0 {
        println!(
            "{}  {} {}  {} {}",
            "│".dimmed(),
            "Arena Crowns:".bold(),
            format!("{}", char.tournament_wins).yellow().bold(),
            "Arena Best:".bold(),
            format!("{}", char.best_tournament_round).cyan().bold()
        );
    }
    if permadeath {
        println!("{}  {} {}", "│".dimmed(), "Mode:".bold(), "☠ PERMADEATH".red().bold());
    }
    println!("{}", "└──────────────────────────────────────────┘".dimmed());
    println!();
}

pub fn print_inventory(char: &Character) {
    println!();
    println!("{}", "📦 Inventory".bold().cyan());
    println!("{}", "─".repeat(40).dimmed());

    if char.inventory.is_empty() {
        println!("{}", "  (empty)".dimmed());
    } else {
        for (i, item) in char.inventory.iter().enumerate() {
            let (name_styled, rarity_styled) = format_item_rarity(&item.name, &item.rarity);
            println!(
                "  {}. {} (+{} {}) {}",
                format!("{}", i + 1).dimmed(),
                name_styled,
                item.power,
                format!("{}", item.slot).dimmed(),
                rarity_styled
            );
        }
    }
    println!();
}

pub fn print_journal(entries: &[JournalEntry]) {
    println!();
    println!("{}", "📜 Adventure Journal".bold().yellow());
    println!("{}", "─".repeat(50).dimmed());

    if entries.is_empty() {
        println!("{}", "  No entries yet. Go run some commands!".dimmed());
    } else {
        let recent: Vec<&JournalEntry> = entries.iter().rev().take(20).collect();
        for entry in recent.iter().rev() {
            let time = entry.timestamp.format("%m/%d %H:%M").to_string().dimmed();
            let icon = match entry.event_type {
                EventType::Combat => "⚔️ ",
                EventType::Loot => "📦",
                EventType::Travel => "🗺️ ",
                EventType::Discovery => "🔮",
                EventType::LevelUp => "🎉",
                EventType::Death => "💀",
                EventType::Quest => "🏆",
                EventType::Craft => "🔨",
                EventType::Tournament => "🏅",
            };
            let msg_colored = match entry.event_type {
                EventType::Combat => entry.message.white().to_string(),
                EventType::Loot => entry.message.green().to_string(),
                EventType::Travel => entry.message.cyan().to_string(),
                EventType::Discovery => entry.message.magenta().to_string(),
                EventType::LevelUp => entry.message.yellow().bold().to_string(),
                EventType::Death => entry.message.red().bold().to_string(),
                EventType::Quest => entry.message.yellow().to_string(),
                EventType::Craft => entry.message.cyan().to_string(),
                EventType::Tournament => entry.message.yellow().bold().to_string(),
            };
            println!("  {} {} {}", time, icon, msg_colored);
        }
    }
    println!();
}

pub fn print_boss_spawn(boss: &crate::boss::Boss) {
    eprintln!();
    eprintln!("{}", "╔══════════════════════════════════════════════╗".red().bold());
    eprintln!("{} {} {}",
        "║".red().bold(),
        format!("⚠️  WORLD BOSS: {} HAS APPEARED!", boss.name).red().bold(),
        "║".red().bold());
    eprintln!("{} {} {}",
        "║".red().bold(),
        format!("   HP: {}  ATK: {}  — Defeat it for legendary rewards!", boss.max_hp, boss.attack).red(),
        "║".red().bold());
    eprintln!("{}", "╚══════════════════════════════════════════════╝".red().bold());
    eprintln!();
}

pub fn print_boss_tick(boss: &crate::boss::Boss, player_dmg: Option<i32>, boss_dmg: Option<i32>) {
    if let Some(dmg) = player_dmg {
        eprintln!("{} {} You strike for {}! (HP: {}/{})",
            "💀".bold(),
            format!("[BOSS] {}!", boss.name).red().bold(),
            format!("{}", dmg).green().bold(),
            boss.hp.max(0), boss.max_hp);
    } else {
        eprintln!("{} {} You swing and miss!",
            "💀".bold(),
            format!("[BOSS] {}!", boss.name).red().dimmed());
    }
    if let Some(dmg) = boss_dmg {
        eprintln!("   {} {}",
            "It retaliates —".red(),
            format!("took {} damage.", dmg).red().bold());
    }
}

pub fn print_boss_victory(boss: &crate::boss::Boss, xp: u32, gold: u32) {
    eprintln!();
    eprintln!("{}", "╔══════════════════════════════════════════════╗".yellow().bold());
    eprintln!("{} {} {}",
        "║".yellow().bold(),
        format!("🏆  {} HAS BEEN DEFEATED!", boss.name).yellow().bold(),
        "║".yellow().bold());
    eprintln!("{} {} {}",
        "║".yellow().bold(),
        format!("   +{} XP  +{} gold  — Loot awaits!", xp, gold).yellow(),
        "║".yellow().bold());
    eprintln!("{}", "╚══════════════════════════════════════════════╝".yellow().bold());
    eprintln!();
}

pub fn print_boss_flee(boss_name: &str, reason: &str) {
    eprintln!("{} {} {}",
        "👻".bold(),
        "[BOSS]".red().dimmed(),
        format!("{} {}.", boss_name, reason).dimmed().italic());
}

pub fn print_permadeath_eulogy(char: &Character, killer: &str) {
    eprintln!();
    eprintln!("{}", "☠  ═══════════════════════════════════════════  ☠".red().bold());
    eprintln!();
    eprintln!("       {}", "Y O U   H A V E   D I E D".red().bold());
    eprintln!();
    eprintln!(
        "  Here lies {}, the {} {}.",
        char.name.bold().white(),
        format!("{}", char.race).magenta(),
        format!("{}", char.class).cyan().bold()
    );
    let subclass_str = char
        .subclass
        .as_ref()
        .map_or(String::new(), |s| format!("{}", s).magenta().bold().to_string());
    if !subclass_str.is_empty() {
        eprintln!("  Known also as the {}.", subclass_str);
    }
    eprintln!(
        "  Felled by {} at level {}.",
        killer.red().bold(),
        format!("{}", char.level).yellow().bold()
    );
    eprintln!(
        "  After {} commands, {} kills, {} deaths.",
        format!("{}", char.commands_run).cyan(),
        format!("{}", char.kills).green(),
        format!("{}", char.deaths + 1).red()
    );
    if char.gold > 0 {
        eprintln!(
            "  They carried {} gold into the grave.",
            format!("{}", char.gold).yellow()
        );
    }
    if let Some(w) = &char.weapon {
        eprintln!("  Their blade: {}.", w.name.cyan().italic());
    }
    eprintln!("  Their legend: {}", char.title.yellow().italic());
    eprintln!();
    eprintln!(
        "  {}",
        "The save file has been deleted. All is lost."
            .dimmed()
            .italic()
    );
    eprintln!("{}", "☠  ═══════════════════════════════════════════  ☠".red().bold());
    eprintln!();
}

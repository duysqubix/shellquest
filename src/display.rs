use crate::character::{Character, Item, ItemSlot, Rarity};
use crate::journal::{EventType, JournalEntry};
use crate::zones::Zone;
use colored::*;

// ── Inventory grouping (display-side organization) ──

pub struct InventoryGroup<'a> {
    pub slot: ItemSlot,
    pub items: Vec<&'a Item>,
}

fn rarity_rank(r: &Rarity) -> u8 {
    match r {
        Rarity::Common => 0,
        Rarity::Uncommon => 1,
        Rarity::Rare => 2,
        Rarity::Epic => 3,
        Rarity::Legendary => 4,
    }
}

pub fn group_inventory_by_slot(items: &[Item]) -> Vec<InventoryGroup<'_>> {
    const DISPLAY_ORDER: [ItemSlot; 4] = [
        ItemSlot::Weapon,
        ItemSlot::Armor,
        ItemSlot::Ring,
        ItemSlot::Potion,
    ];
    DISPLAY_ORDER
        .iter()
        .map(|slot| {
            let mut bucket: Vec<&Item> =
                items.iter().filter(|i| i.slot == *slot).collect();
            bucket.sort_by(|a, b| {
                rarity_rank(&b.rarity)
                    .cmp(&rarity_rank(&a.rarity))
                    .then(b.power.cmp(&a.power))
            });
            InventoryGroup { slot: *slot, items: bucket }
        })
        .filter(|g| !g.items.is_empty())
        .collect()
}

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

pub struct GearBonus {
    pub attack: i32,
    pub attack_breakdown: String,
    pub defense: i32,
    pub defense_breakdown: String,
}

fn item_part_label(slot_name: &str, base: i32, enchant: u32) -> String {
    if enchant > 0 {
        format!("{} {} +{} enchant", slot_name, base, enchant)
    } else {
        format!("{} {}", slot_name, base)
    }
}

pub fn gear_bonus(char: &Character) -> GearBonus {
    let mut attack = 0;
    let mut atk_parts: Vec<String> = Vec::new();
    if let Some(w) = char.weapon.as_ref() {
        attack += w.power + w.enchant_level as i32;
        atk_parts.push(item_part_label("weapon", w.power, w.enchant_level));
    }

    let mut defense = 0;
    let mut def_parts: Vec<String> = Vec::new();
    if let Some(a) = char.armor.as_ref() {
        defense += a.power + a.enchant_level as i32;
        def_parts.push(item_part_label("armor", a.power, a.enchant_level));
    }
    if let Some(r) = char.ring.as_ref() {
        defense += r.power + r.enchant_level as i32;
        def_parts.push(item_part_label("ring", r.power, r.enchant_level));
    }

    GearBonus {
        attack,
        attack_breakdown: atk_parts.join(", "),
        defense,
        defense_breakdown: def_parts.join(", "),
    }
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

    let base_atk = char.strength + char.dexterity / 2;
    let base_def = char.dexterity / 3;
    let bonus = gear_bonus(char);
    println!(
        "{}  {} {} {}  {} {} {}",
        "│".dimmed(),
        "ATK:".bold(),
        format!("{}", base_atk).red().bold(),
        format!("(+{})", bonus.attack).dimmed(),
        "DEF:".bold(),
        format!("{}", base_def).green().bold(),
        format!("(+{})", bonus.defense).dimmed(),
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
        let eff = w.power + w.enchant_level as i32;
        format!("{} (+{}) {}{}", name, eff, rarity, enchant_tag(w.enchant_level))
    });
    let armor_str = char.armor.as_ref().map_or("(none)".dimmed().to_string(), |a| {
        let (name, rarity) = format_item_rarity(&a.name, &a.rarity);
        let eff = a.power + a.enchant_level as i32;
        format!("{} (+{}) {}{}", name, eff, rarity, enchant_tag(a.enchant_level))
    });
    let ring_str = char.ring.as_ref().map_or("(none)".dimmed().to_string(), |r| {
        let (name, rarity) = format_item_rarity(&r.name, &r.rarity);
        let eff = r.power + r.enchant_level as i32;
        format!("{} (+{}) {}{}", name, eff, rarity, enchant_tag(r.enchant_level))
    });

    println!("{}  {} {}", "│".dimmed(), "Weapon:".bold(), weapon_str);
    println!("{}  {} {}", "│".dimmed(), "Armor: ".bold(), armor_str);
    println!("{}  {} {}", "│".dimmed(), "Ring:  ".bold(), ring_str);

    if bonus.attack > 0 || bonus.defense > 0 {
        println!("{}", "│".dimmed());
        println!("{}  {}", "│".dimmed(), "Gear bonuses:".bold());
        if bonus.attack > 0 {
            println!(
                "{}    {} {}  {}",
                "│".dimmed(),
                "Attack: ".bold(),
                format!("+{}", bonus.attack).yellow().bold(),
                format!("({})", bonus.attack_breakdown).dimmed()
            );
        }
        if bonus.defense > 0 {
            println!(
                "{}    {} {}  {}",
                "│".dimmed(),
                "Defense:".bold(),
                format!("+{}", bonus.defense).yellow().bold(),
                format!("({})", bonus.defense_breakdown).dimmed()
            );
        }
    }
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

fn slot_icon(slot: ItemSlot) -> &'static str {
    match slot {
        ItemSlot::Weapon => "⚔ ",
        ItemSlot::Armor => "🛡 ",
        ItemSlot::Ring => "💍",
        ItemSlot::Potion => "🧪",
    }
}

fn slot_section_label(slot: ItemSlot) -> &'static str {
    match slot {
        ItemSlot::Weapon => "Weapons",
        ItemSlot::Armor => "Armor",
        ItemSlot::Ring => "Rings",
        ItemSlot::Potion => "Potions",
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ItemSource {
    Equipped,
    Inventory { index: usize, total: usize },
}

fn rarity_label(rarity: &Rarity) -> String {
    let text = format!("{}", rarity);
    match rarity {
        Rarity::Common => text.dimmed().to_string(),
        Rarity::Uncommon => text.white().bold().to_string(),
        Rarity::Rare => text.green().bold().to_string(),
        Rarity::Epic => text.magenta().bold().to_string(),
        Rarity::Legendary => text.yellow().bold().on_black().to_string(),
    }
}

const ITEM_DETAIL_RULE_WIDTH: usize = 42;

pub fn render_item_detail(item: &Item, source: ItemSource) -> String {
    use crate::loot;

    let mut out = String::new();
    let rule = "─".repeat(ITEM_DETAIL_RULE_WIDTH).dimmed().to_string();

    out.push('\n');
    out.push_str(&format!("{}\n", "🔍 Item Details".bold().cyan()));
    out.push_str(&format!("{}\n", rule));

    let (name_styled, _) = format_item_rarity(&item.name, &item.rarity);
    let rarity_styled = rarity_label(&item.rarity);
    out.push_str(&format!(
        "  {}        {}\n",
        "Name:".bold(),
        name_styled
    ));
    out.push_str(&format!(
        "  {}        {}\n",
        "Slot:".bold(),
        format!("{}", item.slot)
    ));
    out.push_str(&format!(
        "  {}      {}\n",
        "Rarity:".bold(),
        rarity_styled
    ));

    let is_potion = matches!(item.slot, ItemSlot::Potion);

    if !is_potion {
        let affects = match item.slot {
            ItemSlot::Weapon => "attack_power",
            ItemSlot::Armor | ItemSlot::Ring => "defense",
            ItemSlot::Potion => unreachable!(),
        };
        out.push_str(&format!(
            "  {}     {}\n",
            "Affects:".bold(),
            affects.cyan()
        ));
    }

    if is_potion {
        out.push_str(&format!(
            "  {}       {}\n",
            "Heals:".bold(),
            format!("{}", item.power).green().bold()
        ));
    } else {
        out.push_str(&format!(
            "  {}  {}\n",
            "Base power:".bold(),
            format!("+{}", item.power).white().bold()
        ));
        if item.enchant_level > 0 {
            out.push_str(&format!(
                "  {}     {}\n",
                "Enchant:".bold(),
                enchant_tag(item.enchant_level).trim_start()
            ));
            let eff = item.power + item.enchant_level as i32;
            out.push_str(&format!(
                "  {}   {}  {}\n",
                "Effective:".bold(),
                format!("+{}", eff).yellow().bold(),
                format!("(base {} + enchant {})", item.power, item.enchant_level).dimmed()
            ));
        }
    }

    let buy = loot::item_price(item);
    out.push_str(&format!(
        "  {}   {}\n",
        "Buy price:".bold(),
        format!("{} gold", buy).yellow()
    ));

    let total_sell = loot::sell_price(item);
    if item.enchant_level > 0 {
        let mut base_item = item.clone();
        base_item.enchant_level = 0;
        let base_sell = loot::sell_price(&base_item);
        let enchant_bonus = total_sell.saturating_sub(base_sell);
        out.push_str(&format!(
            "  {}  {} base + {} enchant bonus = {}\n",
            "Sell value:".bold(),
            format!("{} gold", base_sell).yellow(),
            format!("{} gold", enchant_bonus).yellow(),
            format!("{} gold", total_sell).yellow().bold()
        ));
    } else {
        out.push_str(&format!(
            "  {}  {}\n",
            "Sell value:".bold(),
            format!("{} gold", total_sell).yellow().bold()
        ));
    }

    let source_str = match source {
        ItemSource::Equipped => format!("Equipped ({} slot)", item.slot),
        ItemSource::Inventory { index, total } => {
            format!("Inventory slot {} of {}", index, total)
        }
    };
    out.push_str(&format!(
        "  {}      {}\n",
        "Source:".bold(),
        source_str.cyan()
    ));

    out.push_str(&format!("{}\n", rule));
    out.push('\n');
    out
}

pub fn print_item_detail(item: &Item, source: ItemSource) {
    print!("{}", render_item_detail(item, source));
}

pub fn enchant_tag(level: u32) -> String {
    if level == 0 {
        return String::new();
    }
    let label = format!("[Enchanted +{}]", level);
    let styled = match level {
        1 => label.green().bold().to_string(),
        2 => label.cyan().bold().to_string(),
        3 => label.blue().bold().to_string(),
        4 => label.magenta().bold().to_string(),
        _ => label
            .chars()
            .enumerate()
            .map(|(i, c)| {
                let s = c.to_string();
                match i % 6 {
                    0 => s.red().bold().to_string(),
                    1 => s.yellow().bold().to_string(),
                    2 => s.green().bold().to_string(),
                    3 => s.cyan().bold().to_string(),
                    4 => s.blue().bold().to_string(),
                    _ => s.magenta().bold().to_string(),
                }
            })
            .collect::<String>(),
    };
    format!(" {}", styled)
}

pub fn print_inventory(char: &Character) {
    println!();
    println!("{}", "📦 Inventory".bold().cyan());
    println!("{}", "─".repeat(40).dimmed());

    if char.inventory.is_empty() {
        println!("{}", "  (empty)".dimmed());
    } else {
        let groups = group_inventory_by_slot(&char.inventory);
        let mut idx = 1;
        for group in &groups {
            println!(
                "{} {} {}",
                slot_icon(group.slot),
                slot_section_label(group.slot).bold(),
                format!("({})", group.items.len()).dimmed()
            );
            for item in &group.items {
                let (name_styled, rarity_styled) = format_item_rarity(&item.name, &item.rarity);
                let effective_power = item.power + item.enchant_level as i32;
                println!(
                    "  {}. {} (+{}) {}{}",
                    format!("{}", idx).dimmed(),
                    name_styled,
                    effective_power,
                    rarity_styled,
                    enchant_tag(item.enchant_level),
                );
                idx += 1;
            }
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

pub fn print_boss_tick(boss: &crate::boss::Boss, player_dmg: Option<(i32, bool)>, boss_dmg: Option<i32>) {
    if let Some((dmg, is_crit)) = player_dmg {
        if is_crit {
            eprintln!("{} {} {} You strike for {}! (HP: {}/{})",
                "💀".bold(),
                "CRITICAL!".yellow().bold(),
                format!("[BOSS] {}!", boss.name).red().bold(),
                format!("{}", dmg).yellow().bold(),
                boss.hp.max(0), boss.max_hp);
        } else {
            eprintln!("{} {} You strike for {}! (HP: {}/{})",
                "💀".bold(),
                format!("[BOSS] {}!", boss.name).red().bold(),
                format!("{}", dmg).green().bold(),
                boss.hp.max(0), boss.max_hp);
        }
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

pub fn print_soul_drain(hp_restored: i32, hp: i32, max_hp: i32) {
    eprintln!("   {} {} {}",
        "🩸".bold(),
        format!("Soul drained — +{} HP", hp_restored).magenta().bold(),
        format!("(HP: {}/{})", hp, max_hp).magenta().dimmed());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::{Item, ItemSlot, Rarity};

    fn make_item(name: &str, slot: ItemSlot, power: i32, rarity: Rarity) -> Item {
        Item { name: name.to_string(), slot, power, rarity, enchant_level: 0 }
    }

    #[test]
    fn group_inventory_by_slot_empty_returns_no_groups() {
        let groups = group_inventory_by_slot(&[]);
        assert!(groups.is_empty());
    }

    #[test]
    fn group_inventory_by_slot_single_weapon_produces_one_weapon_group() {
        let items = vec![make_item("Iron Sword", ItemSlot::Weapon, 5, Rarity::Common)];
        let groups = group_inventory_by_slot(&items);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].slot, ItemSlot::Weapon);
        assert_eq!(groups[0].items.len(), 1);
        assert_eq!(groups[0].items[0].name, "Iron Sword");
    }

    #[test]
    fn group_inventory_by_slot_returns_groups_in_fixed_display_order() {
        let items = vec![
            make_item("Potion", ItemSlot::Potion, 5, Rarity::Common),
            make_item("Ring", ItemSlot::Ring, 5, Rarity::Common),
            make_item("Armor", ItemSlot::Armor, 5, Rarity::Common),
            make_item("Weapon", ItemSlot::Weapon, 5, Rarity::Common),
        ];
        let groups = group_inventory_by_slot(&items);
        let slots: Vec<ItemSlot> = groups.iter().map(|g| g.slot).collect();
        assert_eq!(
            slots,
            vec![ItemSlot::Weapon, ItemSlot::Armor, ItemSlot::Ring, ItemSlot::Potion]
        );
    }

    #[test]
    fn group_inventory_by_slot_skips_slots_with_no_items() {
        let items = vec![
            make_item("Sword", ItemSlot::Weapon, 5, Rarity::Common),
            make_item("Mana Potion", ItemSlot::Potion, 5, Rarity::Common),
        ];
        let groups = group_inventory_by_slot(&items);
        let slots: Vec<ItemSlot> = groups.iter().map(|g| g.slot).collect();
        assert_eq!(slots, vec![ItemSlot::Weapon, ItemSlot::Potion]);
    }

    #[test]
    fn group_inventory_by_slot_sorts_equal_rarity_by_power_descending() {
        let items = vec![
            make_item("Weak", ItemSlot::Weapon, 3, Rarity::Common),
            make_item("Strong", ItemSlot::Weapon, 7, Rarity::Common),
            make_item("Mid", ItemSlot::Weapon, 5, Rarity::Common),
        ];
        let groups = group_inventory_by_slot(&items);
        let names: Vec<&str> = groups[0].items.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(names, vec!["Strong", "Mid", "Weak"]);
    }

    #[test]
    fn group_inventory_by_slot_sorts_items_by_rarity_descending_within_group() {
        let items = vec![
            make_item("Common Sword", ItemSlot::Weapon, 5, Rarity::Common),
            make_item("Legendary Sword", ItemSlot::Weapon, 5, Rarity::Legendary),
            make_item("Rare Sword", ItemSlot::Weapon, 5, Rarity::Rare),
            make_item("Uncommon Sword", ItemSlot::Weapon, 5, Rarity::Uncommon),
            make_item("Epic Sword", ItemSlot::Weapon, 5, Rarity::Epic),
        ];
        let groups = group_inventory_by_slot(&items);
        let names: Vec<&str> = groups[0].items.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "Legendary Sword",
                "Epic Sword",
                "Rare Sword",
                "Uncommon Sword",
                "Common Sword",
            ]
        );
    }

    #[test]
    fn enchant_tag_level_zero_returns_empty_string() {
        assert_eq!(enchant_tag(0), "");
    }

    #[test]
    fn enchant_tag_level_one_contains_label_and_has_color_prefix() {
        colored::control::set_override(false);
        let tag = enchant_tag(1);
        assert!(tag.starts_with(' '));
        assert!(tag.contains("[Enchanted +1]"));
    }

    #[test]
    fn enchant_tag_level_five_contains_label() {
        colored::control::set_override(false);
        let tag = enchant_tag(5);
        assert!(tag.contains("[Enchanted +5]"));
    }

    #[test]
    fn enchant_tag_each_level_uses_distinct_color() {
        colored::control::set_override(true);
        let t1 = enchant_tag(1);
        let t2 = enchant_tag(2);
        let t3 = enchant_tag(3);
        let t4 = enchant_tag(4);
        let t5 = enchant_tag(5);
        let distinct = std::collections::HashSet::from([t1.clone(), t2.clone(), t3.clone(), t4.clone(), t5.clone()]);
        assert_eq!(distinct.len(), 5, "all five tag styles should be visually distinct");
    }

    fn item_with(name: &str, slot: ItemSlot, power: i32, rarity: Rarity, enchant_level: u32) -> Item {
        Item { name: name.to_string(), slot, power, rarity, enchant_level }
    }

    #[test]
    fn render_item_detail_common_weapon_shows_core_fields() {
        colored::control::set_override(false);
        let it = item_with("Iron Sword", ItemSlot::Weapon, 5, Rarity::Common, 0);
        let out = render_item_detail(&it, ItemSource::Inventory { index: 3, total: 12 });
        assert!(out.contains("🔍 Item Details"), "header missing:\n{out}");
        assert!(out.contains("Name:"));
        assert!(out.contains("Iron Sword"));
        assert!(out.contains("Slot:"));
        assert!(out.contains("Weapon"));
        assert!(out.contains("Rarity:"));
        assert!(out.contains("Common"), "rarity word 'Common' must appear:\n{out}");
        assert!(!out.contains("[Common]"), "rarity should render as plain word, not as bracketed tag:\n{out}");
        assert!(out.contains("Base power:"));
        assert!(out.contains("+5"));
        assert!(out.contains("Buy price:"));
        assert!(out.contains("Sell value:"));
        assert!(out.contains("Source:"));
        assert!(out.contains("Inventory slot 3 of 12"));
        assert!(!out.contains("Enchant:"), "common item should not show Enchant line:\n{out}");
        assert!(!out.contains("Effective:"), "common item should not show Effective line:\n{out}");
        assert!(!out.contains("Heals:"), "weapon should not show Heals line:\n{out}");
    }

    #[test]
    fn render_item_detail_enchanted_epic_ring_shows_enchant_and_effective() {
        colored::control::set_override(false);
        let it = item_with("Ring of Fortune", ItemSlot::Ring, 12, Rarity::Epic, 3);
        let out = render_item_detail(&it, ItemSource::Equipped);
        assert!(out.contains("Ring of Fortune"));
        assert!(out.contains("Epic"));
        assert!(!out.contains("[Epic]"), "rarity should render as plain word, not as bracketed tag:\n{out}");
        assert!(out.contains("Base power:"));
        assert!(out.contains("+12"));
        assert!(out.contains("Enchant:"));
        assert!(out.contains("[Enchanted +3]"));
        assert!(out.contains("Effective:"));
        assert!(out.contains("+15"));
        assert!(out.contains("base 12 + enchant 3"));
        assert!(out.contains("Sell value:"));
        assert!(out.contains("base + "));
        assert!(out.contains("enchant bonus = "));
        assert!(out.contains("Source:"));
        assert!(out.contains("Equipped (Ring slot)"));
    }

    #[test]
    fn render_item_detail_potion_shows_heals_not_enchant() {
        colored::control::set_override(false);
        let it = item_with("Common Potion", ItemSlot::Potion, 5, Rarity::Common, 0);
        let out = render_item_detail(&it, ItemSource::Inventory { index: 7, total: 20 });
        assert!(out.contains("Common Potion"));
        assert!(out.contains("Slot:"));
        assert!(out.contains("Potion"));
        assert!(out.contains("Heals:"));
        assert!(out.contains("Buy price:"));
        assert!(out.contains("Sell value:"));
        assert!(out.contains("Inventory slot 7 of 20"));
        assert!(!out.contains("Base power:"), "potion should not show Base power:\n{out}");
        assert!(!out.contains("Enchant:"), "potion should not show Enchant:\n{out}");
        assert!(!out.contains("Effective:"), "potion should not show Effective:\n{out}");
        assert!(!out.contains("Affects:"), "potion has no affects line (single-use):\n{out}");
    }

    #[test]
    fn render_item_detail_weapon_shows_affects_attack_power() {
        colored::control::set_override(false);
        let it = item_with("Iron Sword", ItemSlot::Weapon, 5, Rarity::Common, 0);
        let out = render_item_detail(&it, ItemSource::Equipped);
        assert!(out.contains("Affects:"), "weapon must show Affects line:\n{out}");
        assert!(out.contains("attack_power"), "weapon affects attack_power:\n{out}");
    }

    #[test]
    fn render_item_detail_armor_shows_affects_defense() {
        colored::control::set_override(false);
        let it = item_with("Plate Mail", ItemSlot::Armor, 5, Rarity::Common, 0);
        let out = render_item_detail(&it, ItemSource::Equipped);
        assert!(out.contains("Affects:"), "armor must show Affects line:\n{out}");
        assert!(out.contains("defense"), "armor affects defense:\n{out}");
    }

    #[test]
    fn render_item_detail_ring_shows_affects_defense() {
        colored::control::set_override(false);
        let it = item_with("Ring of Vigor", ItemSlot::Ring, 5, Rarity::Common, 0);
        let out = render_item_detail(&it, ItemSource::Equipped);
        assert!(out.contains("Affects:"), "ring must show Affects line:\n{out}");
        assert!(out.contains("defense"), "ring affects defense:\n{out}");
    }

    #[test]
    fn gear_bonus_naked_character_zero_both() {
        use crate::character::{Character, Class, Race};
        let c = Character::new("Test".to_string(), Class::Warrior, Race::Human);
        let b = gear_bonus(&c);
        assert_eq!(b.attack, 0);
        assert_eq!(b.defense, 0);
        assert_eq!(b.attack_breakdown, "");
        assert_eq!(b.defense_breakdown, "");
    }

    #[test]
    fn gear_bonus_full_loadout_sums_correctly() {
        use crate::character::{Character, Class, Race};
        let mut c = Character::new("Test".to_string(), Class::Warrior, Race::Human);
        let mut weapon = item_with("W", ItemSlot::Weapon, 10, Rarity::Rare, 3);
        weapon.enchant_level = 3;
        let mut armor = item_with("A", ItemSlot::Armor, 8, Rarity::Common, 2);
        armor.enchant_level = 2;
        let mut ring = item_with("R", ItemSlot::Ring, 4, Rarity::Common, 0);
        ring.enchant_level = 0;
        c.weapon = Some(weapon);
        c.armor = Some(armor);
        c.ring = Some(ring);
        let b = gear_bonus(&c);
        assert_eq!(b.attack, 13, "weapon 10 + 3 enchant");
        assert_eq!(b.defense, 14, "armor 10 + ring 4");
        assert!(b.attack_breakdown.contains("weapon 10 +3 enchant"));
        assert!(b.defense_breakdown.contains("armor 8 +2 enchant"));
        assert!(b.defense_breakdown.contains("ring 4"));
        assert!(!b.defense_breakdown.contains("ring 4 +0"), "no-enchant ring should not show enchant note");
    }

    #[test]
    fn render_item_detail_equipped_source_names_slot_per_item_kind() {
        colored::control::set_override(false);
        let weapon = item_with("Pike", ItemSlot::Weapon, 6, Rarity::Common, 0);
        let armor = item_with("Plate", ItemSlot::Armor, 6, Rarity::Common, 0);
        let ring = item_with("Band", ItemSlot::Ring, 6, Rarity::Common, 0);
        assert!(render_item_detail(&weapon, ItemSource::Equipped).contains("Equipped (Weapon slot)"));
        assert!(render_item_detail(&armor, ItemSource::Equipped).contains("Equipped (Armor slot)"));
        assert!(render_item_detail(&ring, ItemSource::Equipped).contains("Equipped (Ring slot)"));
    }

    #[test]
    fn render_item_detail_aligns_all_value_columns() {
        colored::control::set_override(false);
        let it = item_with("Pike", ItemSlot::Weapon, 6, Rarity::Rare, 2);
        let out = render_item_detail(&it, ItemSource::Equipped);
        let labels = [
            "Name:",
            "Slot:",
            "Rarity:",
            "Base power:",
            "Enchant:",
            "Effective:",
            "Buy price:",
            "Sell value:",
            "Source:",
        ];
        for line in out.lines() {
            for label in labels {
                if let Some(idx) = line.find(label) {
                    let after_label = idx + label.len();
                    let trimmed_tail = &line[after_label..];
                    let value_col = after_label
                        + trimmed_tail.chars().take_while(|c| *c == ' ').count();
                    assert_eq!(
                        value_col, 15,
                        "label {:?} should align its value at column 15, got col {} on line:\n{}",
                        label, value_col, line
                    );
                }
            }
        }
    }

    #[test]
    fn render_item_detail_sell_value_math_matches_loot_helper() {
        colored::control::set_override(false);
        let it = item_with("Pike of Ping", ItemSlot::Weapon, 8, Rarity::Uncommon, 2);
        let out = render_item_detail(&it, ItemSource::Equipped);
        let buy = crate::loot::item_price(&it);
        let total = crate::loot::sell_price(&it);
        let base = buy / 2;
        let bonus = 2 * (buy / 5);
        assert_eq!(base + bonus, total, "test fixture sanity");
        assert!(out.contains(&format!("{} gold base", base)), "missing base sell:\n{out}");
        assert!(out.contains(&format!("{} gold enchant", bonus)), "missing bonus sell:\n{out}");
        assert!(out.contains(&format!("= {} gold", total)), "missing total sell:\n{out}");
    }
}

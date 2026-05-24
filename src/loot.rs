use crate::character::{Item, ItemSlot, Rarity};
use rand::Rng;

struct LootEntry {
    name: &'static str,
    slot: ItemSlot,
    power_range: (i32, i32),
}

// ═══════════════════════════════════════════════════════════════
//  RARITY TIERS — drop rates (out of 10000)
//  Common: 70%, Uncommon: 25%, Rare: 4%, Epic: 0.99%, Legendary: 0.01%
// ═══════════════════════════════════════════════════════════════

fn roll_rarity(rng: &mut impl Rng) -> Rarity {
    let roll = rng.gen_range(0u32..10000);
    match roll {
        0..=6999 => Rarity::Common,       // 70.00%
        7000..=9499 => Rarity::Uncommon,   // 25.00%
        9500..=9899 => Rarity::Rare,       // 4.00%
        9900..=9998 => Rarity::Epic,       // 0.99%
        _ => Rarity::Legendary,            // 0.01%
    }
}

// ═══════════════════════════════════════════════════════════════
//  COMMON ITEMS — the bread and butter (lots of variety)
// ═══════════════════════════════════════════════════════════════

const COMMON: &[LootEntry] = &[
    // Weapons
    LootEntry { name: "Rusty Pipe", slot: ItemSlot::Weapon, power_range: (1, 3) },
    LootEntry { name: "Keyboard of Smiting", slot: ItemSlot::Weapon, power_range: (2, 4) },
    LootEntry { name: "Mouse of Clicking", slot: ItemSlot::Weapon, power_range: (1, 3) },
    LootEntry { name: "Broken USB Stick", slot: ItemSlot::Weapon, power_range: (1, 2) },
    LootEntry { name: "Ethernet Whip", slot: ItemSlot::Weapon, power_range: (2, 3) },
    LootEntry { name: "Floppy Disk Shuriken", slot: ItemSlot::Weapon, power_range: (1, 4) },
    LootEntry { name: "Paperclip of Poking", slot: ItemSlot::Weapon, power_range: (1, 2) },
    LootEntry { name: "Sticky Note Dart", slot: ItemSlot::Weapon, power_range: (1, 3) },
    LootEntry { name: "VGA Cable Lasso", slot: ItemSlot::Weapon, power_range: (2, 3) },
    LootEntry { name: "CD-ROM Frisbee", slot: ItemSlot::Weapon, power_range: (1, 4) },
    LootEntry { name: "Bent Antenna", slot: ItemSlot::Weapon, power_range: (1, 2) },
    LootEntry { name: "Thermal Paste Spatula", slot: ItemSlot::Weapon, power_range: (1, 3) },
    LootEntry { name: "Dead Battery Club", slot: ItemSlot::Weapon, power_range: (2, 4) },
    LootEntry { name: "Tangled Earbuds Whip", slot: ItemSlot::Weapon, power_range: (1, 3) },
    LootEntry { name: "HDMI Cable Nunchucks", slot: ItemSlot::Weapon, power_range: (2, 4) },
    LootEntry { name: "Dust Bunny Launcher", slot: ItemSlot::Weapon, power_range: (1, 3) },
    LootEntry { name: "404 Page Scroll", slot: ItemSlot::Weapon, power_range: (1, 2) },
    LootEntry { name: "Cracked Screen Shard", slot: ItemSlot::Weapon, power_range: (2, 4) },
    // Armor
    LootEntry { name: "Hoodie of Comfort", slot: ItemSlot::Armor, power_range: (1, 3) },
    LootEntry { name: "T-Shirt of Localhost", slot: ItemSlot::Armor, power_range: (1, 2) },
    LootEntry { name: "Pajama Pants of WFH", slot: ItemSlot::Armor, power_range: (1, 3) },
    LootEntry { name: "Flip-Flops of Friday Deploy", slot: ItemSlot::Armor, power_range: (1, 2) },
    LootEntry { name: "Beanie of Bluetooth", slot: ItemSlot::Armor, power_range: (2, 3) },
    LootEntry { name: "Lanyard of the Intern", slot: ItemSlot::Armor, power_range: (1, 2) },
    LootEntry { name: "Cardigan of Code Review", slot: ItemSlot::Armor, power_range: (1, 3) },
    LootEntry { name: "Socks of Static", slot: ItemSlot::Armor, power_range: (1, 2) },
    LootEntry { name: "Conference Swag Tee", slot: ItemSlot::Armor, power_range: (1, 3) },
    LootEntry { name: "Wrinkled Khakis of Standup", slot: ItemSlot::Armor, power_range: (2, 3) },
    LootEntry { name: "Baseball Cap of Backwards Compat", slot: ItemSlot::Armor, power_range: (1, 3) },
    LootEntry { name: "Crocs of Casual Friday", slot: ItemSlot::Armor, power_range: (1, 2) },
    LootEntry { name: "Scarf of Spaghetti Code", slot: ItemSlot::Armor, power_range: (1, 3) },
    LootEntry { name: "Wristband of Agile", slot: ItemSlot::Armor, power_range: (1, 2) },
    LootEntry { name: "Sunglasses of Screen Glare", slot: ItemSlot::Armor, power_range: (1, 3) },
    // Rings
    LootEntry { name: "Ring of Tab Completion", slot: ItemSlot::Ring, power_range: (1, 3) },
    LootEntry { name: "Band of Backspace", slot: ItemSlot::Ring, power_range: (1, 2) },
    LootEntry { name: "Loop of Localhost", slot: ItemSlot::Ring, power_range: (1, 3) },
    LootEntry { name: "Rubber Band of Resilience", slot: ItemSlot::Ring, power_range: (1, 2) },
    LootEntry { name: "Twist Tie of Binding", slot: ItemSlot::Ring, power_range: (1, 2) },
    LootEntry { name: "Keyring of SSH", slot: ItemSlot::Ring, power_range: (1, 3) },
    LootEntry { name: "Washer of the Machine Room", slot: ItemSlot::Ring, power_range: (1, 2) },
    LootEntry { name: "Ring Pull of Red Bull", slot: ItemSlot::Ring, power_range: (1, 3) },
    // Potions
    LootEntry { name: "Potion of Coffee", slot: ItemSlot::Potion, power_range: (5, 10) },
    LootEntry { name: "Vial of Green Tea", slot: ItemSlot::Potion, power_range: (3, 8) },
    LootEntry { name: "Flask of Water (Stay Hydrated)", slot: ItemSlot::Potion, power_range: (4, 7) },
    LootEntry { name: "Sip of Instant Noodle Broth", slot: ItemSlot::Potion, power_range: (3, 6) },
    LootEntry { name: "Half a Granola Bar", slot: ItemSlot::Potion, power_range: (2, 5) },
    LootEntry { name: "Stale Donut of the Breakroom", slot: ItemSlot::Potion, power_range: (3, 7) },
    LootEntry { name: "Lukewarm La Croix", slot: ItemSlot::Potion, power_range: (2, 6) },
    LootEntry { name: "Vending Machine Chips", slot: ItemSlot::Potion, power_range: (3, 6) },
    LootEntry { name: "Day-Old Pizza Slice", slot: ItemSlot::Potion, power_range: (4, 8) },
    LootEntry { name: "Mug of Decaf (Placebo)", slot: ItemSlot::Potion, power_range: (1, 4) },
];

// ═══════════════════════════════════════════════════════════════
//  UNCOMMON ITEMS — solid upgrades
// ═══════════════════════════════════════════════════════════════

const UNCOMMON: &[LootEntry] = &[
    // Weapons
    LootEntry { name: "Sword of Regex", slot: ItemSlot::Weapon, power_range: (4, 7) },
    LootEntry { name: "Axe of Grep", slot: ItemSlot::Weapon, power_range: (5, 8) },
    LootEntry { name: "Dagger of Sed", slot: ItemSlot::Weapon, power_range: (3, 9) },
    LootEntry { name: "Mace of Makefile", slot: ItemSlot::Weapon, power_range: (4, 8) },
    LootEntry { name: "Bow of Bash", slot: ItemSlot::Weapon, power_range: (5, 7) },
    LootEntry { name: "Halberd of HTTP", slot: ItemSlot::Weapon, power_range: (4, 9) },
    LootEntry { name: "Spear of SQL", slot: ItemSlot::Weapon, power_range: (5, 8) },
    LootEntry { name: "Crossbow of CORS", slot: ItemSlot::Weapon, power_range: (4, 8) },
    LootEntry { name: "Flail of Flexbox", slot: ItemSlot::Weapon, power_range: (5, 7) },
    LootEntry { name: "Pike of Ping", slot: ItemSlot::Weapon, power_range: (4, 7) },
    LootEntry { name: "Morningstar of Middleware", slot: ItemSlot::Weapon, power_range: (5, 9) },
    LootEntry { name: "Rapier of REST", slot: ItemSlot::Weapon, power_range: (5, 8) },
    LootEntry { name: "Javelin of JSON", slot: ItemSlot::Weapon, power_range: (4, 8) },
    LootEntry { name: "Sling of Svelte", slot: ItemSlot::Weapon, power_range: (4, 7) },
    LootEntry { name: "Whip of Webpack", slot: ItemSlot::Weapon, power_range: (5, 9) },
    // Armor
    LootEntry { name: "Cloak of Stdout", slot: ItemSlot::Armor, power_range: (3, 6) },
    LootEntry { name: "Vest of Version Control", slot: ItemSlot::Armor, power_range: (4, 7) },
    LootEntry { name: "Gauntlets of Gzip", slot: ItemSlot::Armor, power_range: (3, 6) },
    LootEntry { name: "Helmet of HTTPS", slot: ItemSlot::Armor, power_range: (4, 7) },
    LootEntry { name: "Boots of Bootstrap", slot: ItemSlot::Armor, power_range: (3, 5) },
    LootEntry { name: "Cape of CI/CD", slot: ItemSlot::Armor, power_range: (5, 7) },
    LootEntry { name: "Pauldrons of PostgreSQL", slot: ItemSlot::Armor, power_range: (4, 7) },
    LootEntry { name: "Bracers of Brotli", slot: ItemSlot::Armor, power_range: (3, 6) },
    LootEntry { name: "Leggings of Linting", slot: ItemSlot::Armor, power_range: (4, 6) },
    LootEntry { name: "Visor of Vim Motions", slot: ItemSlot::Armor, power_range: (4, 7) },
    LootEntry { name: "Shoulderguards of Scrum", slot: ItemSlot::Armor, power_range: (3, 6) },
    LootEntry { name: "Chaps of Caching", slot: ItemSlot::Armor, power_range: (4, 7) },
    // Rings
    LootEntry { name: "Ring of Syntax Highlight", slot: ItemSlot::Ring, power_range: (2, 5) },
    LootEntry { name: "Signet of SSH", slot: ItemSlot::Ring, power_range: (3, 6) },
    LootEntry { name: "Circlet of Cron", slot: ItemSlot::Ring, power_range: (3, 5) },
    LootEntry { name: "Band of Base64", slot: ItemSlot::Ring, power_range: (3, 6) },
    LootEntry { name: "Ring of Rate Limiting", slot: ItemSlot::Ring, power_range: (2, 5) },
    LootEntry { name: "Amulet of Async/Await", slot: ItemSlot::Ring, power_range: (3, 6) },
    LootEntry { name: "Pendant of Package.json", slot: ItemSlot::Ring, power_range: (2, 5) },
    // Potions
    LootEntry { name: "Elixir of Energy Drink", slot: ItemSlot::Potion, power_range: (10, 20) },
    LootEntry { name: "Brew of Debugging", slot: ItemSlot::Potion, power_range: (12, 18) },
    LootEntry { name: "Tincture of Focus Mode", slot: ItemSlot::Potion, power_range: (8, 15) },
    LootEntry { name: "Smoothie of Sprint Planning", slot: ItemSlot::Potion, power_range: (10, 16) },
    LootEntry { name: "Espresso Shot of Urgency", slot: ItemSlot::Potion, power_range: (8, 14) },
    LootEntry { name: "Matcha of Mindfulness", slot: ItemSlot::Potion, power_range: (10, 18) },
    LootEntry { name: "Cold Brew of All-Nighter", slot: ItemSlot::Potion, power_range: (12, 20) },
];

// ═══════════════════════════════════════════════════════════════
//  RARE ITEMS — meaningful finds
// ═══════════════════════════════════════════════════════════════

const RARE: &[LootEntry] = &[
    // Weapons
    LootEntry { name: "Blade of Sudo", slot: ItemSlot::Weapon, power_range: (8, 12) },
    LootEntry { name: "Staff of Stack Overflow", slot: ItemSlot::Weapon, power_range: (7, 13) },
    LootEntry { name: "Hammer of Compiler", slot: ItemSlot::Weapon, power_range: (9, 14) },
    LootEntry { name: "Trident of TypeScript", slot: ItemSlot::Weapon, power_range: (8, 13) },
    LootEntry { name: "Scythe of Segfault", slot: ItemSlot::Weapon, power_range: (10, 15) },
    LootEntry { name: "Wand of WebSocket", slot: ItemSlot::Weapon, power_range: (7, 12) },
    LootEntry { name: "Claymore of Concurrency", slot: ItemSlot::Weapon, power_range: (9, 14) },
    LootEntry { name: "Katana of Kubernetes", slot: ItemSlot::Weapon, power_range: (10, 15) },
    // Armor
    LootEntry { name: "Chestplate of Chmod 777", slot: ItemSlot::Armor, power_range: (7, 11) },
    LootEntry { name: "Plate of the Firewall", slot: ItemSlot::Armor, power_range: (8, 12) },
    LootEntry { name: "Breastplate of Bcrypt", slot: ItemSlot::Armor, power_range: (9, 13) },
    LootEntry { name: "Shield of CORS", slot: ItemSlot::Armor, power_range: (7, 11) },
    LootEntry { name: "Greaves of GraphQL", slot: ItemSlot::Armor, power_range: (8, 12) },
    LootEntry { name: "Crown of CloudFormation", slot: ItemSlot::Armor, power_range: (9, 14) },
    // Rings
    LootEntry { name: "Ring of the Daemon", slot: ItemSlot::Ring, power_range: (4, 8) },
    LootEntry { name: "Seal of Semaphore", slot: ItemSlot::Ring, power_range: (5, 9) },
    LootEntry { name: "Band of the Borrow Checker", slot: ItemSlot::Ring, power_range: (6, 10) },
    LootEntry { name: "Talisman of TLS", slot: ItemSlot::Ring, power_range: (5, 9) },
    // Potions
    LootEntry { name: "Flask of Liquid Nitrogen", slot: ItemSlot::Potion, power_range: (20, 35) },
    LootEntry { name: "Draught of Deep Work", slot: ItemSlot::Potion, power_range: (18, 30) },
    LootEntry { name: "Philter of Flow State", slot: ItemSlot::Potion, power_range: (22, 35) },
];

// ═══════════════════════════════════════════════════════════════
//  EPIC ITEMS — jaw-dropping finds (0.99%)
// ═══════════════════════════════════════════════════════════════

const EPIC: &[LootEntry] = &[
    // Weapons
    LootEntry { name: "Excalibash", slot: ItemSlot::Weapon, power_range: (13, 18) },
    LootEntry { name: "Vorpal Pointer", slot: ItemSlot::Weapon, power_range: (15, 22) },
    LootEntry { name: "Mjolnir of Monorepo", slot: ItemSlot::Weapon, power_range: (16, 21) },
    LootEntry { name: "Gungnir of Git Rebase", slot: ItemSlot::Weapon, power_range: (14, 23) },
    LootEntry { name: "Naginata of Nginx", slot: ItemSlot::Weapon, power_range: (15, 20) },
    // Armor
    LootEntry { name: "Armor of the Container", slot: ItemSlot::Armor, power_range: (12, 17) },
    LootEntry { name: "Aegis of the Load Balancer", slot: ItemSlot::Armor, power_range: (14, 19) },
    LootEntry { name: "Warplate of Kubernetes", slot: ItemSlot::Armor, power_range: (13, 18) },
    // Rings
    LootEntry { name: "Ring of Root Access", slot: ItemSlot::Ring, power_range: (8, 14) },
    LootEntry { name: "Signet of Zero-Day", slot: ItemSlot::Ring, power_range: (10, 16) },
    // Potions
    LootEntry { name: "Phoenix Elixir of Hot Reload", slot: ItemSlot::Potion, power_range: (30, 50) },
];

// ═══════════════════════════════════════════════════════════════
//  LEGENDARY ITEMS — once in a lifetime (0.01%)
// ═══════════════════════════════════════════════════════════════

const LEGENDARY: &[LootEntry] = &[
    // Weapons
    LootEntry { name: "Mass Migration Sword of Chaos", slot: ItemSlot::Weapon, power_range: (20, 30) },
    LootEntry { name: "Mass Migration Blade of the Kernel", slot: ItemSlot::Weapon, power_range: (25, 35) },
    LootEntry { name: "The Mass Migration Fork Bomb", slot: ItemSlot::Weapon, power_range: (22, 32) },
    // Armor
    LootEntry { name: "Shell of Invulnerability", slot: ItemSlot::Armor, power_range: (18, 25) },
    LootEntry { name: "Divine Armor of /dev/null", slot: ItemSlot::Armor, power_range: (20, 28) },
    // Rings
    LootEntry { name: "The One Ring (of SSH Keys)", slot: ItemSlot::Ring, power_range: (14, 20) },
    LootEntry { name: "Eternal Band of Uptime", slot: ItemSlot::Ring, power_range: (16, 22) },
    // Potions
    LootEntry { name: "Elixir of Infinite Context", slot: ItemSlot::Potion, power_range: (50, 99) },
];

fn pick_from(rng: &mut impl Rng, table: &[LootEntry], rarity: Rarity) -> Item {
    let entry = &table[rng.gen_range(0..table.len())];
    let power = rng.gen_range(entry.power_range.0..=entry.power_range.1);
    Item { name: entry.name.to_string(), slot: entry.slot, power, rarity, enchant_level: 0 }
}

pub fn roll_loot(_danger_level: u32) -> Item {
    let mut rng = rand::thread_rng();
    let rarity = roll_rarity(&mut rng);

    match rarity {
        Rarity::Common => pick_from(&mut rng, COMMON, Rarity::Common),
        Rarity::Uncommon => pick_from(&mut rng, UNCOMMON, Rarity::Uncommon),
        Rarity::Rare => pick_from(&mut rng, RARE, Rarity::Rare),
        Rarity::Epic => pick_from(&mut rng, EPIC, Rarity::Epic),
        Rarity::Legendary => pick_from(&mut rng, LEGENDARY, Rarity::Legendary),
    }
}

fn roll_item_of_rarity(rarity: Rarity, _danger_level: u32) -> Item {
    let mut rng = rand::thread_rng();
    match rarity {
        Rarity::Common => pick_from(&mut rng, COMMON, Rarity::Common),
        Rarity::Uncommon => pick_from(&mut rng, UNCOMMON, Rarity::Uncommon),
        Rarity::Rare => pick_from(&mut rng, RARE, Rarity::Rare),
        Rarity::Epic => pick_from(&mut rng, EPIC, Rarity::Epic),
        Rarity::Legendary => pick_from(&mut rng, LEGENDARY, Rarity::Legendary),
    }
}

/// Roll boss loot — no Commons, weighted toward Rare/Epic/Legendary.
/// Uncommon 40%, Rare ~47%, Epic ~10%, Legendary ~3%
pub fn roll_boss_loot() -> Item {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let rarity = if rng.gen_ratio(3, 100) {
        Rarity::Legendary
    } else if rng.gen_ratio(10, 97) {
        Rarity::Epic
    } else if rng.gen_ratio(47, 87) {
        Rarity::Rare
    } else {
        Rarity::Uncommon
    };
    roll_item_of_rarity(rarity, 3)
}

/// Roll loot for the shop — Common, Uncommon, or Rare only (no Epic/Legendary).
pub fn roll_shop_loot() -> Item {
    let mut rng = rand::thread_rng();
    // Redistribute: Common 70%, Uncommon 25%, Rare 5%
    let roll = rng.gen_range(0u32..100);
    match roll {
        0..=69 => pick_from(&mut rng, COMMON, Rarity::Common),
        70..=94 => pick_from(&mut rng, UNCOMMON, Rarity::Uncommon),
        _ => pick_from(&mut rng, RARE, Rarity::Rare),
    }
}

/// Roll loot with danger-based rarity scaling.
/// Higher danger = better odds for rare/epic/legendary drops.
/// danger_level 1: normal odds (matching roll_loot)
/// danger_level 2+3: pushes common→uncommon, small epic chance
/// danger_level 4+5: significantly better odds, legendary possible
pub fn roll_loot_scaled(danger_level: u32) -> Item {
    let mut rng = rand::thread_rng();
    let roll = rng.gen_range(0u32..10000);
    let rarity = match danger_level {
        1 => match roll {
            0..=6999 => Rarity::Common,
            7000..=9499 => Rarity::Uncommon,
            9500..=9899 => Rarity::Rare,
            9900..=9998 => Rarity::Epic,
            _ => Rarity::Legendary,
        },
        2 => match roll {
            0..=6499 => Rarity::Common,
            6500..=8999 => Rarity::Uncommon,
            9000..=9799 => Rarity::Rare,
            9800..=9969 => Rarity::Epic,
            _ => Rarity::Legendary,
        },
        3 => match roll {
            0..=5499 => Rarity::Common,
            5500..=8299 => Rarity::Uncommon,
            8300..=9499 => Rarity::Rare,
            9500..=9919 => Rarity::Epic,
            _ => Rarity::Legendary,
        },
        4 => match roll {
            0..=4500 => Rarity::Common,
            4501..=7500 => Rarity::Uncommon,
            7501..=9100 => Rarity::Rare,
            9101..=9900 => Rarity::Epic,
            _ => Rarity::Legendary,
        },
        5 => match roll {
            0..=3000 => Rarity::Common,
            3001..=6000 => Rarity::Uncommon,
            6001..=8200 => Rarity::Rare,
            8201..=9700 => Rarity::Epic,
            _ => Rarity::Legendary,
        },
        6 => match roll {
            0..=2000 => Rarity::Common,
            2001..=5000 => Rarity::Uncommon,
            5001..=7800 => Rarity::Rare,
            7801..=9600 => Rarity::Epic,
            _ => Rarity::Legendary,
        },
        7 => match roll {
            0..=1500 => Rarity::Common,
            1501..=4000 => Rarity::Uncommon,
            4001..=7200 => Rarity::Rare,
            7201..=9400 => Rarity::Epic,
            _ => Rarity::Legendary,
        },
        8 => match roll {
            0..=1000 => Rarity::Common,
            1001..=3000 => Rarity::Uncommon,
            3001..=6500 => Rarity::Rare,
            6501..=9200 => Rarity::Epic,
            _ => Rarity::Legendary,
        },
        9 => match roll {
            0..=499 => Rarity::Common,
            500..=1999 => Rarity::Uncommon,
            2000..=5499 => Rarity::Rare,
            5500..=9499 => Rarity::Epic,
            _ => Rarity::Legendary,
        },
        _ => match roll {
            0..=499 => Rarity::Common,
            500..=1999 => Rarity::Uncommon,
            2000..=5499 => Rarity::Rare,
            5500..=9499 => Rarity::Epic,
            _ => Rarity::Legendary,
        },
    };
    match rarity {
        Rarity::Common => pick_from(&mut rng, COMMON, Rarity::Common),
        Rarity::Uncommon => pick_from(&mut rng, UNCOMMON, Rarity::Uncommon),
        Rarity::Rare => pick_from(&mut rng, RARE, Rarity::Rare),
        Rarity::Epic => pick_from(&mut rng, EPIC, Rarity::Epic),
        Rarity::Legendary => pick_from(&mut rng, LEGENDARY, Rarity::Legendary),
    }
}

pub fn item_price(item: &Item) -> u32 {
    let multiplier = match item.rarity {
        Rarity::Common => 5,
        Rarity::Uncommon => 8,
        Rarity::Rare => 13,
        Rarity::Epic => 22,
        Rarity::Legendary => 35,
    };
    (item.power as u32) * multiplier + multiplier
}

pub fn enchant_cost(item: &Item) -> u32 {
    item_price(item) * (item.enchant_level + 1)
}

pub const MAX_ENCHANT_LEVEL: u32 = 5;

pub fn is_enchantable(item: &Item) -> bool {
    !matches!(item.slot, crate::character::ItemSlot::Potion)
}

pub fn can_enchant_further(item: &Item) -> bool {
    item.enchant_level < MAX_ENCHANT_LEVEL
}

pub fn sell_price(item: &Item) -> u32 {
    let base = item_price(item);
    base / 2 + item.enchant_level * (base / 5)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::ItemSlot;

    fn make_test_item(slot: ItemSlot, power: i32, rarity: Rarity, enchant_level: u32) -> Item {
        Item {
            name: "Test".to_string(),
            slot,
            power,
            rarity,
            enchant_level,
        }
    }

    #[test]
    fn enchant_cost_at_level_zero_equals_item_price() {
        let item = make_test_item(ItemSlot::Weapon, 2, Rarity::Common, 0);
        assert_eq!(enchant_cost(&item), item_price(&item));
    }

    #[test]
    fn enchant_cost_at_level_two_is_three_times_item_price() {
        let item = make_test_item(ItemSlot::Weapon, 2, Rarity::Common, 2);
        assert_eq!(enchant_cost(&item), item_price(&item) * 3);
    }

    #[test]
    fn enchant_cost_at_level_four_is_five_times_item_price() {
        let item = make_test_item(ItemSlot::Weapon, 2, Rarity::Common, 4);
        assert_eq!(enchant_cost(&item), item_price(&item) * 5);
    }

    #[test]
    fn enchant_cost_total_for_common_max_enchant_is_15x_base() {
        let mut item = make_test_item(ItemSlot::Weapon, 2, Rarity::Common, 0);
        let mut total: u32 = 0;
        for _ in 0..5 {
            total += enchant_cost(&item);
            item.enchant_level += 1;
        }
        assert_eq!(total, item_price(&item) * 15);
    }

    #[test]
    fn is_enchantable_rejects_potions() {
        let p = make_test_item(ItemSlot::Potion, 5, Rarity::Common, 0);
        assert!(!is_enchantable(&p));
    }

    #[test]
    fn is_enchantable_accepts_weapon_armor_ring() {
        assert!(is_enchantable(&make_test_item(ItemSlot::Weapon, 1, Rarity::Common, 0)));
        assert!(is_enchantable(&make_test_item(ItemSlot::Armor, 1, Rarity::Common, 0)));
        assert!(is_enchantable(&make_test_item(ItemSlot::Ring, 1, Rarity::Common, 0)));
    }

    #[test]
    fn can_enchant_further_allows_levels_0_through_4() {
        for lvl in 0..=4 {
            let item = make_test_item(ItemSlot::Weapon, 1, Rarity::Common, lvl);
            assert!(can_enchant_further(&item), "level {lvl} should still be enchantable");
        }
    }

    #[test]
    fn can_enchant_further_blocks_at_level_5() {
        let item = make_test_item(ItemSlot::Weapon, 1, Rarity::Common, 5);
        assert!(!can_enchant_further(&item));
    }

    #[test]
    fn sell_price_at_enchant_zero_equals_half_item_price() {
        let item = make_test_item(ItemSlot::Weapon, 5, Rarity::Common, 0);
        assert_eq!(sell_price(&item), item_price(&item) / 2);
    }

    #[test]
    fn sell_price_includes_enchant_bonus_per_level() {
        let item = make_test_item(ItemSlot::Weapon, 5, Rarity::Common, 3);
        let base = item_price(&item);
        assert_eq!(sell_price(&item), base / 2 + 3 * (base / 5));
    }

    #[test]
    fn sell_price_never_exceeds_total_investment() {
        let base_item = make_test_item(ItemSlot::Weapon, 5, Rarity::Common, 0);
        let mut max_item = base_item.clone();
        max_item.enchant_level = 5;
        let buy = item_price(&base_item);
        let enchant_invested: u32 = (1..=5u32).map(|n| buy * n).sum();
        let total_invested = buy + enchant_invested;
        assert!(sell_price(&max_item) < total_invested,
            "enchant-then-sell must not be profitable");
    }

    #[test]
    fn roll_loot_returns_non_empty_name() {
        for _ in 0..20 {
            let item = roll_loot(1);
            assert!(!item.name.is_empty(), "item name should not be empty");
        }
    }

    #[test]
    fn roll_loot_returns_positive_power() {
        for _ in 0..20 {
            let item = roll_loot(1);
            assert!(item.power > 0, "item power should be positive, got {}", item.power);
        }
    }

    #[test]
    fn roll_shop_loot_never_epic_or_legendary() {
        for i in 0..1000 {
            let item = roll_shop_loot();
            match item.rarity {
                Rarity::Epic | Rarity::Legendary => {
                    panic!("shop loot returned Epic/Legendary on iteration {}: {}", i, item.name);
                }
                _ => {}
            }
        }
    }

    #[test]
    fn roll_shop_loot_returns_valid_rarity() {
        for _ in 0..50 {
            let item = roll_shop_loot();
            match item.rarity {
                Rarity::Common | Rarity::Uncommon | Rarity::Rare => {}
                _ => panic!("unexpected rarity from shop: {:?}", item.rarity),
            }
        }
    }

    #[test]
    fn item_price_scales_by_rarity() {
        let common = Item { name: "A".to_string(), slot: ItemSlot::Weapon, power: 5, rarity: Rarity::Common, enchant_level: 0 };
        let uncommon = Item { name: "B".to_string(), slot: ItemSlot::Weapon, power: 5, rarity: Rarity::Uncommon, enchant_level: 0 };
        let rare = Item { name: "C".to_string(), slot: ItemSlot::Weapon, power: 5, rarity: Rarity::Rare, enchant_level: 0 };
        let common_price = item_price(&common);
        let uncommon_price = item_price(&uncommon);
        let rare_price = item_price(&rare);
        assert!(
            common_price < uncommon_price,
            "common ({}) should be cheaper than uncommon ({})",
            common_price,
            uncommon_price
        );
        assert!(
            uncommon_price < rare_price,
            "uncommon ({}) should be cheaper than rare ({})",
            uncommon_price,
            rare_price
        );
    }

    #[test]
    fn item_price_formula_correct() {
        let item = Item { name: "X".to_string(), slot: ItemSlot::Armor, power: 3, rarity: Rarity::Common, enchant_level: 0 };
        // multiplier = 5; price = 3 * 5 + 5 = 20
        assert_eq!(item_price(&item), 20);
    }

    #[test]
    fn item_price_legendary_formula() {
        let item = Item { name: "X".to_string(), slot: ItemSlot::Weapon, power: 10, rarity: Rarity::Legendary, enchant_level: 0 };
        // multiplier = 35; price = 10 * 35 + 35 = 385
        assert_eq!(item_price(&item), 385);
    }

    #[test]
    fn rarity_multipliers_within_industry_range_common_to_legendary_ratio_56x() {
        let common = Item { name: "C".to_string(), slot: ItemSlot::Weapon, power: 1, rarity: Rarity::Common, enchant_level: 0 };
        let legendary = Item { name: "L".to_string(), slot: ItemSlot::Weapon, power: 1, rarity: Rarity::Legendary, enchant_level: 0 };
        let common_price = item_price(&common);
        let legendary_price = item_price(&legendary);
        let ratio = legendary_price as f32 / common_price as f32;
        assert!(
            ratio > 5.0 && ratio < 10.0,
            "Common→Legendary multiplier ratio must stay in industry-typical 5-10x range, got {}x",
            ratio
        );
    }

    #[test]
    fn boss_loot_never_rolls_common() {
        for _ in 0..200 {
            let item = roll_boss_loot();
            assert!(
                !matches!(item.rarity, crate::character::Rarity::Common),
                "boss loot rolled Common"
            );
        }
    }

    #[test]
    fn boss_loot_returns_valid_item() {
        let item = roll_boss_loot();
        assert!(!item.name.is_empty());
        assert!(item.power > 0);
    }
}

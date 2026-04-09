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
    Item {
        name: entry.name.to_string(),
        slot: entry.slot,
        power,
        rarity,
    }
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

/// Calculate a gold price for an item based on rarity and power.
pub fn item_price(item: &Item) -> u32 {
    let multiplier = match item.rarity {
        Rarity::Common => 5,
        Rarity::Uncommon => 10,
        Rarity::Rare => 20,
        Rarity::Epic => 40,
        Rarity::Legendary => 100,
    };
    (item.power as u32) * multiplier + multiplier
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let common = Item {
            name: "A".to_string(),
            slot: ItemSlot::Weapon,
            power: 5,
            rarity: Rarity::Common,
        };
        let uncommon = Item {
            name: "B".to_string(),
            slot: ItemSlot::Weapon,
            power: 5,
            rarity: Rarity::Uncommon,
        };
        let rare = Item {
            name: "C".to_string(),
            slot: ItemSlot::Weapon,
            power: 5,
            rarity: Rarity::Rare,
        };
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
        let item = Item {
            name: "X".to_string(),
            slot: ItemSlot::Armor,
            power: 3,
            rarity: Rarity::Common,
        };
        // multiplier = 5; price = 3 * 5 + 5 = 20
        assert_eq!(item_price(&item), 20);
    }

    #[test]
    fn item_price_legendary_formula() {
        let item = Item {
            name: "X".to_string(),
            slot: ItemSlot::Weapon,
            power: 10,
            rarity: Rarity::Legendary,
        };
        // multiplier = 100; price = 10 * 100 + 100 = 1100
        assert_eq!(item_price(&item), 1100);
    }
}

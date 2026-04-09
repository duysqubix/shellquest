use crate::character::{Item, ItemSlot, Rarity};
use rand::Rng;

struct LootEntry {
    name: &'static str,
    slot: ItemSlot,
    power_range: (i32, i32),
    rarity: Rarity,
    weight: u32,
}

const LOOT_TABLE: &[LootEntry] = &[
    // ═══════════════════════════════════════════
    //  WEAPONS
    // ═══════════════════════════════════════════

    // Common weapons
    LootEntry { name: "Rusty Pipe", slot: ItemSlot::Weapon, power_range: (1, 3), rarity: Rarity::Common, weight: 20 },
    LootEntry { name: "Keyboard of Smiting", slot: ItemSlot::Weapon, power_range: (2, 4), rarity: Rarity::Common, weight: 15 },
    LootEntry { name: "Mouse of Clicking", slot: ItemSlot::Weapon, power_range: (1, 3), rarity: Rarity::Common, weight: 15 },
    LootEntry { name: "Broken USB Stick", slot: ItemSlot::Weapon, power_range: (1, 2), rarity: Rarity::Common, weight: 18 },
    LootEntry { name: "Ethernet Whip", slot: ItemSlot::Weapon, power_range: (2, 3), rarity: Rarity::Common, weight: 16 },
    LootEntry { name: "Floppy Disk Shuriken", slot: ItemSlot::Weapon, power_range: (1, 4), rarity: Rarity::Common, weight: 14 },
    // Uncommon weapons
    LootEntry { name: "Sword of Regex", slot: ItemSlot::Weapon, power_range: (4, 7), rarity: Rarity::Uncommon, weight: 10 },
    LootEntry { name: "Axe of Grep", slot: ItemSlot::Weapon, power_range: (5, 8), rarity: Rarity::Uncommon, weight: 10 },
    LootEntry { name: "Dagger of Sed", slot: ItemSlot::Weapon, power_range: (3, 9), rarity: Rarity::Uncommon, weight: 10 },
    LootEntry { name: "Mace of Makefile", slot: ItemSlot::Weapon, power_range: (4, 8), rarity: Rarity::Uncommon, weight: 9 },
    LootEntry { name: "Bow of Bash", slot: ItemSlot::Weapon, power_range: (5, 7), rarity: Rarity::Uncommon, weight: 9 },
    LootEntry { name: "Halberd of HTTP", slot: ItemSlot::Weapon, power_range: (4, 9), rarity: Rarity::Uncommon, weight: 8 },
    LootEntry { name: "Spear of SQL", slot: ItemSlot::Weapon, power_range: (5, 8), rarity: Rarity::Uncommon, weight: 8 },
    // Rare weapons
    LootEntry { name: "Blade of Sudo", slot: ItemSlot::Weapon, power_range: (8, 12), rarity: Rarity::Rare, weight: 5 },
    LootEntry { name: "Staff of Stack Overflow", slot: ItemSlot::Weapon, power_range: (7, 13), rarity: Rarity::Rare, weight: 5 },
    LootEntry { name: "Hammer of Compiler", slot: ItemSlot::Weapon, power_range: (9, 14), rarity: Rarity::Rare, weight: 4 },
    LootEntry { name: "Trident of TypeScript", slot: ItemSlot::Weapon, power_range: (8, 13), rarity: Rarity::Rare, weight: 4 },
    LootEntry { name: "Scythe of Segfault", slot: ItemSlot::Weapon, power_range: (10, 15), rarity: Rarity::Rare, weight: 3 },
    LootEntry { name: "Wand of WebSocket", slot: ItemSlot::Weapon, power_range: (7, 12), rarity: Rarity::Rare, weight: 4 },
    // Epic weapons
    LootEntry { name: "Excalibash", slot: ItemSlot::Weapon, power_range: (13, 18), rarity: Rarity::Epic, weight: 2 },
    LootEntry { name: "The Mass Migration Blade", slot: ItemSlot::Weapon, power_range: (14, 20), rarity: Rarity::Epic, weight: 2 },
    LootEntry { name: "Vorpal Pointer", slot: ItemSlot::Weapon, power_range: (15, 22), rarity: Rarity::Epic, weight: 2 },
    LootEntry { name: "Mjolnir of Monorepo", slot: ItemSlot::Weapon, power_range: (16, 21), rarity: Rarity::Epic, weight: 1 },
    LootEntry { name: "Gungnir of Git Rebase", slot: ItemSlot::Weapon, power_range: (14, 23), rarity: Rarity::Epic, weight: 1 },
    // Legendary weapons
    LootEntry { name: "Mass Migration Sword of Chaos", slot: ItemSlot::Weapon, power_range: (20, 30), rarity: Rarity::Legendary, weight: 1 },
    LootEntry { name: "Mass Migration Blade of the Kernel", slot: ItemSlot::Weapon, power_range: (25, 35), rarity: Rarity::Legendary, weight: 1 },
    LootEntry { name: "Mass Migration Fork Bomb", slot: ItemSlot::Weapon, power_range: (22, 32), rarity: Rarity::Legendary, weight: 1 },

    // ═══════════════════════════════════════════
    //  ARMOR
    // ═══════════════════════════════════════════

    // Common armor
    LootEntry { name: "Hoodie of Comfort", slot: ItemSlot::Armor, power_range: (1, 3), rarity: Rarity::Common, weight: 20 },
    LootEntry { name: "T-Shirt of Localhost", slot: ItemSlot::Armor, power_range: (1, 2), rarity: Rarity::Common, weight: 15 },
    LootEntry { name: "Pajama Pants of WFH", slot: ItemSlot::Armor, power_range: (1, 3), rarity: Rarity::Common, weight: 16 },
    LootEntry { name: "Flip-Flops of Friday Deploy", slot: ItemSlot::Armor, power_range: (1, 2), rarity: Rarity::Common, weight: 14 },
    LootEntry { name: "Beanie of Bluetooth", slot: ItemSlot::Armor, power_range: (2, 3), rarity: Rarity::Common, weight: 13 },
    // Uncommon armor
    LootEntry { name: "Cloak of Stdout", slot: ItemSlot::Armor, power_range: (3, 6), rarity: Rarity::Uncommon, weight: 10 },
    LootEntry { name: "Vest of Version Control", slot: ItemSlot::Armor, power_range: (4, 7), rarity: Rarity::Uncommon, weight: 10 },
    LootEntry { name: "Gauntlets of Gzip", slot: ItemSlot::Armor, power_range: (3, 6), rarity: Rarity::Uncommon, weight: 9 },
    LootEntry { name: "Helmet of HTTPS", slot: ItemSlot::Armor, power_range: (4, 7), rarity: Rarity::Uncommon, weight: 8 },
    LootEntry { name: "Boots of Bootstrap", slot: ItemSlot::Armor, power_range: (3, 5), rarity: Rarity::Uncommon, weight: 9 },
    LootEntry { name: "Cape of CI/CD", slot: ItemSlot::Armor, power_range: (5, 7), rarity: Rarity::Uncommon, weight: 7 },
    // Rare armor
    LootEntry { name: "Chestplate of Chmod 777", slot: ItemSlot::Armor, power_range: (7, 11), rarity: Rarity::Rare, weight: 5 },
    LootEntry { name: "Plate of the Firewall", slot: ItemSlot::Armor, power_range: (8, 12), rarity: Rarity::Rare, weight: 4 },
    LootEntry { name: "Breastplate of Bcrypt", slot: ItemSlot::Armor, power_range: (9, 13), rarity: Rarity::Rare, weight: 4 },
    LootEntry { name: "Shield of CORS", slot: ItemSlot::Armor, power_range: (7, 11), rarity: Rarity::Rare, weight: 4 },
    LootEntry { name: "Greaves of GraphQL", slot: ItemSlot::Armor, power_range: (8, 12), rarity: Rarity::Rare, weight: 3 },
    // Epic armor
    LootEntry { name: "Armor of the Container", slot: ItemSlot::Armor, power_range: (12, 17), rarity: Rarity::Epic, weight: 2 },
    LootEntry { name: "Aegis of the Load Balancer", slot: ItemSlot::Armor, power_range: (14, 19), rarity: Rarity::Epic, weight: 2 },
    LootEntry { name: "Warplate of Kubernetes", slot: ItemSlot::Armor, power_range: (13, 18), rarity: Rarity::Epic, weight: 1 },
    // Legendary armor
    LootEntry { name: "Shell of Invulnerability", slot: ItemSlot::Armor, power_range: (18, 25), rarity: Rarity::Legendary, weight: 1 },
    LootEntry { name: "Divine Armor of /dev/null", slot: ItemSlot::Armor, power_range: (20, 28), rarity: Rarity::Legendary, weight: 1 },

    // ═══════════════════════════════════════════
    //  RINGS
    // ═══════════════════════════════════════════

    // Common rings
    LootEntry { name: "Ring of Tab Completion", slot: ItemSlot::Ring, power_range: (1, 3), rarity: Rarity::Common, weight: 15 },
    LootEntry { name: "Band of Backspace", slot: ItemSlot::Ring, power_range: (1, 2), rarity: Rarity::Common, weight: 14 },
    LootEntry { name: "Loop of Localhost", slot: ItemSlot::Ring, power_range: (1, 3), rarity: Rarity::Common, weight: 13 },
    // Uncommon rings
    LootEntry { name: "Ring of Syntax Highlight", slot: ItemSlot::Ring, power_range: (2, 5), rarity: Rarity::Uncommon, weight: 8 },
    LootEntry { name: "Signet of SSH", slot: ItemSlot::Ring, power_range: (3, 6), rarity: Rarity::Uncommon, weight: 7 },
    LootEntry { name: "Circlet of Cron", slot: ItemSlot::Ring, power_range: (3, 5), rarity: Rarity::Uncommon, weight: 7 },
    // Rare rings
    LootEntry { name: "Ring of the Daemon", slot: ItemSlot::Ring, power_range: (4, 8), rarity: Rarity::Rare, weight: 4 },
    LootEntry { name: "Seal of Semaphore", slot: ItemSlot::Ring, power_range: (5, 9), rarity: Rarity::Rare, weight: 3 },
    LootEntry { name: "Band of the Borrow Checker", slot: ItemSlot::Ring, power_range: (6, 10), rarity: Rarity::Rare, weight: 3 },
    // Epic rings
    LootEntry { name: "Ring of Root Access", slot: ItemSlot::Ring, power_range: (8, 14), rarity: Rarity::Epic, weight: 2 },
    LootEntry { name: "Signet of Zero-Day", slot: ItemSlot::Ring, power_range: (10, 16), rarity: Rarity::Epic, weight: 1 },
    // Legendary rings
    LootEntry { name: "The One Ring (of SSH Keys)", slot: ItemSlot::Ring, power_range: (14, 20), rarity: Rarity::Legendary, weight: 1 },
    LootEntry { name: "Eternal Band of Uptime", slot: ItemSlot::Ring, power_range: (16, 22), rarity: Rarity::Legendary, weight: 1 },

    // ═══════════════════════════════════════════
    //  POTIONS
    // ═══════════════════════════════════════════

    // Common potions
    LootEntry { name: "Potion of Coffee", slot: ItemSlot::Potion, power_range: (5, 10), rarity: Rarity::Common, weight: 20 },
    LootEntry { name: "Vial of Green Tea", slot: ItemSlot::Potion, power_range: (3, 8), rarity: Rarity::Common, weight: 16 },
    LootEntry { name: "Flask of Water (Stay Hydrated)", slot: ItemSlot::Potion, power_range: (4, 7), rarity: Rarity::Common, weight: 15 },
    // Uncommon potions
    LootEntry { name: "Elixir of Energy Drink", slot: ItemSlot::Potion, power_range: (10, 20), rarity: Rarity::Uncommon, weight: 10 },
    LootEntry { name: "Brew of Debugging", slot: ItemSlot::Potion, power_range: (12, 18), rarity: Rarity::Uncommon, weight: 8 },
    LootEntry { name: "Tincture of Focus Mode", slot: ItemSlot::Potion, power_range: (8, 15), rarity: Rarity::Uncommon, weight: 9 },
    // Rare potions
    LootEntry { name: "Flask of Liquid Nitrogen", slot: ItemSlot::Potion, power_range: (20, 35), rarity: Rarity::Rare, weight: 5 },
    LootEntry { name: "Draught of Deep Work", slot: ItemSlot::Potion, power_range: (18, 30), rarity: Rarity::Rare, weight: 4 },
    // Epic potions
    LootEntry { name: "Phoenix Elixir of Hot Reload", slot: ItemSlot::Potion, power_range: (30, 50), rarity: Rarity::Epic, weight: 2 },
    // Legendary potions
    LootEntry { name: "Mass Migration Elixir of Infinite Context", slot: ItemSlot::Potion, power_range: (50, 99), rarity: Rarity::Legendary, weight: 1 },
];

pub fn roll_loot(danger_level: u32) -> Item {
    let mut rng = rand::thread_rng();

    // Higher danger = better loot chance (filter out some common items)
    let eligible: Vec<&LootEntry> = LOOT_TABLE
        .iter()
        .filter(|entry| {
            let rarity_val = match entry.rarity {
                Rarity::Common => 1,
                Rarity::Uncommon => 2,
                Rarity::Rare => 3,
                Rarity::Epic => 4,
                Rarity::Legendary => 5,
            };
            // Allow items up to danger_level + 2 in rarity
            rarity_val <= (danger_level + 2) as u8
        })
        .collect();

    let total_weight: u32 = eligible.iter().map(|e| e.weight).sum();
    let mut roll = rng.gen_range(0..total_weight);

    for entry in &eligible {
        if roll < entry.weight {
            let power = rng.gen_range(entry.power_range.0..=entry.power_range.1);
            return Item {
                name: entry.name.to_string(),
                slot: entry.slot,
                power,
                rarity: entry.rarity,
            };
        }
        roll -= entry.weight;
    }

    // Fallback
    Item {
        name: "Mysterious Byte".to_string(),
        slot: ItemSlot::Potion,
        power: 1,
        rarity: Rarity::Common,
    }
}

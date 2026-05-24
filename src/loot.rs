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

// ===============================================================
//  COMMON ITEMS - the everyday clutter (mundane office supplies and stale snacks)
// ===============================================================

const COMMON: &[LootEntry] = &[
    // Weapons
    LootEntry { name: "Rusty Crowbar", slot: ItemSlot::Weapon, power_range: (1, 2) },
    LootEntry { name: "Dull Letter Opener", slot: ItemSlot::Weapon, power_range: (1, 2) },
    LootEntry { name: "Chipped Screwdriver", slot: ItemSlot::Weapon, power_range: (2, 3) },
    LootEntry { name: "Bent Fork", slot: ItemSlot::Weapon, power_range: (2, 3) },
    LootEntry { name: "Heavy Stapler", slot: ItemSlot::Weapon, power_range: (2, 3) },
    LootEntry { name: "Wooden Ruler", slot: ItemSlot::Weapon, power_range: (2, 4) },
    LootEntry { name: "Plastic Spork", slot: ItemSlot::Weapon, power_range: (3, 4) },
    LootEntry { name: "Blunt Pencil", slot: ItemSlot::Weapon, power_range: (3, 4) },
    // Armors
    LootEntry { name: "Coffee-Stained Hoodie", slot: ItemSlot::Armor, power_range: (1, 2) },
    LootEntry { name: "Tattered T-shirt", slot: ItemSlot::Armor, power_range: (1, 2) },
    LootEntry { name: "Cardboard Box", slot: ItemSlot::Armor, power_range: (2, 3) },
    LootEntry { name: "Plastic Poncho", slot: ItemSlot::Armor, power_range: (2, 3) },
    LootEntry { name: "Thick Denim Jacket", slot: ItemSlot::Armor, power_range: (2, 3) },
    LootEntry { name: "Old Sneakers", slot: ItemSlot::Armor, power_range: (2, 4) },
    LootEntry { name: "Frayed Beanie", slot: ItemSlot::Armor, power_range: (3, 4) },
    LootEntry { name: "Stained Apron", slot: ItemSlot::Armor, power_range: (3, 4) },
    // Rings
    LootEntry { name: "Copper Washer", slot: ItemSlot::Ring, power_range: (1, 2) },
    LootEntry { name: "Plastic Zip-tie", slot: ItemSlot::Ring, power_range: (1, 2) },
    LootEntry { name: "Rubber Band", slot: ItemSlot::Ring, power_range: (2, 3) },
    LootEntry { name: "String Loop", slot: ItemSlot::Ring, power_range: (2, 3) },
    LootEntry { name: "Rusty Nut", slot: ItemSlot::Ring, power_range: (2, 3) },
    LootEntry { name: "Glass Bead", slot: ItemSlot::Ring, power_range: (2, 4) },
    LootEntry { name: "Paperclip Ring", slot: ItemSlot::Ring, power_range: (3, 4) },
    LootEntry { name: "Soda Tab", slot: ItemSlot::Ring, power_range: (3, 4) },
    // Potions
    LootEntry { name: "Lukewarm Water", slot: ItemSlot::Potion, power_range: (1, 2) },
    LootEntry { name: "Stale Coffee", slot: ItemSlot::Potion, power_range: (1, 2) },
    LootEntry { name: "Flat Soda", slot: ItemSlot::Potion, power_range: (2, 3) },
    LootEntry { name: "Generic Energy Drink", slot: ItemSlot::Potion, power_range: (2, 3) },
    LootEntry { name: "Tap Water", slot: ItemSlot::Potion, power_range: (2, 3) },
    LootEntry { name: "Half-Empty Juice Box", slot: ItemSlot::Potion, power_range: (2, 4) },
    LootEntry { name: "Cold Soup", slot: ItemSlot::Potion, power_range: (3, 4) },
    LootEntry { name: "Expired Milk", slot: ItemSlot::Potion, power_range: (3, 4) },
];

// ===============================================================
//  UNCOMMON ITEMS - solid upgrades (real tools and decent provisions)
// ===============================================================

const UNCOMMON: &[LootEntry] = &[
    // Weapons
    LootEntry { name: "Polished Wrench", slot: ItemSlot::Weapon, power_range: (3, 4) },
    LootEntry { name: "Sharp Utility Knife", slot: ItemSlot::Weapon, power_range: (3, 4) },
    LootEntry { name: "Weighted Hammer", slot: ItemSlot::Weapon, power_range: (3, 5) },
    LootEntry { name: "Steel-Tipped Pen", slot: ItemSlot::Weapon, power_range: (4, 5) },
    LootEntry { name: "Industrial Scissors", slot: ItemSlot::Weapon, power_range: (4, 5) },
    LootEntry { name: "Brass Knuckles", slot: ItemSlot::Weapon, power_range: (4, 6) },
    LootEntry { name: "Iron Pipe", slot: ItemSlot::Weapon, power_range: (5, 6) },
    LootEntry { name: "Sharpened Spatula", slot: ItemSlot::Weapon, power_range: (5, 6) },
    // Armors
    LootEntry { name: "Reinforced Vest", slot: ItemSlot::Armor, power_range: (3, 4) },
    LootEntry { name: "Padded Windbreaker", slot: ItemSlot::Armor, power_range: (3, 4) },
    LootEntry { name: "Leather Apron", slot: ItemSlot::Armor, power_range: (3, 5) },
    LootEntry { name: "Hard Hat", slot: ItemSlot::Armor, power_range: (4, 5) },
    LootEntry { name: "Work Gloves", slot: ItemSlot::Armor, power_range: (4, 5) },
    LootEntry { name: "Cargo Pants", slot: ItemSlot::Armor, power_range: (4, 6) },
    LootEntry { name: "Steel-Toed Boots", slot: ItemSlot::Armor, power_range: (5, 6) },
    LootEntry { name: "Safety Goggles", slot: ItemSlot::Armor, power_range: (5, 6) },
    // Rings
    LootEntry { name: "Silver Band", slot: ItemSlot::Ring, power_range: (3, 4) },
    LootEntry { name: "Polished Brass Ring", slot: ItemSlot::Ring, power_range: (3, 4) },
    LootEntry { name: "Iron Signet", slot: ItemSlot::Ring, power_range: (3, 5) },
    LootEntry { name: "Braided Wire Loop", slot: ItemSlot::Ring, power_range: (4, 5) },
    LootEntry { name: "Quartz Ring", slot: ItemSlot::Ring, power_range: (4, 5) },
    LootEntry { name: "Steel Washer", slot: ItemSlot::Ring, power_range: (4, 6) },
    LootEntry { name: "Polished Pebble", slot: ItemSlot::Ring, power_range: (5, 6) },
    LootEntry { name: "Copper Coil", slot: ItemSlot::Ring, power_range: (5, 6) },
    // Potions
    LootEntry { name: "Fresh Espresso", slot: ItemSlot::Potion, power_range: (3, 4) },
    LootEntry { name: "Cold Brew", slot: ItemSlot::Potion, power_range: (3, 4) },
    LootEntry { name: "Vitamin Water", slot: ItemSlot::Potion, power_range: (3, 5) },
    LootEntry { name: "Electrolyte Drink", slot: ItemSlot::Potion, power_range: (4, 5) },
    LootEntry { name: "Herbal Tea", slot: ItemSlot::Potion, power_range: (4, 5) },
    LootEntry { name: "Protein Shake", slot: ItemSlot::Potion, power_range: (4, 6) },
    LootEntry { name: "Bottled Water", slot: ItemSlot::Potion, power_range: (5, 6) },
    LootEntry { name: "Fruit Smoothie", slot: ItemSlot::Potion, power_range: (5, 6) },
];

// ===============================================================
//  RARE ITEMS - named tools with single-word evocative concepts
// ===============================================================

const RARE: &[LootEntry] = &[
    // Weapons
    LootEntry { name: "Bit-Blitzer", slot: ItemSlot::Weapon, power_range: (5, 6) },
    LootEntry { name: "Logic-Lash", slot: ItemSlot::Weapon, power_range: (5, 7) },
    LootEntry { name: "Syntax-Slicer", slot: ItemSlot::Weapon, power_range: (6, 7) },
    LootEntry { name: "Code-Cracker", slot: ItemSlot::Weapon, power_range: (6, 8) },
    LootEntry { name: "Data-Dagger", slot: ItemSlot::Weapon, power_range: (7, 8) },
    LootEntry { name: "Thread-Thresher", slot: ItemSlot::Weapon, power_range: (7, 9) },
    LootEntry { name: "Memory-Maul", slot: ItemSlot::Weapon, power_range: (8, 10) },
    LootEntry { name: "Buffer-Blade", slot: ItemSlot::Weapon, power_range: (9, 10) },
    // Armors
    LootEntry { name: "Buffer-Bulwark", slot: ItemSlot::Armor, power_range: (5, 6) },
    LootEntry { name: "Shell-Shield", slot: ItemSlot::Armor, power_range: (5, 7) },
    LootEntry { name: "Script-Shroud", slot: ItemSlot::Armor, power_range: (6, 7) },
    LootEntry { name: "Logic-Layer", slot: ItemSlot::Armor, power_range: (6, 8) },
    LootEntry { name: "Data-Drape", slot: ItemSlot::Armor, power_range: (7, 8) },
    LootEntry { name: "Thread-Tunic", slot: ItemSlot::Armor, power_range: (7, 9) },
    LootEntry { name: "Memory-Mail", slot: ItemSlot::Armor, power_range: (8, 10) },
    LootEntry { name: "Kernel-Cloak", slot: ItemSlot::Armor, power_range: (9, 10) },
    // Rings
    LootEntry { name: "Loop-Link", slot: ItemSlot::Ring, power_range: (5, 6) },
    LootEntry { name: "Node-Nexus", slot: ItemSlot::Ring, power_range: (5, 7) },
    LootEntry { name: "Bit-Bind", slot: ItemSlot::Ring, power_range: (6, 7) },
    LootEntry { name: "Code-Coil", slot: ItemSlot::Ring, power_range: (6, 8) },
    LootEntry { name: "Data-Dot", slot: ItemSlot::Ring, power_range: (7, 8) },
    LootEntry { name: "Thread-Tie", slot: ItemSlot::Ring, power_range: (7, 9) },
    LootEntry { name: "Memory-Mark", slot: ItemSlot::Ring, power_range: (8, 10) },
    LootEntry { name: "Kernel-Key", slot: ItemSlot::Ring, power_range: (9, 10) },
    // Potions
    LootEntry { name: "Logic-Litre", slot: ItemSlot::Potion, power_range: (5, 6) },
    LootEntry { name: "Syntax-Sip", slot: ItemSlot::Potion, power_range: (5, 7) },
    LootEntry { name: "Code-Cordial", slot: ItemSlot::Potion, power_range: (6, 7) },
    LootEntry { name: "Data-Draught", slot: ItemSlot::Potion, power_range: (6, 8) },
    LootEntry { name: "Thread-Tonic", slot: ItemSlot::Potion, power_range: (7, 8) },
    LootEntry { name: "Bit-Brew", slot: ItemSlot::Potion, power_range: (7, 9) },
    LootEntry { name: "Memory-Mist", slot: ItemSlot::Potion, power_range: (8, 10) },
    LootEntry { name: "Kernel-Keg", slot: ItemSlot::Potion, power_range: (9, 10) },
];

// ===============================================================
//  EPIC ITEMS - darker fantasy, tier-specific multi-word artifacts
// ===============================================================

const EPIC: &[LootEntry] = &[
    // Weapons
    LootEntry { name: "Void-Pointer Voulge", slot: ItemSlot::Weapon, power_range: (8, 10) },
    LootEntry { name: "Heap-Stalker Harpoon", slot: ItemSlot::Weapon, power_range: (8, 11) },
    LootEntry { name: "Stack-Smasher Sledge", slot: ItemSlot::Weapon, power_range: (9, 11) },
    LootEntry { name: "Mutex-Mangler Mace", slot: ItemSlot::Weapon, power_range: (10, 12) },
    LootEntry { name: "Segfault Scythe", slot: ItemSlot::Weapon, power_range: (10, 13) },
    LootEntry { name: "Kernel-Cutter Katana", slot: ItemSlot::Weapon, power_range: (11, 14) },
    LootEntry { name: "Runtime-Ravager Rapier", slot: ItemSlot::Weapon, power_range: (12, 14) },
    LootEntry { name: "Compiler-Crushing Claymore", slot: ItemSlot::Weapon, power_range: (13, 15) },
    // Armors
    LootEntry { name: "Mantle of the Mainframe", slot: ItemSlot::Armor, power_range: (8, 10) },
    LootEntry { name: "Robes of the Root", slot: ItemSlot::Armor, power_range: (8, 11) },
    LootEntry { name: "Plate of the Processor", slot: ItemSlot::Armor, power_range: (9, 11) },
    LootEntry { name: "Vestments of the Virtual", slot: ItemSlot::Armor, power_range: (10, 12) },
    LootEntry { name: "Cloak of the Compiler", slot: ItemSlot::Armor, power_range: (10, 13) },
    LootEntry { name: "Armor of the Assembler", slot: ItemSlot::Armor, power_range: (11, 14) },
    LootEntry { name: "Guard of the Garbage-Collector", slot: ItemSlot::Armor, power_range: (12, 14) },
    LootEntry { name: "Suit of the Superuser", slot: ItemSlot::Armor, power_range: (13, 15) },
    // Rings
    LootEntry { name: "Sigil of the Sysadmin", slot: ItemSlot::Ring, power_range: (8, 10) },
    LootEntry { name: "Band of the Binary", slot: ItemSlot::Ring, power_range: (8, 11) },
    LootEntry { name: "Loop of the Low-Level", slot: ItemSlot::Ring, power_range: (9, 11) },
    LootEntry { name: "Amulet of the Architect", slot: ItemSlot::Ring, power_range: (10, 12) },
    LootEntry { name: "Ring of the Runtime", slot: ItemSlot::Ring, power_range: (10, 13) },
    LootEntry { name: "Coil of the Core", slot: ItemSlot::Ring, power_range: (11, 14) },
    LootEntry { name: "Mark of the Mutex", slot: ItemSlot::Ring, power_range: (12, 14) },
    LootEntry { name: "Signet of the Socket", slot: ItemSlot::Ring, power_range: (13, 15) },
    // Potions
    LootEntry { name: "Essence of the Epoch", slot: ItemSlot::Potion, power_range: (8, 10) },
    LootEntry { name: "Distillation of the Debugger", slot: ItemSlot::Potion, power_range: (8, 11) },
    LootEntry { name: "Brew of the Binary", slot: ItemSlot::Potion, power_range: (9, 11) },
    LootEntry { name: "Elixir of the Executable", slot: ItemSlot::Potion, power_range: (10, 12) },
    LootEntry { name: "Vial of the Virtual", slot: ItemSlot::Potion, power_range: (10, 13) },
    LootEntry { name: "Draught of the Daemon", slot: ItemSlot::Potion, power_range: (11, 14) },
    LootEntry { name: "Tonic of the Terminal", slot: ItemSlot::Potion, power_range: (12, 14) },
    LootEntry { name: "Sip of the System", slot: ItemSlot::Potion, power_range: (13, 15) },
];

// ===============================================================
//  LEGENDARY ITEMS - once in a lifetime - 'The X-of-Y' construction, eyeball-grabbing
// ===============================================================

const LEGENDARY: &[LootEntry] = &[
    // Weapons
    LootEntry { name: "The Final Commit of Fate", slot: ItemSlot::Weapon, power_range: (15, 17) },
    LootEntry { name: "The Last Allocator's Wrath", slot: ItemSlot::Weapon, power_range: (16, 18) },
    LootEntry { name: "The Blade of the Broken Build", slot: ItemSlot::Weapon, power_range: (17, 19) },
    LootEntry { name: "The Hammer of the Hard Reset", slot: ItemSlot::Weapon, power_range: (18, 20) },
    LootEntry { name: "The Spear of the Single Source", slot: ItemSlot::Weapon, power_range: (19, 21) },
    LootEntry { name: "The Bow of the Binary Search", slot: ItemSlot::Weapon, power_range: (20, 22) },
    LootEntry { name: "The Axe of the Absolute Zero", slot: ItemSlot::Weapon, power_range: (21, 23) },
    LootEntry { name: "The Dagger of the Deep Copy", slot: ItemSlot::Weapon, power_range: (22, 25) },
    // Armors
    LootEntry { name: "The Shroud of the Silent Error", slot: ItemSlot::Armor, power_range: (15, 17) },
    LootEntry { name: "The Aegis of the Absolute Path", slot: ItemSlot::Armor, power_range: (16, 18) },
    LootEntry { name: "The Mantle of the Master Branch", slot: ItemSlot::Armor, power_range: (17, 19) },
    LootEntry { name: "The Plate of the Persistent State", slot: ItemSlot::Armor, power_range: (18, 20) },
    LootEntry { name: "The Robes of the Recursive Call", slot: ItemSlot::Armor, power_range: (19, 21) },
    LootEntry { name: "The Vest of the Validated Input", slot: ItemSlot::Armor, power_range: (20, 22) },
    LootEntry { name: "The Cloak of the Clean Code", slot: ItemSlot::Armor, power_range: (21, 23) },
    LootEntry { name: "The Armor of the Atomic Operation", slot: ItemSlot::Armor, power_range: (22, 25) },
    // Rings
    LootEntry { name: "The Loop of the Lost Link", slot: ItemSlot::Ring, power_range: (15, 17) },
    LootEntry { name: "The Sigil of the Sovereign System", slot: ItemSlot::Ring, power_range: (16, 18) },
    LootEntry { name: "The Band of the Boundless Buffer", slot: ItemSlot::Ring, power_range: (17, 19) },
    LootEntry { name: "The Ring of the Root Access", slot: ItemSlot::Ring, power_range: (18, 20) },
    LootEntry { name: "The Amulet of the Async Await", slot: ItemSlot::Ring, power_range: (19, 21) },
    LootEntry { name: "The Coil of the Constant Time", slot: ItemSlot::Ring, power_range: (20, 22) },
    LootEntry { name: "The Signet of the Singleton", slot: ItemSlot::Ring, power_range: (21, 23) },
    LootEntry { name: "The Mark of the Memory Map", slot: ItemSlot::Ring, power_range: (22, 25) },
    // Potions
    LootEntry { name: "The Tear of the Tired Tech-Lead", slot: ItemSlot::Potion, power_range: (15, 17) },
    LootEntry { name: "The Blood of the Beta Tester", slot: ItemSlot::Potion, power_range: (16, 18) },
    LootEntry { name: "The Sweat of the Senior Dev", slot: ItemSlot::Potion, power_range: (17, 19) },
    LootEntry { name: "The Breath of the Backend", slot: ItemSlot::Potion, power_range: (18, 20) },
    LootEntry { name: "The Soul of the Source Code", slot: ItemSlot::Potion, power_range: (19, 21) },
    LootEntry { name: "The Heart of the Hardware", slot: ItemSlot::Potion, power_range: (20, 22) },
    LootEntry { name: "The Essence of the End-User", slot: ItemSlot::Potion, power_range: (21, 23) },
    LootEntry { name: "The Draught of the Deployment", slot: ItemSlot::Potion, power_range: (22, 25) },
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

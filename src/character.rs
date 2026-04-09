use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Class {
    Wizard,
    Warrior,
    Rogue,
    Ranger,
    Necromancer,
}

impl Class {
    pub fn base_stats(&self) -> (i32, i32, i32) {
        // (STR, DEX, INT)
        match self {
            Class::Wizard => (6, 8, 16),
            Class::Warrior => (16, 8, 6),
            Class::Rogue => (8, 16, 6),
            Class::Ranger => (10, 14, 6),
            Class::Necromancer => (6, 6, 18),
        }
    }
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Class::Wizard => write!(f, "Wizard"),
            Class::Warrior => write!(f, "Warrior"),
            Class::Rogue => write!(f, "Rogue"),
            Class::Ranger => write!(f, "Ranger"),
            Class::Necromancer => write!(f, "Necromancer"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Race {
    Human,
    Elf,
    Dwarf,
    Orc,
    Goblin,
}

impl Race {
    pub fn stat_bonus(&self) -> (i32, i32, i32) {
        // (STR, DEX, INT)
        match self {
            Race::Human => (1, 1, 1),
            Race::Elf => (0, 2, 2),
            Race::Dwarf => (3, 0, 1),
            Race::Orc => (4, 1, -1),
            Race::Goblin => (-1, 3, 1),
        }
    }
}

impl fmt::Display for Race {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Race::Human => write!(f, "Human"),
            Race::Elf => write!(f, "Elf"),
            Race::Dwarf => write!(f, "Dwarf"),
            Race::Orc => write!(f, "Orc"),
            Race::Goblin => write!(f, "Goblin"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub slot: ItemSlot,
    pub power: i32,
    pub rarity: Rarity,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ItemSlot {
    Weapon,
    Armor,
    Ring,
    Potion,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl fmt::Display for Rarity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rarity::Common => write!(f, "Common"),
            Rarity::Uncommon => write!(f, "Uncommon"),
            Rarity::Rare => write!(f, "Rare"),
            Rarity::Epic => write!(f, "Epic"),
            Rarity::Legendary => write!(f, "Legendary"),
        }
    }
}

impl fmt::Display for ItemSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemSlot::Weapon => write!(f, "Weapon"),
            ItemSlot::Armor => write!(f, "Armor"),
            ItemSlot::Ring => write!(f, "Ring"),
            ItemSlot::Potion => write!(f, "Potion"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Subclass {
    None,
    // Wizard subclasses
    Archmage,
    Chronomancer,
    Datamancer,
    // Warrior subclasses
    Berserker,
    Paladin,
    Warlord,
    // Rogue subclasses
    Assassin,
    Hacker,
    Shadow,
    // Ranger subclasses
    Beastmaster,
    Sniper,
    Scout,
    // Necromancer subclasses
    Lich,
    Plaguebearer,
    SoulReaper,
}

impl Subclass {
    pub fn stat_bonus(&self) -> (i32, i32, i32) {
        // (STR, DEX, INT) per prestige tier
        match self {
            Subclass::None => (0, 0, 0),
            Subclass::Archmage => (0, 1, 3),
            Subclass::Chronomancer => (0, 2, 2),
            Subclass::Datamancer => (1, 1, 2),
            Subclass::Berserker => (3, 1, 0),
            Subclass::Paladin => (2, 0, 2),
            Subclass::Warlord => (2, 1, 1),
            Subclass::Assassin => (1, 3, 0),
            Subclass::Hacker => (0, 2, 2),
            Subclass::Shadow => (1, 3, 0),
            Subclass::Beastmaster => (2, 2, 0),
            Subclass::Sniper => (0, 3, 1),
            Subclass::Scout => (1, 2, 1),
            Subclass::Lich => (0, 0, 4),
            Subclass::Plaguebearer => (1, 0, 3),
            Subclass::SoulReaper => (2, 0, 2),
        }
    }

    pub fn available_for(class: &Class) -> Vec<Subclass> {
        match class {
            Class::Wizard => vec![Subclass::Archmage, Subclass::Chronomancer, Subclass::Datamancer],
            Class::Warrior => vec![Subclass::Berserker, Subclass::Paladin, Subclass::Warlord],
            Class::Rogue => vec![Subclass::Assassin, Subclass::Hacker, Subclass::Shadow],
            Class::Ranger => vec![Subclass::Beastmaster, Subclass::Sniper, Subclass::Scout],
            Class::Necromancer => vec![Subclass::Lich, Subclass::Plaguebearer, Subclass::SoulReaper],
        }
    }
}

impl fmt::Display for Subclass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Subclass::None => write!(f, ""),
            Subclass::Archmage => write!(f, "Archmage"),
            Subclass::Chronomancer => write!(f, "Chronomancer"),
            Subclass::Datamancer => write!(f, "Datamancer"),
            Subclass::Berserker => write!(f, "Berserker"),
            Subclass::Paladin => write!(f, "Paladin"),
            Subclass::Warlord => write!(f, "Warlord"),
            Subclass::Assassin => write!(f, "Assassin"),
            Subclass::Hacker => write!(f, "Hacker"),
            Subclass::Shadow => write!(f, "Shadow"),
            Subclass::Beastmaster => write!(f, "Beastmaster"),
            Subclass::Sniper => write!(f, "Sniper"),
            Subclass::Scout => write!(f, "Scout"),
            Subclass::Lich => write!(f, "Lich"),
            Subclass::Plaguebearer => write!(f, "Plaguebearer"),
            Subclass::SoulReaper => write!(f, "Soul Reaper"),
        }
    }
}

pub const MAX_LEVEL: u32 = 150;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub class: Class,
    pub race: Race,
    pub level: u32,
    pub xp: u32,
    pub xp_to_next: u32,
    pub hp: i32,
    pub max_hp: i32,
    pub strength: i32,
    pub dexterity: i32,
    pub intelligence: i32,
    pub gold: u32,
    pub kills: u32,
    pub deaths: u32,
    pub commands_run: u64,
    pub weapon: Option<Item>,
    pub armor: Option<Item>,
    pub ring: Option<Item>,
    pub inventory: Vec<Item>,
    pub title: String,
    #[serde(default)]
    pub prestige: u32,
    #[serde(default)]
    pub subclass: Option<Subclass>,
    #[serde(default)]
    pub total_prestiges: u32,
}

impl Character {
    pub fn new(name: String, class: Class, race: Race) -> Self {
        let (base_str, base_dex, base_int) = class.base_stats();
        let (bonus_str, bonus_dex, bonus_int) = race.stat_bonus();
        let strength = base_str + bonus_str;
        let dexterity = base_dex + bonus_dex;
        let intelligence = base_int + bonus_int;
        let max_hp = 20 + (strength * 2);

        Character {
            name,
            class,
            race,
            level: 1,
            xp: 0,
            xp_to_next: 100,
            hp: max_hp,
            max_hp,
            strength,
            dexterity,
            intelligence,
            gold: 10,
            kills: 0,
            deaths: 0,
            commands_run: 0,
            weapon: None,
            armor: None,
            ring: None,
            inventory: Vec::new(),
            title: "Terminal Novice".to_string(),
            prestige: 0,
            subclass: None,
            total_prestiges: 0,
        }
    }

    pub fn attack_power(&self) -> i32 {
        let base = self.strength + (self.dexterity / 2);
        let weapon_bonus = self.weapon.as_ref().map_or(0, |w| w.power);
        base + weapon_bonus
    }

    pub fn defense(&self) -> i32 {
        let base = self.dexterity / 2;
        let armor_bonus = self.armor.as_ref().map_or(0, |a| a.power);
        let ring_bonus = self.ring.as_ref().map_or(0, |r| r.power);
        base + armor_bonus + ring_bonus
    }

    pub fn gain_xp(&mut self, amount: u32) -> bool {
        if self.level >= MAX_LEVEL {
            return false; // At max level, prestige to continue
        }
        self.xp += amount;
        if self.xp >= self.xp_to_next {
            self.level_up();
            return true;
        }
        false
    }

    fn level_up(&mut self) {
        self.xp -= self.xp_to_next;
        self.level += 1;
        // XP curve: starts easy, gets harder
        self.xp_to_next = match self.level {
            1..=10 => self.level * 80 + 50,
            11..=30 => self.level * 100 + 100,
            31..=60 => self.level * 130 + 200,
            61..=100 => self.level * 170 + 400,
            101..=130 => self.level * 220 + 800,
            _ => self.level * 300 + 1500,
        };
        self.strength += 1;
        self.dexterity += 1;
        self.intelligence += 1;
        self.max_hp += 5;
        self.hp = self.max_hp;
        self.update_title();
    }

    pub fn can_prestige(&self) -> bool {
        self.level >= MAX_LEVEL
    }

    pub fn prestige(&mut self, subclass: Subclass) {
        self.prestige += 1;
        self.total_prestiges += 1;

        // Prestige bonus: +2 to each stat per prestige tier
        let (sub_str, sub_dex, sub_int) = subclass.stat_bonus();

        // Reset to level 1 but keep gold, kills, inventory, gear
        let (base_str, base_dex, base_int) = self.class.base_stats();
        let (race_str, race_dex, race_int) = self.race.stat_bonus();
        let prestige_bonus = self.prestige as i32 * 2;

        self.level = 1;
        self.xp = 0;
        self.xp_to_next = 100;
        self.strength = base_str + race_str + prestige_bonus + sub_str;
        self.dexterity = base_dex + race_dex + prestige_bonus + sub_dex;
        self.intelligence = base_int + race_int + prestige_bonus + sub_int;
        self.max_hp = 20 + (self.strength * 2) + (self.prestige as i32 * 10);
        self.hp = self.max_hp;
        self.subclass = Some(subclass);
        self.update_title();
    }

    fn update_title(&mut self) {
        let prestige_prefix = match self.prestige {
            0 => "",
            1 => "Prestigious ",
            2 => "Exalted ",
            3 => "Transcendent ",
            4 => "Mythical ",
            _ => "Godlike ",
        };

        let base_title = match self.level {
            1..=3 => "Terminal Novice",
            4..=8 => "Shell Apprentice",
            9..=14 => "Command Adept",
            15..=22 => "Pipe Weaver",
            23..=30 => "Script Sorcerer",
            31..=40 => "Kernel Knight",
            41..=50 => "Daemon Slayer",
            51..=65 => "Binary Sage",
            66..=80 => "System Architect",
            81..=95 => "Process Overlord",
            96..=110 => "Thread Titan",
            111..=125 => "Memory Monarch",
            126..=140 => "Stack Sovereign",
            141..=149 => "Root Demigod",
            150 => "Root Overlord",
            _ => "Root Overlord",
        };

        self.title = format!("{}{}", prestige_prefix, base_title);
    }

    pub fn heal(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    pub fn take_damage(&mut self, amount: i32) -> bool {
        self.hp -= amount;
        if self.hp <= 0 {
            self.die();
            return true;
        }
        false
    }

    fn die(&mut self) {
        self.deaths += 1;
        self.gold = self.gold.saturating_sub(self.gold / 4);
        self.hp = self.max_hp / 2;
    }

    pub fn equip(&mut self, item: Item) -> Option<Item> {
        match item.slot {
            ItemSlot::Weapon => {
                let old = self.weapon.take();
                self.weapon = Some(item);
                old
            }
            ItemSlot::Armor => {
                let old = self.armor.take();
                self.armor = Some(item);
                old
            }
            ItemSlot::Ring => {
                let old = self.ring.take();
                self.ring = Some(item);
                old
            }
            ItemSlot::Potion => {
                self.inventory.push(item);
                None
            }
        }
    }
}

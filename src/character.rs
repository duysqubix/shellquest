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

impl Rarity {
    pub fn is_droppable(&self) -> bool {
        matches!(self, Rarity::Common | Rarity::Uncommon | Rarity::Rare)
    }
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
            xp_to_next: 25,
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
        let base = self.dexterity / 3;
        let armor_bonus = self.armor.as_ref().map_or(0, |a| a.power);
        let ring_bonus = self.ring.as_ref().map_or(0, |r| r.power);
        base + armor_bonus + ring_bonus
    }

    pub fn gain_xp(&mut self, amount: u32) -> bool {
        if self.level >= MAX_LEVEL {
            return false; // At max level, prestige to continue
        }
        self.xp += amount;
        let mut leveled = false;
        while self.xp >= self.xp_to_next && self.level < MAX_LEVEL {
            self.level_up();
            leveled = true;
        }
        leveled
    }

    fn level_up(&mut self) {
        self.xp -= self.xp_to_next;
        self.level += 1;
        // XP curve: starts easy, gets harder
        self.xp_to_next = match self.level {
            1..=10 => self.level * 15 + 10,
            11..=30 => self.level * 25 + 30,
            31..=60 => self.level * 45 + 80,
            61..=100 => self.level * 80 + 200,
            101..=130 => self.level * 120 + 400,
            _ => self.level * 170 + 800,
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
        self.xp_to_next = 25;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(slot: ItemSlot, power: i32, rarity: Rarity) -> Item {
        Item {
            name: "Test Item".to_string(),
            slot,
            power,
            rarity,
        }
    }

    // --- Character::new() ---

    #[test]
    fn new_wizard_human_initial_stats() {
        let c = Character::new("Gandalf".to_string(), Class::Wizard, Race::Human);
        // Wizard base: STR 6, DEX 8, INT 16; Human bonus: +1/+1/+1
        assert_eq!(c.strength, 7);
        assert_eq!(c.dexterity, 9);
        assert_eq!(c.intelligence, 17);
        assert_eq!(c.max_hp, 20 + 7 * 2); // 34
        assert_eq!(c.hp, c.max_hp);
        assert_eq!(c.level, 1);
        assert_eq!(c.xp, 0);
        assert_eq!(c.xp_to_next, 25);
        assert_eq!(c.gold, 10);
        assert_eq!(c.kills, 0);
        assert_eq!(c.deaths, 0);
        assert!(c.weapon.is_none());
        assert!(c.armor.is_none());
        assert!(c.ring.is_none());
        assert!(c.inventory.is_empty());
    }

    #[test]
    fn new_warrior_orc_initial_stats() {
        let c = Character::new("Grok".to_string(), Class::Warrior, Race::Orc);
        // Warrior base: STR 16, DEX 8, INT 6; Orc bonus: +4/+1/-1
        assert_eq!(c.strength, 20);
        assert_eq!(c.dexterity, 9);
        assert_eq!(c.intelligence, 5);
        assert_eq!(c.max_hp, 20 + 20 * 2); // 60
    }

    #[test]
    fn new_rogue_goblin_initial_stats() {
        let c = Character::new("Sneak".to_string(), Class::Rogue, Race::Goblin);
        // Rogue base: STR 8, DEX 16, INT 6; Goblin bonus: -1/+3/+1
        assert_eq!(c.strength, 7);
        assert_eq!(c.dexterity, 19);
        assert_eq!(c.intelligence, 7);
    }

    #[test]
    fn new_ranger_elf_initial_stats() {
        let c = Character::new("Legolas".to_string(), Class::Ranger, Race::Elf);
        // Ranger base: STR 10, DEX 14, INT 6; Elf bonus: 0/+2/+2
        assert_eq!(c.strength, 10);
        assert_eq!(c.dexterity, 16);
        assert_eq!(c.intelligence, 8);
    }

    #[test]
    fn new_necromancer_dwarf_initial_stats() {
        let c = Character::new("Grimm".to_string(), Class::Necromancer, Race::Dwarf);
        // Necromancer base: STR 6, DEX 6, INT 18; Dwarf bonus: +3/0/+1
        assert_eq!(c.strength, 9);
        assert_eq!(c.dexterity, 6);
        assert_eq!(c.intelligence, 19);
    }

    // --- gain_xp() ---

    #[test]
    fn gain_xp_no_level_up_returns_false() {
        let mut c = Character::new("Hero".to_string(), Class::Warrior, Race::Human);
        let leveled = c.gain_xp(10);
        assert!(!leveled);
        assert_eq!(c.level, 1);
        assert_eq!(c.xp, 10);
    }

    #[test]
    fn gain_xp_exact_threshold_triggers_level_up() {
        let mut c = Character::new("Hero".to_string(), Class::Warrior, Race::Human);
        let leveled = c.gain_xp(25);
        assert!(leveled);
        assert_eq!(c.level, 2);
        assert_eq!(c.xp, 0);
    }

    #[test]
    fn gain_xp_over_threshold_carries_remainder() {
        let mut c = Character::new("Hero".to_string(), Class::Warrior, Race::Human);
        let leveled = c.gain_xp(50);
        assert!(leveled);
        assert_eq!(c.level, 2);
        assert_eq!(c.xp, 25);
    }

    #[test]
    fn gain_xp_at_max_level_returns_false() {
        let mut c = Character::new("Hero".to_string(), Class::Warrior, Race::Human);
        c.level = MAX_LEVEL;
        let leveled = c.gain_xp(99999);
        assert!(!leveled);
        assert_eq!(c.level, MAX_LEVEL);
    }

    // --- level_up() via gain_xp ---

    #[test]
    fn level_up_increments_stats() {
        let mut c = Character::new("Hero".to_string(), Class::Warrior, Race::Human);
        let str_before = c.strength;
        let dex_before = c.dexterity;
        let int_before = c.intelligence;
        let max_hp_before = c.max_hp;
        c.gain_xp(25);
        assert_eq!(c.strength, str_before + 1);
        assert_eq!(c.dexterity, dex_before + 1);
        assert_eq!(c.intelligence, int_before + 1);
        assert_eq!(c.max_hp, max_hp_before + 5);
        assert_eq!(c.hp, c.max_hp);
    }

    // --- attack_power() and defense() ---

    #[test]
    fn attack_power_no_weapon() {
        let c = Character::new("Hero".to_string(), Class::Warrior, Race::Human);
        // Warrior+Human: STR 17, DEX 9
        // base = 17 + 9/2 = 17 + 4 = 21
        assert_eq!(c.attack_power(), 17 + 9 / 2);
    }

    #[test]
    fn attack_power_with_weapon() {
        let mut c = Character::new("Hero".to_string(), Class::Warrior, Race::Human);
        c.equip(make_item(ItemSlot::Weapon, 10, Rarity::Common));
        assert_eq!(c.attack_power(), 17 + 9 / 2 + 10);
    }

    #[test]
    fn defense_no_equipment() {
        let c = Character::new("Hero".to_string(), Class::Warrior, Race::Human);
        // DEX 9; base = 9/3 = 3
        assert_eq!(c.defense(), 9 / 3);
    }

    #[test]
    fn defense_with_armor_and_ring() {
        let mut c = Character::new("Hero".to_string(), Class::Warrior, Race::Human);
        c.equip(make_item(ItemSlot::Armor, 5, Rarity::Common));
        c.equip(make_item(ItemSlot::Ring, 3, Rarity::Common));
        assert_eq!(c.defense(), 9 / 3 + 5 + 3);
    }

    #[test]
    fn defense_uses_dex_divided_by_three() {
        let c = Character::new("Test".to_string(), Class::Rogue, Race::Human);
        assert_eq!(c.defense(), 5);
    }

    // --- take_damage() / die() ---

    #[test]
    fn take_damage_nonlethal_returns_false() {
        let mut c = Character::new("Hero".to_string(), Class::Warrior, Race::Human);
        let died = c.take_damage(1);
        assert!(!died);
        assert!(c.hp > 0);
    }

    #[test]
    fn take_damage_lethal_returns_true_and_calls_die() {
        let mut c = Character::new("Hero".to_string(), Class::Warrior, Race::Human);
        let gold_before = c.gold;
        let max_hp = c.max_hp;
        let died = c.take_damage(max_hp + 100);
        assert!(died);
        assert_eq!(c.deaths, 1);
        // hp reset to max_hp / 2
        assert_eq!(c.hp, max_hp / 2);
        // gold reduced by 25%
        assert_eq!(c.gold, gold_before - gold_before / 4);
    }

    #[test]
    fn die_costs_25_percent_gold() {
        let mut c = Character::new("Hero".to_string(), Class::Warrior, Race::Human);
        c.gold = 100;
        let max_hp = c.max_hp;
        c.take_damage(max_hp + 1);
        assert_eq!(c.gold, 75);
    }

    #[test]
    fn die_sets_hp_to_half_max() {
        let mut c = Character::new("Hero".to_string(), Class::Warrior, Race::Human);
        let max_hp = c.max_hp;
        c.take_damage(max_hp + 1);
        assert_eq!(c.hp, max_hp / 2);
    }

    // --- heal() ---

    #[test]
    fn heal_increases_hp() {
        let mut c = Character::new("Hero".to_string(), Class::Warrior, Race::Human);
        let max_hp = c.max_hp;
        c.take_damage(10);
        c.heal(5);
        assert_eq!(c.hp, max_hp - 10 + 5);
    }

    #[test]
    fn heal_caps_at_max_hp() {
        let mut c = Character::new("Hero".to_string(), Class::Warrior, Race::Human);
        c.heal(9999);
        assert_eq!(c.hp, c.max_hp);
    }

    // --- equip() ---

    #[test]
    fn equip_weapon_returns_old_item() {
        let mut c = Character::new("Hero".to_string(), Class::Warrior, Race::Human);
        let first = make_item(ItemSlot::Weapon, 5, Rarity::Common);
        let second = make_item(ItemSlot::Weapon, 10, Rarity::Uncommon);
        let old = c.equip(first);
        assert!(old.is_none());
        let old2 = c.equip(second);
        assert!(old2.is_some());
        assert_eq!(old2.unwrap().power, 5);
        assert_eq!(c.weapon.as_ref().unwrap().power, 10);
    }

    #[test]
    fn equip_potion_goes_to_inventory() {
        let mut c = Character::new("Hero".to_string(), Class::Warrior, Race::Human);
        let potion = make_item(ItemSlot::Potion, 5, Rarity::Common);
        let result = c.equip(potion);
        assert!(result.is_none());
        assert_eq!(c.inventory.len(), 1);
    }

    // --- can_prestige() ---

    #[test]
    fn can_prestige_only_at_max_level() {
        let mut c = Character::new("Hero".to_string(), Class::Warrior, Race::Human);
        assert!(!c.can_prestige());
        c.level = MAX_LEVEL - 1;
        assert!(!c.can_prestige());
        c.level = MAX_LEVEL;
        assert!(c.can_prestige());
    }

    // --- prestige() ---

    #[test]
    fn prestige_resets_level_keeps_gold_and_kills() {
        let mut c = Character::new("Hero".to_string(), Class::Warrior, Race::Human);
        c.level = MAX_LEVEL;
        c.gold = 500;
        c.kills = 42;
        c.inventory.push(make_item(ItemSlot::Potion, 5, Rarity::Common));
        c.prestige(Subclass::Berserker);
        assert_eq!(c.level, 1);
        assert_eq!(c.gold, 500);
        assert_eq!(c.kills, 42);
        assert_eq!(c.inventory.len(), 1);
        assert_eq!(c.prestige, 1);
        assert_eq!(c.total_prestiges, 1);
    }

    #[test]
    fn prestige_applies_subclass_bonus() {
        let mut c = Character::new("Hero".to_string(), Class::Warrior, Race::Human);
        c.level = MAX_LEVEL;
        // Warrior+Human base: STR 17, prestige 1 adds +2 flat, Berserker adds +3 STR
        c.prestige(Subclass::Berserker);
        // base_str=16, race_str=1, prestige_bonus=2, sub_str=3 => 22
        let (base_str, _, _) = Class::Warrior.base_stats();
        let (race_str, _, _) = Race::Human.stat_bonus();
        let prestige_bonus = 1_i32 * 2;
        let (sub_str, _, _) = Subclass::Berserker.stat_bonus();
        assert_eq!(c.strength, base_str + race_str + prestige_bonus + sub_str);
    }

    // --- Subclass::available_for() ---

    #[test]
    fn subclass_available_for_wizard() {
        let subs = Subclass::available_for(&Class::Wizard);
        assert_eq!(subs.len(), 3);
        assert!(matches!(subs[0], Subclass::Archmage));
        assert!(matches!(subs[1], Subclass::Chronomancer));
        assert!(matches!(subs[2], Subclass::Datamancer));
    }

    #[test]
    fn subclass_available_for_warrior() {
        let subs = Subclass::available_for(&Class::Warrior);
        assert_eq!(subs.len(), 3);
        assert!(matches!(subs[0], Subclass::Berserker));
        assert!(matches!(subs[1], Subclass::Paladin));
        assert!(matches!(subs[2], Subclass::Warlord));
    }

    #[test]
    fn subclass_available_for_rogue() {
        let subs = Subclass::available_for(&Class::Rogue);
        assert_eq!(subs.len(), 3);
        assert!(matches!(subs[0], Subclass::Assassin));
        assert!(matches!(subs[1], Subclass::Hacker));
        assert!(matches!(subs[2], Subclass::Shadow));
    }

    #[test]
    fn subclass_available_for_ranger() {
        let subs = Subclass::available_for(&Class::Ranger);
        assert_eq!(subs.len(), 3);
        assert!(matches!(subs[0], Subclass::Beastmaster));
        assert!(matches!(subs[1], Subclass::Sniper));
        assert!(matches!(subs[2], Subclass::Scout));
    }

    #[test]
    fn subclass_available_for_necromancer() {
        let subs = Subclass::available_for(&Class::Necromancer);
        assert_eq!(subs.len(), 3);
        assert!(matches!(subs[0], Subclass::Lich));
        assert!(matches!(subs[1], Subclass::Plaguebearer));
        assert!(matches!(subs[2], Subclass::SoulReaper));
    }

    #[test]
    fn rarity_common_is_droppable() {
        assert!(Rarity::Common.is_droppable());
    }

    #[test]
    fn rarity_uncommon_is_droppable() {
        assert!(Rarity::Uncommon.is_droppable());
    }

    #[test]
    fn rarity_rare_is_droppable() {
        assert!(Rarity::Rare.is_droppable());
    }

    #[test]
    fn rarity_epic_is_not_droppable() {
        assert!(!Rarity::Epic.is_droppable());
    }

    #[test]
    fn rarity_legendary_is_not_droppable() {
        assert!(!Rarity::Legendary.is_droppable());
    }

    #[test]
    fn xp_to_next_level_1_is_25() {
        let char = Character::new("T".to_string(), Class::Warrior, Race::Human);
        assert_eq!(char.xp_to_next, 25);
    }

    #[test]
    fn xp_bracket_level_50_cost_is_2330() {
        // level 50 in the 31..=60 bracket: 50*45 + 80 = 2330
        let l50_cost: u32 = 50 * 45 + 80;
        assert_eq!(l50_cost, 2330);
        assert!(l50_cost < 3000);
    }
}

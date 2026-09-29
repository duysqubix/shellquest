#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Boss {
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub attack: i32,
    pub xp_reward: u32,
    pub gold_reward: u32,
    pub spawned_at: DateTime<Utc>,
    #[serde(default)]
    pub dex_mod: i32,
    #[serde(default)]
    pub dmg_dealt_total: i32,
    #[serde(default)]
    pub dmg_taken_total: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct BossInfo {
    pub name: String,
    pub hp: i32,
    pub attack: i32,
    pub xp_reward: u32,
    pub gold_reward: u32,
    pub dex_mod: i32,
}

pub const BOSS_SPAWN_RATE: f64 = 1.0 / 500.0;
pub const BOSS_MAX_PLAYER_DODGE_ADVANTAGE: i32 = 6;
/// No boss appears below this level: under it a duel was nearly unwinnable (0-25%
/// win rate at L10-15), and the roster's stats are a fair fight right at it.
pub const BOSS_MIN_LEVEL: u32 = 25;
/// Boss HP, attack and rewards grow with the player's level as
/// (level + OFFSET) / (BOSS_MIN_LEVEL + OFFSET), roughly how player HP and attack
/// grow, so a boss costs about the same share of a player's HP at L50 or L150 as
/// at L25 instead of becoming trivial.
pub const BOSS_SCALE_OFFSET: u32 = 5;
/// Share of gold `sq flee` costs (dying to a boss costs 15% plus the level's XP).
pub const BOSS_FLEE_GOLD_PERCENT: u32 = 10;

pub const BOSS_ROSTER: &[(&str, i32, i32, u32, u32, i32)] = &[
    ("The Kernel Panic", 500, 48, 1450, 560, 6),
    ("Lord of /dev/null", 440, 42, 1150, 450, 8),
    ("SIGKILL Supreme", 470, 54, 1325, 520, 9),
    ("The Infinite Loop", 540, 36, 1550, 500, 5),
    ("The Memory Corruption", 490, 46, 1400, 510, 7),
];

fn effective_boss_target_dodge_mod(player_dex_mod: i32, boss_attack_mod: i32) -> i32 {
    player_dex_mod.min(boss_attack_mod + BOSS_MAX_PLAYER_DODGE_ADVANTAGE)
}

fn boss_damage_after_defense(boss_atk: i32, player_defense: i32) -> i32 {
    (boss_atk - player_defense / 3).max(1)
}

pub fn boss_roster() -> Vec<BossInfo> {
    BOSS_ROSTER
        .iter()
        .map(
            |(name, hp, attack, xp_reward, gold_reward, dex_mod)| BossInfo {
                name: (*name).to_string(),
                hp: *hp,
                attack: *attack,
                xp_reward: *xp_reward,
                gold_reward: *gold_reward,
                dex_mod: *dex_mod,
            },
        )
        .collect()
}

/// How much stronger (and richer) than its roster entry a boss is at `level`.
pub fn level_scale(level: u32) -> f64 {
    (level.max(BOSS_MIN_LEVEL) + BOSS_SCALE_OFFSET) as f64
        / (BOSS_MIN_LEVEL + BOSS_SCALE_OFFSET) as f64
}

/// A random roster boss scaled to a player of `level`.
pub fn spawn_boss_for_level(level: u32) -> Boss {
    use rand::Rng;

    let mut rng = rand::thread_rng();
    let (name, hp, attack, xp_reward, gold_reward, dex_mod) =
        BOSS_ROSTER[rng.gen_range(0..BOSS_ROSTER.len())];
    let scale = level_scale(level);
    let hp = (hp as f64 * scale).round() as i32;

    Boss {
        name: name.to_string(),
        hp,
        max_hp: hp,
        attack: (attack as f64 * scale).round() as i32,
        xp_reward: (xp_reward as f64 * scale).round() as u32,
        gold_reward: (gold_reward as f64 * scale).round() as u32,
        spawned_at: Utc::now(),
        dex_mod,
        dmg_dealt_total: 0,
        dmg_taken_total: 0,
    }
}

/// A roster boss at its base stats (the ones a player at `BOSS_MIN_LEVEL` meets).
pub fn spawn_boss() -> Boss {
    spawn_boss_for_level(BOSS_MIN_LEVEL)
}

#[derive(Debug, PartialEq)]
pub enum Flight {
    /// Fled from the named boss, paying this much gold.
    Fled { boss: String, cost: u32 },
    /// The boss had already given up (stale, or the player is under the level
    /// gate) and would have left on the next tick: it leaves now, for free.
    AlreadyLeaving { boss: String },
}

/// `sq flee`: the active boss leaves and the player pays `BOSS_FLEE_GOLD_PERCENT` of
/// their gold. None without a boss.
pub fn flee(state: &mut crate::state::GameState) -> Option<Flight> {
    use crate::journal::{EventType, JournalEntry};

    let boss = state.active_boss.take()?;
    crate::telemetry::emit_encounter(
        "boss",
        &boss.name,
        false,
        boss.dmg_dealt_total,
        boss.dmg_taken_total,
        "flee",
        0,
        0,
    );
    if boss.is_stale() || state.character.level < BOSS_MIN_LEVEL {
        return Some(Flight::AlreadyLeaving { boss: boss.name });
    }
    let cost = state.character.gold * BOSS_FLEE_GOLD_PERCENT / 100;
    state.character.gold -= cost;
    state.add_journal(JournalEntry::new(
        EventType::Combat,
        format!("Fled from {}. -{} gold.", boss.name, cost),
    ));
    Some(Flight::Fled {
        boss: boss.name,
        cost,
    })
}

impl Boss {
    pub fn is_stale(&self) -> bool {
        let age = Utc::now() - self.spawned_at;
        age.num_hours() >= 24
    }
}

pub fn maybe_spawn(state: &mut crate::state::GameState) {
    use rand::Rng;

    if state.active_boss.is_some() || state.character.level < BOSS_MIN_LEVEL {
        return;
    }

    let mut rng = rand::thread_rng();
    if rng.gen_ratio(1, 500) {
        let boss = spawn_boss_for_level(state.character.level);
        crate::display::print_boss_spawn(&boss);
        state.active_boss = Some(boss);
    }
}

pub fn tick_boss(state: &mut crate::state::GameState) {
    use crate::journal::{EventType, JournalEntry};
    use rand::Rng;

    let boss_is_stale = state
        .active_boss
        .as_ref()
        .is_some_and(|boss| boss.is_stale());
    if boss_is_stale {
        let (name, dmg_dealt_total, dmg_taken_total) = {
            let boss = state.active_boss.as_ref().unwrap();
            (
                boss.name.clone(),
                boss.dmg_dealt_total,
                boss.dmg_taken_total,
            )
        };
        state.active_boss = None;
        crate::telemetry::emit_encounter(
            "boss",
            &name,
            false,
            dmg_dealt_total,
            dmg_taken_total,
            "flee",
            0,
            0,
        );
        crate::display::print_boss_flee(&name, "grows bored waiting and retreats. It will return");
        return;
    }

    if state.active_boss.is_none() {
        return;
    }

    // A boss from before the level gate (an older save), or one met after prestige
    // reset the level: it doesn't fight anyone under the gate.
    if state.character.level < BOSS_MIN_LEVEL {
        let boss = state.active_boss.take().unwrap();
        crate::telemetry::emit_encounter(
            "boss",
            &boss.name,
            false,
            boss.dmg_dealt_total,
            boss.dmg_taken_total,
            "flee",
            0,
            0,
        );
        crate::display::print_boss_flee(&boss.name, "loses interest in so small a foe and departs");
        return;
    }

    let mut rng = rand::thread_rng();
    let player_power = state.character.attack_power();
    let player_defense = state.character.defense();
    let player_int = state.character.intelligence;
    let player_str = state.character.strength;
    let player_hp = state.character.hp;
    let player_max_hp = state.character.max_hp;
    let player_class = state.character.class.clone();

    let boss_at_full_hp = state.active_boss.as_ref().is_some_and(|b| b.hp == b.max_hp);

    let hit_roll: i32 = rng.gen_range(1..=20);
    let crit_threshold = (20 - player_int / 8).max(13);
    let mut signature_label: Option<&'static str> = None;
    let player_dmg: Option<(i32, bool)> = {
        let boss = state.active_boss.as_mut().unwrap();
        let hit_landed = match player_class {
            crate::character::Class::Rogue => hit_roll + player_power > 10 || hit_roll == 1,
            _ => hit_roll + player_power > 10,
        };
        if hit_landed {
            if matches!(player_class, crate::character::Class::Rogue) && hit_roll == 1 {
                signature_label = Some("shadow strike");
            }
            let mut raw_dmg = rng.gen_range((player_power / 2).max(1)..=player_power.max(1));
            let (sig_bonus, sig_label) = crate::character::signature_bonus(
                &player_class,
                player_int,
                player_str,
                player_hp,
                player_max_hp,
                boss_at_full_hp,
            );
            if sig_bonus > 0 {
                raw_dmg += sig_bonus;
                if signature_label.is_none() {
                    signature_label = sig_label;
                }
            }
            let is_crit = hit_roll >= crit_threshold;
            let dmg = if is_crit { raw_dmg * 2 } else { raw_dmg };
            boss.hp -= dmg;
            boss.dmg_dealt_total += dmg;
            Some((dmg, is_crit))
        } else {
            None
        }
    };

    let boss_hp_after = state.active_boss.as_ref().unwrap().hp;
    let boss_max_hp = state.active_boss.as_ref().unwrap().max_hp;
    let boss_atk = state.active_boss.as_ref().unwrap().attack;
    let boss_dex_mod = state.active_boss.as_ref().unwrap().dex_mod;
    let boss_name = state.active_boss.as_ref().unwrap().name.clone();
    let boss_xp = crate::character::scale_xp_gain(state.active_boss.as_ref().unwrap().xp_reward);
    let boss_gold = state.active_boss.as_ref().unwrap().gold_reward;
    let boss_dmg_dealt_total = state.active_boss.as_ref().unwrap().dmg_dealt_total;
    let boss_dmg_taken_total = state.active_boss.as_ref().unwrap().dmg_taken_total;

    if boss_hp_after <= 0 {
        crate::display::print_boss_tick(state.active_boss.as_ref().unwrap(), player_dmg, None);
        print_signature_line(signature_label);
        crate::display::print_boss_victory(state.active_boss.as_ref().unwrap(), boss_xp, boss_gold);

        let loot = crate::loot::roll_boss_loot();
        let loot_msg = format!(
            "Boss loot: {} (+{} {}) [{}]",
            loot.name, loot.power, loot.slot, loot.rarity
        );
        crate::display::print_loot(&loot_msg, &loot.rarity);

        state.add_journal(JournalEntry::new(
            EventType::Combat,
            format!(
                "Defeated {}! +{} XP +{} gold",
                boss_name, boss_xp, boss_gold
            ),
        ));

        crate::telemetry::emit_encounter(
            "boss",
            &boss_name,
            false,
            boss_dmg_dealt_total,
            boss_dmg_taken_total,
            "win",
            boss_xp,
            boss_gold,
        );

        state.active_boss = None;

        let leveled = state.character.gain_xp(boss_xp);
        state.character.gold += boss_gold;
        crate::events::add_to_inventory_pub(state, loot);

        let drained = state.character.signature_on_kill();
        if drained > 0 {
            crate::display::print_soul_drain(drained, state.character.hp, state.character.max_hp);
            state.add_journal(JournalEntry::new(
                EventType::Combat,
                format!("Soul drained from {}: +{} HP.", boss_name, drained),
            ));
        }

        if leveled {
            crate::events::emit_level_up(state);
        }
        return;
    }

    let dodge_roll: i32 = rng.gen_range(1..=20);
    let boss_attack_mod = boss_dex_mod + state.character.total_prestiges as i32;
    let player_dodge_mod =
        effective_boss_target_dodge_mod(state.character.dex_mod(), boss_attack_mod);
    let boss_dmg = if crate::character::attack_lands(dodge_roll, boss_attack_mod, player_dodge_mod)
    {
        let dmg = boss_damage_after_defense(boss_atk, player_defense);
        let gold_before = state.character.gold;
        let died = state.character.take_damage(dmg);
        if let Some(boss) = state.active_boss.as_mut() {
            boss.dmg_taken_total += dmg;
        }
        let boss_dmg_dealt_total = state.active_boss.as_ref().unwrap().dmg_dealt_total;
        let boss_dmg_taken_total = state.active_boss.as_ref().unwrap().dmg_taken_total;
        if died {
            if state.permadeath {
                // A boss never ends a permadeath run: it leaves the player at 1 HP.
                state.character.hp = 1;
                crate::display::print_boss_tick(
                    state.active_boss.as_ref().unwrap(),
                    player_dmg,
                    Some(dmg),
                );
                print_signature_line(signature_label);
                crate::display::print_boss_flee(
                    &boss_name,
                    "stays its final blow, leaving you at 1 HP, and vanishes into the void",
                );
                state.add_journal(JournalEntry::new(
                    EventType::Combat,
                    format!("{} spared you at 1 HP and left.", boss_name),
                ));
                crate::telemetry::emit_encounter(
                    "boss",
                    &boss_name,
                    false,
                    boss_dmg_dealt_total,
                    boss_dmg_taken_total,
                    "flee",
                    0,
                    0,
                );
                state.active_boss = None;
                return;
            } else {
                state.character.die();
                let gold_loss = gold_before * 15 / 100;
                crate::display::print_boss_tick(
                    state.active_boss.as_ref().unwrap(),
                    player_dmg,
                    Some(dmg),
                );
                print_signature_line(signature_label);
                crate::display::print_boss_flee(
                    &boss_name,
                    "laughs as you fall... and vanishes into the void",
                );
                state.add_journal(crate::journal::JournalEntry::new(
                    crate::journal::EventType::Death,
                    format!(
                        "{} fled after you fell. XP reset, -{} gold.",
                        boss_name, gold_loss
                    ),
                ));
                crate::telemetry::emit_encounter(
                    "boss",
                    &boss_name,
                    false,
                    boss_dmg_dealt_total,
                    boss_dmg_taken_total,
                    "loss",
                    0,
                    0,
                );
                state.active_boss = None;
                return;
            }
        }
        Some(dmg)
    } else {
        None
    };

    crate::display::print_boss_tick(state.active_boss.as_ref().unwrap(), player_dmg, boss_dmg);
    print_signature_line(signature_label);
    state.add_journal(JournalEntry::new(
        EventType::Combat,
        format!(
            "[BOSS] {} — HP: {}/{}",
            boss_name,
            boss_hp_after.max(0),
            boss_max_hp
        ),
    ));
}

fn print_signature_line(label: Option<&'static str>) {
    use colored::Colorize;
    if let Some(label) = label {
        eprintln!("   {} {}", "✨".cyan(), label.cyan().italic());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boss_roster_has_five_entries() {
        assert_eq!(BOSS_ROSTER.len(), 5);
    }

    #[test]
    fn boss_roster_catalog_has_five_positive_entries() {
        let bosses = boss_roster();

        assert_eq!(bosses.len(), 5);
        for boss in bosses {
            assert!(!boss.name.is_empty());
            assert!(boss.hp > 0);
            assert!(boss.attack > 0);
            assert!(boss.xp_reward > 0);
            assert!(boss.gold_reward > 0);
        }
    }

    #[test]
    fn all_bosses_have_positive_hp_and_attack() {
        for (_, hp, atk, _, _, _) in BOSS_ROSTER.iter() {
            assert!(*hp > 0);
            assert!(*atk > 0);
        }
    }

    #[test]
    fn boss_roster_pins_endgame_pressure_band() {
        assert_eq!(BOSS_SPAWN_RATE, 1.0 / 500.0);
        assert_eq!(BOSS_MAX_PLAYER_DODGE_ADVANTAGE, 6);

        let min_hp = BOSS_ROSTER.iter().map(|(_, hp, _, _, _, _)| *hp).min();
        let max_hp = BOSS_ROSTER.iter().map(|(_, hp, _, _, _, _)| *hp).max();
        let min_attack = BOSS_ROSTER.iter().map(|(_, _, atk, _, _, _)| *atk).min();
        let max_attack = BOSS_ROSTER.iter().map(|(_, _, atk, _, _, _)| *atk).max();

        assert_eq!(min_hp, Some(440));
        assert_eq!(max_hp, Some(540));
        assert_eq!(min_attack, Some(36));
        assert_eq!(max_attack, Some(54));
    }

    #[test]
    fn boss_hit_and_damage_pressure_are_soft_capped_not_nullified() {
        assert_eq!(effective_boss_target_dodge_mod(80, 20), 26);
        assert_eq!(effective_boss_target_dodge_mod(24, 20), 24);
        assert_eq!(boss_damage_after_defense(48, 60), 28);
        assert_eq!(boss_damage_after_defense(12, 99), 1);
    }

    fn state_at_level(level: u32) -> crate::state::GameState {
        use crate::character::{Character, Class, Race};
        let mut s = crate::state::GameState::new(Character::new(
            "T".to_string(),
            Class::Warrior,
            Race::Human,
        ));
        s.character.level = level;
        s
    }

    fn test_boss(hp: i32, attack: i32) -> Boss {
        Boss {
            name: "Test Colossus".to_string(),
            hp,
            max_hp: hp,
            attack,
            xp_reward: 900,
            gold_reward: 350,
            spawned_at: Utc::now(),
            dex_mod: 30,
            dmg_dealt_total: 0,
            dmg_taken_total: 0,
        }
    }

    #[test]
    fn level_gate_and_scale_are_pinned() {
        assert_eq!(BOSS_MIN_LEVEL, 25);
        assert_eq!(BOSS_SCALE_OFFSET, 5);
        assert_eq!(BOSS_FLEE_GOLD_PERCENT, 10);
        assert_eq!(level_scale(1), 1.0);
        assert_eq!(level_scale(BOSS_MIN_LEVEL), 1.0);
        assert_eq!(level_scale(55), 2.0);
        assert!((level_scale(150) - 155.0 / 30.0).abs() < 1e-9);
    }

    #[test]
    fn scaled_bosses_stay_within_the_scaled_roster_band() {
        let scale = level_scale(100);
        for _ in 0..50 {
            let boss = spawn_boss_for_level(100);
            assert_eq!(boss.hp, boss.max_hp);
            assert!((440.0 * scale).round() as i32 <= boss.hp);
            assert!(boss.hp <= (540.0 * scale).round() as i32);
            assert!((36.0 * scale).round() as i32 <= boss.attack);
            assert!(boss.attack <= (54.0 * scale).round() as i32);
            assert!(boss.xp_reward >= (1150.0 * scale).round() as u32);
        }
    }

    #[test]
    fn no_boss_spawns_below_the_level_gate() {
        let mut state = state_at_level(BOSS_MIN_LEVEL - 1);
        for _ in 0..20_000 {
            maybe_spawn(&mut state);
        }
        assert!(state.active_boss.is_none());
    }

    #[test]
    fn a_boss_under_the_level_gate_leaves_without_fighting() {
        let mut state = state_at_level(3);
        state.active_boss = Some(spawn_boss());
        let hp = state.character.hp;
        tick_boss(&mut state);
        assert!(state.active_boss.is_none());
        assert_eq!(state.character.hp, hp);
        assert_eq!(state.character.deaths, 0);
    }

    #[test]
    fn a_boss_never_kills_a_permadeath_character() {
        let mut state = state_at_level(BOSS_MIN_LEVEL);
        state.permadeath = true;
        state.character.hp = 5;
        state.active_boss = Some(test_boss(1_000_000, 500));
        for _ in 0..200 {
            if state.active_boss.is_none() {
                break;
            }
            tick_boss(&mut state);
        }
        assert!(state.active_boss.is_none(), "the boss left");
        assert_eq!(state.character.hp, 1);
        assert_eq!(state.character.deaths, 0);
    }

    #[test]
    fn fleeing_costs_a_tenth_of_the_gold_and_ends_the_fight() {
        let mut state = state_at_level(BOSS_MIN_LEVEL);
        assert_eq!(flee(&mut state), None, "nothing to flee from");
        state.character.gold = 1_005;
        state.active_boss = Some(test_boss(500, 40));
        assert_eq!(
            flee(&mut state),
            Some(Flight::Fled {
                boss: "Test Colossus".to_string(),
                cost: 100
            })
        );
        assert_eq!(state.character.gold, 905);
        assert!(state.active_boss.is_none());
    }

    #[test]
    fn fleeing_a_boss_that_would_leave_anyway_is_free() {
        let mut stale = test_boss(500, 40);
        stale.spawned_at = Utc::now() - chrono::Duration::hours(25);
        for (level, boss) in [(BOSS_MIN_LEVEL, stale), (1, test_boss(500, 40))] {
            let mut state = state_at_level(level);
            state.character.gold = 1_000;
            state.active_boss = Some(boss);
            assert_eq!(
                flee(&mut state),
                Some(Flight::AlreadyLeaving {
                    boss: "Test Colossus".to_string()
                })
            );
            assert_eq!(state.character.gold, 1_000);
            assert!(state.active_boss.is_none());
        }
    }

    #[test]
    fn spawn_boss_returns_boss_with_full_hp() {
        let boss = spawn_boss();
        assert_eq!(boss.hp, boss.max_hp);
        assert!(boss.hp > 0);
    }

    #[test]
    fn spawn_boss_xp_reward_is_substantial() {
        let boss = spawn_boss();
        assert!(boss.xp_reward >= 500);
    }

    #[test]
    fn is_stale_returns_false_for_fresh_boss() {
        let boss = spawn_boss();
        assert!(!boss.is_stale());
    }

    #[test]
    fn maybe_spawn_does_not_spawn_if_boss_active() {
        use crate::character::{Character, Class, Race};
        use crate::state::GameState;

        let _: fn(&mut GameState) = maybe_spawn;
        let mut state =
            GameState::new(Character::new("T".to_string(), Class::Warrior, Race::Human));
        let existing = spawn_boss();
        state.active_boss = Some(existing);
        let boss_name_before = state.active_boss.as_ref().unwrap().name.clone();

        maybe_spawn(&mut state);

        assert_eq!(state.active_boss.as_ref().unwrap().name, boss_name_before);
    }

    #[test]
    fn stale_boss_is_detected_correctly() {
        let _: fn(&mut crate::state::GameState) = tick_boss;
        let mut boss = spawn_boss();
        boss.spawned_at = Utc::now() - chrono::Duration::hours(25);
        assert!(boss.is_stale());
    }

    #[test]
    fn boss_roster_dex_mods_are_varied_and_in_range() {
        let mods: Vec<i32> = BOSS_ROSTER.iter().map(|b| b.5).collect();
        assert_eq!(mods.len(), 5);
        for m in &mods {
            assert!((5..=9).contains(m), "boss dex_mod {} out of 5..=9", m);
        }
        let distinct = mods.iter().collect::<std::collections::HashSet<_>>().len();
        assert!(distinct >= 2, "boss dex_mods should vary by power");
    }

    #[test]
    fn high_armor_low_dex_player_can_be_hit_by_boss() {
        use crate::character::{Character, Class, Item, ItemSlot, Race, Rarity};
        use crate::state::GameState;

        let mut state =
            GameState::new(Character::new("T".to_string(), Class::Warrior, Race::Human));
        state.character.level = BOSS_MIN_LEVEL;
        state.character.dexterity = 8;
        state.character.max_hp = 100_000;
        state.character.hp = 100_000;
        state.character.equip(Item {
            name: "Bastion Plate".to_string(),
            slot: ItemSlot::Armor,
            power: 100,
            rarity: Rarity::Legendary,
            enchant_level: 0,
        });
        state.active_boss = Some(Boss {
            name: "Test Colossus".to_string(),
            hp: 100_000,
            max_hp: 100_000,
            attack: 20,
            xp_reward: 900,
            gold_reward: 350,
            spawned_at: Utc::now(),
            dex_mod: 8,
            dmg_dealt_total: 0,
            dmg_taken_total: 0,
        });

        let hp0 = state.character.hp;
        for _ in 0..100 {
            if state.active_boss.is_none() {
                break;
            }
            tick_boss(&mut state);
        }
        assert!(
            state.character.hp < hp0,
            "a boss must be able to land hits on a high-armor low-dex player (was un-hittable before)"
        );
    }
}

/// Monte Carlo duels through the real `tick_boss`, by level and gear, as evidence
/// for BOSS_MIN_LEVEL and BOSS_SCALE_OFFSET (the Docker balance sim can't measure
/// boss fights until shellqeuest-dpl closes). Run:
/// `cargo test --release duel_table -- --ignored --nocapture 2>/dev/null | grep DUEL`
#[cfg(test)]
mod duel_harness {
    use super::*;
    use crate::character::{Character, Class, Item, ItemSlot, Race, Rarity};
    use crate::state::GameState;

    fn gear(slot: ItemSlot, power: i32) -> Item {
        Item {
            name: "G".into(),
            slot,
            power,
            rarity: Rarity::Rare,
            enchant_level: 0,
        }
    }

    fn player(class: Class, level: u32, gear_power: i32) -> GameState {
        let mut s = GameState::new(Character::new("T".into(), class, Race::Human));
        while s.character.level < level {
            let need = s.character.xp_to_next - s.character.xp;
            s.character.gain_xp(need);
        }
        if gear_power > 0 {
            for slot in [ItemSlot::Weapon, ItemSlot::Armor, ItemSlot::Ring] {
                s.character.equip(gear(slot, gear_power));
            }
        }
        s.character.hp = s.character.max_hp;
        s
    }

    #[test]
    #[ignore]
    fn duel_table() {
        let classes = [
            Class::Warrior,
            Class::Wizard,
            Class::Rogue,
            Class::Ranger,
            Class::Necromancer,
        ];
        for gear_power in [0, 7, 15] {
            // Levels at or above the gate: below it the boss just leaves.
            for level in [25, 35, 50, 75, 100, 150] {
                let (mut wins, mut n, mut hp_lost, mut ticks) = (0, 0, 0.0, 0);
                for class in classes.iter() {
                    for _ in 0..300 {
                        let mut s = player(class.clone(), level, gear_power);
                        s.active_boss = Some(spawn_boss_for_level(level));
                        let deaths = s.character.deaths;
                        let max = s.character.max_hp as f64;
                        let mut t = 0;
                        while s.active_boss.is_some() && t < 500 {
                            tick_boss(&mut s);
                            t += 1;
                        }
                        n += 1;
                        ticks += t;
                        if s.character.deaths == deaths {
                            wins += 1;
                            hp_lost += (max - s.character.hp as f64) / max;
                        }
                    }
                }
                println!(
                    "DUEL gear={:>2} L{:>3}: win {:>3}%  hp lost when won {:>3}%  ticks {:.1}",
                    gear_power,
                    level,
                    wins * 100 / n,
                    (hp_lost * 100.0 / wins.max(1) as f64) as i32,
                    ticks as f64 / n as f64
                );
            }
        }
    }
}

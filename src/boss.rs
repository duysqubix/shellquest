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
}

pub const BOSS_ROSTER: &[(&str, i32, i32, u32, u32)] = &[
    ("The Kernel Panic", 100, 22, 900, 350),
    ("Lord of /dev/null", 85, 18, 700, 280),
    ("SIGKILL Supreme", 90, 25, 800, 320),
    ("The Infinite Loop", 110, 15, 950, 300),
    ("The Memory Corruption", 95, 20, 850, 310),
];

pub fn spawn_boss() -> Boss {
    use rand::Rng;

    let mut rng = rand::thread_rng();
    let (name, hp, attack, xp_reward, gold_reward) =
        BOSS_ROSTER[rng.gen_range(0..BOSS_ROSTER.len())];

    Boss {
        name: name.to_string(),
        hp,
        max_hp: hp,
        attack,
        xp_reward,
        gold_reward,
        spawned_at: Utc::now(),
    }
}

impl Boss {
    pub fn is_stale(&self) -> bool {
        let age = Utc::now() - self.spawned_at;
        age.num_hours() >= 24
    }
}

pub fn maybe_spawn(state: &mut crate::state::GameState) {
    use rand::Rng;

    if state.active_boss.is_some() {
        return;
    }

    let mut rng = rand::thread_rng();
    if rng.gen_ratio(1, 500) {
        let boss = spawn_boss();
        crate::display::print_boss_spawn(&boss);
        state.active_boss = Some(boss);
    }
}

pub fn tick_boss(state: &mut crate::state::GameState) {
    use crate::journal::{EventType, JournalEntry};
    use rand::Rng;

    let boss_is_stale = state.active_boss.as_ref().is_some_and(|boss| boss.is_stale());
    if boss_is_stale {
        let name = state.active_boss.take().unwrap().name;
        crate::display::print_boss_flee(&name, "grows bored waiting and retreats. It will return");
        return;
    }

    if state.active_boss.is_none() {
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
    let crit_threshold = (20 - player_int / 4).max(15);
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
            Some((dmg, is_crit))
        } else {
            None
        }
    };

    let boss_hp_after = state.active_boss.as_ref().unwrap().hp;
    let boss_max_hp = state.active_boss.as_ref().unwrap().max_hp;
    let boss_atk = state.active_boss.as_ref().unwrap().attack;
    let boss_name = state.active_boss.as_ref().unwrap().name.clone();
    let boss_xp = state.active_boss.as_ref().unwrap().xp_reward;
    let boss_gold = state.active_boss.as_ref().unwrap().gold_reward;

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
            format!("Defeated {}! +{} XP +{} gold", boss_name, boss_xp, boss_gold),
        ));

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
    let boss_dmg = if dodge_roll > 10 + player_defense {
        let dmg = (boss_atk - player_defense).max(1);
        let gold_before = state.character.gold;
        let died = state.character.take_damage(dmg);
        if died {
            if state.permadeath {
                crate::display::print_boss_tick(state.active_boss.as_ref().unwrap(), player_dmg, Some(dmg));
                print_signature_line(signature_label);
                crate::display::print_permadeath_eulogy(&state.character, &boss_name);
                let path = crate::state::save_path();
                let _ = std::fs::remove_file(&path);
                std::process::exit(0);
            } else {
                state.character.die();
                let gold_loss = gold_before * 15 / 100;
                crate::display::print_boss_tick(state.active_boss.as_ref().unwrap(), player_dmg, Some(dmg));
                print_signature_line(signature_label);
                crate::display::print_boss_flee(
                    &boss_name,
                    "laughs as you fall... and vanishes into the void",
                );
                state.add_journal(crate::journal::JournalEntry::new(
                    crate::journal::EventType::Death,
                    format!("{} fled after you fell. XP reset, -{} gold.", boss_name, gold_loss),
                ));
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
        format!("[BOSS] {} — HP: {}/{}", boss_name, boss_hp_after.max(0), boss_max_hp),
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
    fn all_bosses_have_positive_hp_and_attack() {
        for (_, hp, atk, _, _) in BOSS_ROSTER.iter() {
            assert!(*hp > 0);
            assert!(*atk > 0);
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
}

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
}

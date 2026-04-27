// Types are defined now and wired into the arena loop in later tasks.
#![allow(dead_code)]

use crate::character::{Character, Item};
use colored::*;
use rand::Rng;
use std::io::{self, Write};

/// Identity and static metadata for an arena tier.
///
/// Tiers are defined as `&'static` tables in `src/arena.rs` (Task 3) and are
/// never mutated at runtime. Unlock rules are checked against the player's
/// **current** character state at entry time.
#[derive(Debug, Clone, Copy)]
pub struct ArenaTier {
    pub index: u32,
    pub name: &'static str,
    pub max_rounds: u32,
    /// Minimum character level required to enter this tier.
    pub min_level: u32,
    /// Minimum total prestiges required to enter this tier.
    pub min_prestige: u32,
    /// When `true`, tier unlocks if level **OR** prestige requirement is met.
    /// When `false`, both requirements must be satisfied (AND).
    pub or_unlock: bool,
    /// Cumulative reward bands: `(round, gold_percent, xp_percent)`.
    /// Percents are expressed as whole numbers (e.g. `110` means 110 %).
    pub reward_bands: &'static [(u32, u32, u32)],
    /// Chest milestones: `(round, loot_danger)` to pass to `roll_loot_scaled`.
    pub chest_milestones: &'static [(u32, u32)],
}

pub const TIER_PIT: ArenaTier = ArenaTier {
    index: 0,
    name: "The Pit",
    max_rounds: 5,
    min_level: 0,
    min_prestige: 0,
    or_unlock: false,
    reward_bands: &[
        (1, 10, 5),
        (2, 25, 12),
        (3, 45, 24),
        (4, 70, 40),
        (5, 110, 60),
    ],
    chest_milestones: &[(5, 2)],
};

pub const TIER_GAUNTLET: ArenaTier = ArenaTier {
    index: 1,
    name: "The Gauntlet",
    max_rounds: 10,
    min_level: 25,
    min_prestige: 1,
    or_unlock: true,
    reward_bands: &[
        (5, 35, 22),
        (10, 145, 90),
    ],
    chest_milestones: &[(5, 2), (10, 4)],
};

pub const TIER_COLOSSEUM: ArenaTier = ArenaTier {
    index: 2,
    name: "The Colosseum",
    max_rounds: 15,
    min_level: 60,
    min_prestige: 1,
    or_unlock: true,
    reward_bands: &[
        (5, 30, 20),
        (10, 90, 55),
        (15, 185, 120),
    ],
    chest_milestones: &[(5, 4), (10, 4), (15, 6)],
};

pub const TIER_ABYSSAL: ArenaTier = ArenaTier {
    index: 3,
    name: "The Abyssal Arena",
    max_rounds: 25,
    min_level: 100,
    min_prestige: 2,
    or_unlock: true,
    reward_bands: &[
        (5, 20, 12),
        (10, 55, 30),
        (15, 100, 60),
        (20, 165, 105),
        (25, 240, 160),
    ],
    chest_milestones: &[(10, 4), (20, 6), (25, 6)],
};

pub const TIER_GODSLAYER: ArenaTier = ArenaTier {
    index: 4,
    name: "Godslayer's Court",
    max_rounds: 50,
    min_level: 150,
    min_prestige: 3,
    or_unlock: false,
    reward_bands: &[
        (10, 25, 15),
        (20, 70, 40),
        (30, 130, 85),
        (40, 210, 150),
        (50, 320, 220),
    ],
    chest_milestones: &[(10, 4), (20, 6), (40, 8), (50, 9)],
};

pub const ARENA_TIERS: &[ArenaTier] = &[
    TIER_PIT,
    TIER_GAUNTLET,
    TIER_COLOSSEUM,
    TIER_ABYSSAL,
    TIER_GODSLAYER,
];

impl ArenaTier {
    /// Returns `true` when the character satisfies the tier's unlock rules.
    ///
    /// The logic is controlled by `or_unlock`:
    /// - `true`  → level ≥ min_level **OR** total_prestiges ≥ min_prestige
    /// - `false` → level ≥ min_level **AND** total_prestiges ≥ min_prestige
    pub fn is_unlocked(&self, character: &Character) -> bool {
        let level_ok = character.level >= self.min_level;
        let prestige_ok = character.total_prestiges >= self.min_prestige;
        if self.or_unlock {
            level_ok || prestige_ok
        } else {
            level_ok && prestige_ok
        }
    }

    /// Entry fee for this tier, computed from the entry snapshot.
    ///
    /// The formula is specific to each tier and always uses snapshot values
    /// so that the fee never changes mid-run.
    pub fn compute_fee(&self, entry: &ArenaEntrySnapshot) -> u32 {
        let level = entry.level;
        let prestige = entry.prestige;
        let gold = entry.gold;
        match self.index {
            0 => {
                let a = 40;
                let b = level * 12;
                let c = gold / 10;
                a.max(b).max(c)
            }
            1 => {
                let a = 100;
                let b = level * 18 + prestige * 50;
                let c = gold / 8;
                a.max(b).max(c)
            }
            2 => {
                let a = 300;
                let b = level * 28 + prestige * 150;
                let c = gold / 6;
                a.max(b).max(c)
            }
            3 => {
                let a = 800;
                let b = level * 40 + prestige * 250;
                let c = gold / 5;
                a.max(b).max(c)
            }
            4 => {
                let a = 2500;
                let b = level * 60 + prestige * 400;
                let c = gold / 4;
                a.max(b).max(c)
            }
            _ => 0,
        }
    }

    /// Cumulative reward percentages at a given round.
    ///
    /// Returns `(gold_percent, xp_percent)` where each percent is a whole
    /// number (e.g. `110` means 110 %).  Linear interpolation is used
    /// between milestone rounds; values at exact milestones are exact.
    /// Rounds before the first milestone are interpolated from `(0,0,0)`.
    pub fn reward_percentages_at_round(&self, round: u32) -> (u32, u32) {
        if round == 0 {
            return (0, 0);
        }
        let bands = self.reward_bands;
        if bands.is_empty() {
            return (0, 0);
        }
        // Before the first milestone → interpolate from (0,0,0).
        if round < bands[0].0 {
            let (r2, g2, x2) = bands[0];
            let span = r2; // r1 is implicitly 0
            let step = round;
            let gold_pct = (g2 * step) / span;
            let xp_pct = (x2 * step) / span;
            return (gold_pct, xp_pct);
        }
        // Exact match or interpolate between two milestones.
        for i in 0..bands.len() {
            if round == bands[i].0 {
                return (bands[i].1, bands[i].2);
            }
            if i + 1 < bands.len() && round < bands[i + 1].0 {
                let (r1, g1, x1) = bands[i];
                let (r2, g2, x2) = bands[i + 1];
                let span = r2 - r1;
                let step = round - r1;
                let gold_pct = g1 + (g2 - g1) * step / span;
                let xp_pct = x1 + (x2 - x1) * step / span;
                return (gold_pct, xp_pct);
            }
        }
        // At or past the final milestone → clamp to last band.
        let last = bands.last().unwrap();
        (last.1, last.2)
    }

    /// Compute absolute gold and XP rewards for a given number of cleared
    /// rounds, using the entry fee and `xp_to_next` captured at entry.
    pub fn compute_rewards(
        &self,
        entry_fee: u32,
        xp_to_next: u32,
        rounds_cleared: u32,
    ) -> (u32, u32) {
        let (gold_pct, xp_pct) = self.reward_percentages_at_round(rounds_cleared);
        let gold_reward = (entry_fee as u64 * gold_pct as u64 / 100) as u32;
        let xp_reward = (xp_to_next as u64 * xp_pct as u64 / 100) as u32;
        (gold_reward, xp_reward)
    }

    /// Return all chest milestones earned up to and including `round`.
    pub fn collect_chests(&self, round: u32) -> Vec<PendingChest> {
        self.chest_milestones
            .iter()
            .filter(|(r, _)| *r <= round)
            .map(|(r, danger)| PendingChest {
                round: *r,
                danger: *danger,
            })
            .collect()
    }
}

/// Snapshot of character stats captured exactly once at arena entry.
///
/// All reward math, fee calculations, and combat scaling must use these
/// values and must **never** recompute from the live `Character` mid-run.
/// This prevents low-level characters from power-levelling inside the arena.
#[derive(Debug, Clone)]
pub struct ArenaEntrySnapshot {
    pub level: u32,
    pub xp_to_next: u32,
    pub hp: i32,
    pub max_hp: i32,
    pub attack_power: i32,
    pub defense: i32,
    pub prestige: u32,
    pub gold: u32,
}

impl ArenaEntrySnapshot {
    pub fn from_character(character: &Character) -> Self {
        Self {
            level: character.level,
            xp_to_next: character.xp_to_next,
            hp: character.hp,
            max_hp: character.max_hp,
            attack_power: character.attack_power(),
            defense: character.defense(),
            prestige: character.total_prestiges,
            gold: character.gold,
        }
    }
}

/// A chest milestone earned at a specific round.
///
/// Chests are banked in-memory and resolved only when the run ends in
/// `CashOut` or `Victory`.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingChest {
    pub round: u32,
    pub danger: u32,
}

/// Banked rewards accumulated during an arena run.
///
/// Nothing in this struct is written to disk until the run resolves.
#[derive(Debug, Clone, Default)]
pub struct PendingRewards {
    pub gold: u32,
    pub xp: u32,
    pub chests: Vec<PendingChest>,
}

impl PendingRewards {
    pub fn is_empty(&self) -> bool {
        self.gold == 0 && self.xp == 0 && self.chests.is_empty()
    }
}

/// In-memory arena run state.
///
/// `ArenaRun` is constructed at entry, mutated round-by-round, and dropped
/// when the session ends. It must **never** be stored inside `GameState`.
#[derive(Debug, Clone)]
pub struct ArenaRun {
    pub tier: ArenaTier,
    pub entry: ArenaEntrySnapshot,
    pub entry_fee: u32,
    pub rounds_cleared: u32,
    pub current_hp: i32,
    pub pending: PendingRewards,
}

/// Final resolution of an arena run.
#[derive(Debug, Clone)]
pub enum ArenaOutcome {
    /// Player was knocked out in combat. Unbanked rewards are discarded.
    Defeat { rounds_cleared: u32 },
    /// Player chose to cash out after a victory. Banked rewards are committed.
    CashOut { rounds_cleared: u32 },
    /// All rounds in the tier were cleared.
    Victory { rounds_cleared: u32 },
}

/// Transactional payload describing every mutation to apply to `GameState`
/// when an arena run resolves.
///
/// The arena loop produces an `ArenaCommit` and the caller applies it in a
/// single atomic save. This guarantees that fee subtraction, reward grants,
/// inventory changes, and journal updates are all committed together.
#[derive(Debug, Clone)]
pub struct ArenaCommit {
    pub outcome: ArenaOutcome,
    /// Entry fee already subtracted from the player at the start of the run.
    pub fee: u32,
    /// Total gold to award (this is the banked gold, not net of fee).
    pub gold_reward: u32,
    /// Total XP to award.
    pub xp_reward: u32,
    /// Items resolved from pending chests.
    pub items: Vec<Item>,
    /// Gold granted for chest items that could not fit in inventory.
    pub gold_from_overflow: u32,
    /// Number of kills (rounds cleared) to add to the character.
    pub kills: u32,
    /// New value for `best_tournament_round` if it improved.
    pub best_round: Option<u32>,
    /// Amount to increment `tournament_wins` (0 or 1).
    pub tournament_wins_increment: u32,
    /// HP to set after the run resolves. Always `Some`.
    pub hp_set: Option<i32>,
    /// Journal summary message to append.
    pub journal_msg: String,
}

/// Centralized tuning table for all arena combat parameters.
/// All literal tuning values live here so they can be adjusted in one place.
pub struct ArenaCombatTuning {
    pub enemy_hp_base: i32,
    pub enemy_hp_per_round: i32,
    pub enemy_hp_max_hp_divisor: i32,
    pub enemy_hp_per_prestige: i32,
    pub enemy_attack_base: i32,
    pub enemy_attack_per_round: i32,
    pub enemy_attack_power_divisor: i32,
    pub enemy_attack_per_prestige: i32,
    pub player_hit_fumble: i32,
    pub player_hit_threshold: i32,
    pub player_dmg_power_divisor: i32,
    pub enemy_hit_threshold_base: i32,
    pub enemy_hit_defense_divisor: i32,
    pub enemy_hit_crit: i32,
    pub enemy_dmg_defense_divisor: i32,
    pub recovery_base: i32,
    pub recovery_max_hp_divisor: i32,
    pub max_combat_lines: usize,
}

pub const ARENA_TUNING: ArenaCombatTuning = ArenaCombatTuning {
    enemy_hp_base: 20,
    enemy_hp_per_round: 10,
    enemy_hp_max_hp_divisor: 4,
    enemy_hp_per_prestige: 20,
    enemy_attack_base: 4,
    enemy_attack_per_round: 3,
    enemy_attack_power_divisor: 2,
    enemy_attack_per_prestige: 8,
    player_hit_fumble: 1,
    player_hit_threshold: 10,
    player_dmg_power_divisor: 2,
    enemy_hit_threshold_base: 8,
    enemy_hit_defense_divisor: 2,
    enemy_hit_crit: 20,
    enemy_dmg_defense_divisor: 3,
    recovery_base: 4,
    recovery_max_hp_divisor: 10,
    max_combat_lines: 8,
};

const ENEMY_NAMES: &[&str] = &[
    "Segmentation Fault Sprite",
    "Buffer Overflow Beast",
    "Null Pointer Imp",
    "Race Condition Raider",
    "Deadlock Demon",
    "Memory Leach",
    "Stack Smasher",
    "Heap Corruptor",
    "Infinite Loop Lich",
    "Divide by Zero Demon",
    "Off-by-One Assassin",
    "Deprecated Daemon",
    "Legacy Code Lurker",
    "Dependency Hell Hound",
    "Merge Conflict Monster",
    "Git Rebase Revenant",
    "Compilation Error Centaur",
    "Syntax Error Serpent",
    "Type Mismatch Troll",
    "Unhandled Exception Entity",
    "Floating Point Phantom",
    "Integer Overflow Ogre",
    "Cache Miss Wraith",
    "Page Fault Phantom",
    "Bus Error Banshee",
];

#[derive(Debug)]
struct ArenaEnemy {
    name: String,
    hp: i32,
    max_hp: i32,
    attack: i32,
}

fn generate_enemy(round: u32, entry: &ArenaEntrySnapshot, rng: &mut impl Rng) -> ArenaEnemy {
    let name = ENEMY_NAMES[rng.gen_range(0..ENEMY_NAMES.len())].to_string();
    let t = &ARENA_TUNING;
    let max_hp = t.enemy_hp_base
        + (round as i32) * t.enemy_hp_per_round
        + entry.max_hp / t.enemy_hp_max_hp_divisor
        + (entry.prestige as i32) * t.enemy_hp_per_prestige;
    let attack = t.enemy_attack_base
        + (round as i32) * t.enemy_attack_per_round
        + entry.attack_power / t.enemy_attack_power_divisor
        + (entry.prestige as i32) * t.enemy_attack_per_prestige;
    ArenaEnemy {
        name,
        hp: max_hp,
        max_hp,
        attack,
    }
}

#[derive(Debug)]
struct CombatExchange {
    plain: String,
    colored: String,
}

#[derive(Debug)]
struct CombatResult {
    player_won: bool,
    final_player_hp: i32,
    exchanges: Vec<CombatExchange>,
    total_turns: u32,
}

fn run_compact_combat(
    entry: &ArenaEntrySnapshot,
    current_hp: i32,
    enemy: &mut ArenaEnemy,
    class: &crate::character::Class,
    rng: &mut impl Rng,
) -> CombatResult {
    let mut player_hp = current_hp;
    let mut total_turns = 0u32;
    let mut all_exchanges: Vec<CombatExchange> = Vec::new();

    loop {
        total_turns += 1;
        let mut turn_lines: Vec<CombatExchange> = Vec::new();

        let t = &ARENA_TUNING;
        let player_power = entry.attack_power;
        let hit_roll: i32 = rng.gen_range(1..=20);
        if hit_roll != t.player_hit_fumble && hit_roll + player_power > t.player_hit_threshold {
            let dmg = rng.gen_range(
                (player_power / t.player_dmg_power_divisor).max(1)
                    ..=player_power.max(1),
            );
            enemy.hp -= dmg;
            let (plain, colored) = crate::messages::tournament_player_hit(
                class,
                &enemy.name,
                dmg,
                enemy.hp.max(0),
                enemy.max_hp,
            );
            turn_lines.push(CombatExchange { plain, colored });
        } else {
            let (plain, colored) =
                crate::messages::tournament_player_miss(class, &enemy.name);
            turn_lines.push(CombatExchange { plain, colored });
        }

        if enemy.hp <= 0 {
            all_exchanges.extend(turn_lines);
            return CombatResult {
                player_won: true,
                final_player_hp: player_hp,
                exchanges: compact_exchanges(all_exchanges, total_turns),
                total_turns,
            };
        }

        let player_defense = entry.defense;
        let dodge_roll: i32 = rng.gen_range(1..=20);
        if dodge_roll > (t.enemy_hit_threshold_base + player_defense / t.enemy_hit_defense_divisor)
            || dodge_roll == t.enemy_hit_crit
        {
            let dmg = (enemy.attack - player_defense / t.enemy_dmg_defense_divisor).max(1);
            player_hp -= dmg;
            let (plain, colored) = crate::messages::tournament_enemy_hit(
                &enemy.name,
                dmg,
                player_hp.max(0),
                entry.max_hp,
                total_turns,
            );
            turn_lines.push(CombatExchange { plain, colored });
        } else {
            let (plain, colored) =
                crate::messages::tournament_enemy_miss(&enemy.name, total_turns);
            turn_lines.push(CombatExchange { plain, colored });
        }

        if player_hp <= 0 {
            all_exchanges.extend(turn_lines);
            return CombatResult {
                player_won: false,
                final_player_hp: player_hp,
                exchanges: compact_exchanges(all_exchanges, total_turns),
                total_turns,
            };
        }

        all_exchanges.extend(turn_lines);
    }
}

fn compact_exchanges(exchanges: Vec<CombatExchange>, total_turns: u32) -> Vec<CombatExchange> {
    if exchanges.len() <= ARENA_TUNING.max_combat_lines {
        return exchanges;
    }

    let mut result = Vec::new();

    let prefix_len = 4.min(exchanges.len());
    for i in 0..prefix_len {
        result.push(CombatExchange {
            plain: exchanges[i].plain.clone(),
            colored: exchanges[i].colored.clone(),
        });
    }

    let skipped = total_turns.saturating_sub(3);
    let summary_plain = format!("... {} more exchanges ...", skipped);
    let summary_colored = format!(
        "{} {} {}",
        "...".dimmed(),
        format!("{} more exchanges", skipped).dimmed(),
        "...".dimmed()
    );
    result.push(CombatExchange {
        plain: summary_plain,
        colored: summary_colored,
    });

    let suffix_start = exchanges.len().saturating_sub(2);
    for i in suffix_start..exchanges.len() {
        result.push(CombatExchange {
            plain: exchanges[i].plain.clone(),
            colored: exchanges[i].colored.clone(),
        });
    }

    result
}

fn read_line_trimmed() -> Option<String> {
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(0) => None,
        Ok(_) => Some(input.trim().to_string()),
        Err(_) => None,
    }
}

fn prompt_choice(msg: &str) -> Option<String> {
    print!("{}", msg);
    io::stdout().flush().unwrap();
    read_line_trimmed()
}

fn build_commit(
    character: &Character,
    run: &ArenaRun,
    outcome: ArenaOutcome,
    final_hp: i32,
) -> ArenaCommit {
    let (gold_reward, xp_reward, items, kills) = match outcome {
        ArenaOutcome::Defeat { .. } => (0, 0, Vec::new(), 0),
        _ => {
            let (g, x) = run
                .tier
                .compute_rewards(run.entry_fee, run.entry.xp_to_next, run.rounds_cleared);
            let chests = run.tier.collect_chests(run.rounds_cleared);
            let mut chest_items = Vec::new();
            for chest in &chests {
                chest_items.push(crate::loot::roll_loot_scaled(chest.danger));
            }
            (g, x, chest_items, run.rounds_cleared)
        }
    };

    let best_round = if run.rounds_cleared > character.best_tournament_round {
        Some(run.rounds_cleared)
    } else {
        None
    };

    let tournament_wins_increment = match outcome {
        ArenaOutcome::Victory { .. } if run.tier.index == 4 => 1,
        _ => 0,
    };

    let journal_msg = match outcome {
        ArenaOutcome::Defeat { rounds_cleared } => format!(
            "Arena KO in {} after {} rounds. Fee: {} gold.",
            run.tier.name, rounds_cleared, run.entry_fee
        ),
        ArenaOutcome::CashOut { rounds_cleared } => format!(
            "Arena cash-out in {} after {} rounds. +{} gold, +{} XP.",
            run.tier.name, rounds_cleared, gold_reward, xp_reward
        ),
        ArenaOutcome::Victory { rounds_cleared } => format!(
            "Arena VICTORY in {}! Cleared all {} rounds! +{} gold, +{} XP.",
            run.tier.name, rounds_cleared, gold_reward, xp_reward
        ),
    };

    ArenaCommit {
        outcome,
        fee: run.entry_fee,
        gold_reward,
        xp_reward,
        items,
        gold_from_overflow: 0,
        kills,
        best_round,
        tournament_wins_increment,
        hp_set: Some(final_hp),
        journal_msg,
    }
}

pub fn apply_arena_commit(game: &mut crate::state::GameState, commit: &ArenaCommit) {
    game.character.gold = game.character.gold.saturating_sub(commit.fee);

    match commit.outcome {
        ArenaOutcome::Defeat { .. } => {
            game.character.hp = commit.hp_set.unwrap_or(1);
            if let Some(best) = commit.best_round {
                if best > game.character.best_tournament_round {
                    game.character.best_tournament_round = best;
                }
            }
        }
        ArenaOutcome::CashOut { .. } | ArenaOutcome::Victory { .. } => {
            let mut total_gold = commit.gold_reward;

            let leveled = game.character.gain_xp_arena_safe(commit.xp_reward);
            if leveled {
                let (_plain, colored) = crate::messages::level_up(
                    &game.character.class,
                    game.character.level,
                    &game.character.title,
                );
                crate::display::print_level_up(&colored);
            }

            game.character.kills = game.character.kills.saturating_add(commit.kills);

            let mut overflow_gold: u32 = 0;
            for item in &commit.items {
                if !crate::events::add_to_inventory_pub_quiet(game, item.clone()) {
                    let sell_value = crate::loot::item_price(item) / 2;
                    overflow_gold = overflow_gold.saturating_add(sell_value);
                    eprintln!(
                        "   {} Arena chest item {} converted to {} gold (inventory full).",
                        "💰".yellow(),
                        item.name.dimmed(),
                        sell_value
                    );
                }
            }
            if overflow_gold > 0 {
                total_gold = total_gold.saturating_add(overflow_gold);
            }

            if let Some(best) = commit.best_round {
                if best > game.character.best_tournament_round {
                    game.character.best_tournament_round = best;
                }
            }

            game.character.tournament_wins = game.character
                .tournament_wins
                .saturating_add(commit.tournament_wins_increment);

            game.character.gold = game.character.gold.saturating_add(total_gold);
            game.character.hp = commit.hp_set.unwrap_or(game.character.hp);
        }
    }

    game.add_journal(crate::journal::JournalEntry::new(
        crate::journal::EventType::Tournament,
        commit.journal_msg.clone(),
    ));
}

pub fn run_arena_session(
    character: &Character,
    tier: ArenaTier,
    entry_fee: u32,
) -> Option<ArenaCommit> {
    let mut rng = rand::thread_rng();
    let entry = ArenaEntrySnapshot::from_character(character);
    let class = &character.class;

    let mut run = ArenaRun {
        tier,
        entry: entry.clone(),
        entry_fee,
        rounds_cleared: 0,
        current_hp: entry.hp,
        pending: PendingRewards::default(),
    };

    eprintln!();
    eprintln!(
        "{}",
        "╔══════════════════════════════════════════════╗"
            .yellow()
            .bold()
    );
    eprintln!(
        "{} {} {}",
        "║".yellow().bold(),
        format!("🏟️  {}", tier.name).yellow().bold(),
        "║".yellow().bold()
    );
    eprintln!(
        "{} {} {}",
        "║".yellow().bold(),
        format!(
            "   Entry fee: {} gold  HP: {}/{}",
            entry_fee, entry.hp, entry.max_hp
        )
        .yellow(),
        "║".yellow().bold()
    );
    eprintln!(
        "{}",
        "╚══════════════════════════════════════════════╝"
            .yellow()
            .bold()
    );
    eprintln!();

    loop {
        let round = run.rounds_cleared + 1;
        if round > tier.max_rounds {
            return Some(build_commit(
                character,
                &run,
                ArenaOutcome::Victory {
                    rounds_cleared: run.rounds_cleared,
                },
                run.current_hp,
            ));
        }

        let mut enemy = generate_enemy(round, &entry, &mut rng);

        let (_plain, colored) =
            crate::messages::tournament_round_intro(&class, round, &enemy.name);
        eprintln!("{} {}", "⚔️".bold(), colored);
        eprintln!("{}", "─".repeat(40).dimmed());

        let combat = run_compact_combat(&entry, run.current_hp, &mut enemy, &class, &mut rng);
        run.current_hp = combat.final_player_hp;

        for ex in &combat.exchanges {
            eprintln!("   {}", ex.colored);
        }

        if !combat.player_won {
            eprintln!("{}", "─".repeat(40).dimmed());
            let (_plain, colored) =
                crate::messages::tournament_ko(run.rounds_cleared, 0, 0);
            eprintln!("{} {}", "💀".bold(), colored);
            return Some(build_commit(
                character,
                &run,
                ArenaOutcome::Defeat {
                    rounds_cleared: run.rounds_cleared,
                },
                (entry.max_hp / 4).max(1),
            ));
        }

        run.rounds_cleared = round;

        if run.rounds_cleared >= tier.max_rounds {
            return Some(build_commit(
                character,
                &run,
                ArenaOutcome::Victory {
                    rounds_cleared: run.rounds_cleared,
                },
                run.current_hp,
            ));
        }

        let t = &ARENA_TUNING;
        let recovery = t.recovery_base.max(entry.max_hp / t.recovery_max_hp_divisor);
        let healed = recovery.min(entry.max_hp - run.current_hp);
        run.current_hp = (run.current_hp + recovery).min(entry.max_hp);
        if healed > 0 {
            eprintln!(
                "   {} Recovered {} HP. HP: {}/{}",
                "🩹".dimmed(),
                healed,
                run.current_hp,
                entry.max_hp
            );
        }

        eprintln!("{}", "─".repeat(40).dimmed());
        eprintln!(
            "   Round {} cleared! Current HP: {}/{}",
            run.rounds_cleared, run.current_hp, entry.max_hp
        );
        eprintln!("   1) Continue");
        eprintln!("   2) Cash Out");

        loop {
            let choice = prompt_choice("   Choose [1-2]: ");
            match choice.as_deref() {
                Some("1") => break,
                Some("2") => {
                    return Some(build_commit(
                        character,
                        &run,
                        ArenaOutcome::CashOut {
                            rounds_cleared: run.rounds_cleared,
                        },
                        run.current_hp,
                    ));
                }
                Some(_) => {
                    eprintln!("   Invalid choice. Enter 1 or 2.");
                    continue;
                }
                None => {
                    eprintln!("   (EOF detected — cashing out)");
                    return Some(build_commit(
                        character,
                        &run,
                        ArenaOutcome::CashOut {
                            rounds_cleared: run.rounds_cleared,
                        },
                        run.current_hp,
                    ));
                }
            }
        }

        eprintln!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::{Class, Item, ItemSlot, Race, Rarity};

    fn make_character(level: u32, total_prestiges: u32, gold: u32) -> Character {
        let mut c = Character::new("Test".to_string(), Class::Warrior, Race::Human);
        c.level = level;
        c.total_prestiges = total_prestiges;
        c.gold = gold;
        // Ensure xp_to_next matches the level-up formula for the current level
        if level > 1 {
            // Recalculate xp_to_next by simulating level-ups
            c.xp_to_next = 25;
            for lvl in 1..level {
                c.xp_to_next = match lvl {
                    1..=10 => lvl * 15 + 10,
                    11..=30 => lvl * 25 + 30,
                    31..=60 => lvl * 45 + 80,
                    61..=100 => lvl * 80 + 200,
                    101..=130 => lvl * 120 + 400,
                    _ => lvl * 170 + 800,
                };
            }
        }
        c
    }

    fn make_snapshot(level: u32, prestige: u32, gold: u32, xp_to_next: u32) -> ArenaEntrySnapshot {
        ArenaEntrySnapshot {
            level,
            xp_to_next,
            hp: 100,
            max_hp: 100,
            attack_power: 20,
            defense: 10,
            prestige,
            gold,
        }
    }

    // --- Unlock rules ---

    #[test]
    fn pit_always_unlocked() {
        let c = make_character(1, 0, 10);
        assert!(TIER_PIT.is_unlocked(&c));
    }

    #[test]
    fn gauntlet_unlocked_by_level() {
        let c = make_character(25, 0, 100);
        assert!(TIER_GAUNTLET.is_unlocked(&c));
    }

    #[test]
    fn gauntlet_unlocked_by_prestige() {
        let c = make_character(1, 1, 100);
        assert!(TIER_GAUNTLET.is_unlocked(&c));
    }

    #[test]
    fn gauntlet_locked_for_low_level_no_prestige() {
        let c = make_character(24, 0, 100);
        assert!(!TIER_GAUNTLET.is_unlocked(&c));
    }

    #[test]
    fn colosseum_unlocked_by_level() {
        let c = make_character(60, 0, 100);
        assert!(TIER_COLOSSEUM.is_unlocked(&c));
    }

    #[test]
    fn colosseum_unlocked_by_prestige() {
        let c = make_character(1, 1, 100);
        assert!(TIER_COLOSSEUM.is_unlocked(&c));
    }

    #[test]
    fn colosseum_locked_for_low_level_no_prestige() {
        let c = make_character(59, 0, 100);
        assert!(!TIER_COLOSSEUM.is_unlocked(&c));
    }

    #[test]
    fn abyssal_unlocked_by_level() {
        let c = make_character(100, 0, 100);
        assert!(TIER_ABYSSAL.is_unlocked(&c));
    }

    #[test]
    fn abyssal_unlocked_by_prestige() {
        let c = make_character(1, 2, 100);
        assert!(TIER_ABYSSAL.is_unlocked(&c));
    }

    #[test]
    fn abyssal_locked_for_low_level_no_prestige() {
        let c = make_character(99, 0, 100);
        assert!(!TIER_ABYSSAL.is_unlocked(&c));
    }

    #[test]
    fn abyssal_locked_for_one_prestige() {
        let c = make_character(1, 1, 100);
        assert!(!TIER_ABYSSAL.is_unlocked(&c));
    }

    #[test]
    fn godslayer_locked_without_prestige() {
        let c = make_character(150, 0, 100);
        assert!(!TIER_GODSLAYER.is_unlocked(&c));
    }

    #[test]
    fn godslayer_locked_without_level() {
        let c = make_character(149, 3, 100);
        assert!(!TIER_GODSLAYER.is_unlocked(&c));
    }

    #[test]
    fn godslayer_unlocked_at_max_with_prestige() {
        let c = make_character(150, 3, 100);
        assert!(TIER_GODSLAYER.is_unlocked(&c));
    }

    // --- Fee formulas ---

    #[test]
    fn pit_fee_level_based() {
        let entry = make_snapshot(12, 0, 200, 330);
        // max(40, 144, 20)
        assert_eq!(TIER_PIT.compute_fee(&entry), 144);
    }

    #[test]
    fn pit_fee_gold_based() {
        let entry = make_snapshot(12, 0, 3000, 330);
        // max(40, 144, 300)
        assert_eq!(TIER_PIT.compute_fee(&entry), 300);
    }

    #[test]
    fn pit_fee_floor() {
        let entry = make_snapshot(1, 0, 10, 25);
        // max(40, 12, 1)
        assert_eq!(TIER_PIT.compute_fee(&entry), 40);
    }

    #[test]
    fn gauntlet_fee_formula() {
        let entry = make_snapshot(30, 1, 1000, 1000);
        // max(100, 30*18 + 1*50 = 590, 1000/8 = 125)
        assert_eq!(TIER_GAUNTLET.compute_fee(&entry), 590);
    }

    #[test]
    fn colosseum_fee_formula() {
        let entry = make_snapshot(60, 1, 2000, 2000);
        // max(300, 60*28 + 1*150 = 1830, 2000/6 = 333)
        assert_eq!(TIER_COLOSSEUM.compute_fee(&entry), 1830);
    }

    #[test]
    fn abyssal_fee_formula() {
        let entry = make_snapshot(100, 2, 5000, 5000);
        // max(800, 100*40 + 2*250 = 4500, 5000/5 = 1000)
        assert_eq!(TIER_ABYSSAL.compute_fee(&entry), 4500);
    }

    #[test]
    fn godslayer_fee_formula() {
        let entry = make_snapshot(150, 3, 10000, 10000);
        // max(2500, 150*60 + 3*400 = 10200, 10000/4 = 2500)
        assert_eq!(TIER_GODSLAYER.compute_fee(&entry), 10200);
    }

    // --- Interpolation / milestone math ---

    #[test]
    fn pit_rewards_at_milestones() {
        assert_eq!(TIER_PIT.reward_percentages_at_round(1), (10, 5));
        assert_eq!(TIER_PIT.reward_percentages_at_round(3), (45, 24));
        assert_eq!(TIER_PIT.reward_percentages_at_round(5), (110, 60));
    }

    #[test]
    fn gauntlet_rewards_at_milestones() {
        assert_eq!(TIER_GAUNTLET.reward_percentages_at_round(5), (35, 22));
        assert_eq!(TIER_GAUNTLET.reward_percentages_at_round(10), (145, 90));
    }

    #[test]
    fn godslayer_rewards_at_milestones() {
        assert_eq!(TIER_GODSLAYER.reward_percentages_at_round(10), (25, 15));
        assert_eq!(TIER_GODSLAYER.reward_percentages_at_round(50), (320, 220));
    }

    #[test]
    fn interpolation_gauntlet_round_7() {
        // Between r5 (35/22) and r10 (145/90), span = 5, step = 2
        // gold: 35 + (145-35)*2/5 = 35 + 44 = 79
        // xp: 22 + (90-22)*2/5 = 22 + 27 = 49
        assert_eq!(TIER_GAUNTLET.reward_percentages_at_round(7), (79, 49));
    }

    #[test]
    fn interpolation_colosseum_round_12() {
        // Between r10 (90/55) and r15 (185/120), span = 5, step = 2
        // gold: 90 + (185-90)*2/5 = 90 + 38 = 128
        // xp: 55 + (120-55)*2/5 = 55 + 26 = 81
        assert_eq!(TIER_COLOSSEUM.reward_percentages_at_round(12), (128, 81));
    }

    #[test]
    fn interpolation_godslayer_round_35() {
        // Between r30 (130/85) and r40 (210/150), span = 10, step = 5
        // gold: 130 + (210-130)*5/10 = 130 + 40 = 170
        // xp: 85 + (150-85)*5/10 = 85 + 32 = 117
        assert_eq!(TIER_GODSLAYER.reward_percentages_at_round(35), (170, 117));
    }

    #[test]
    fn interpolation_before_first_milestone() {
        // The Gauntlet first milestone is r5. At r3, interpolate from (0,0).
        // gold: 35 * 3 / 5 = 21
        // xp: 22 * 3 / 5 = 13
        assert_eq!(TIER_GAUNTLET.reward_percentages_at_round(3), (21, 13));
    }

    #[test]
    fn clamp_past_last_milestone() {
        assert_eq!(TIER_PIT.reward_percentages_at_round(10), (110, 60));
        assert_eq!(TIER_GAUNTLET.reward_percentages_at_round(50), (145, 90));
    }

    #[test]
    fn round_zero_returns_zero() {
        assert_eq!(TIER_PIT.reward_percentages_at_round(0), (0, 0));
        assert_eq!(TIER_GODSLAYER.reward_percentages_at_round(0), (0, 0));
    }

    // --- Reward bounds: level-12 full-clear The Pit ---

    #[test]
    fn level_12_pit_full_clear_net_gold_bound() {
        let entry = make_snapshot(12, 0, 200, 330);
        let fee = TIER_PIT.compute_fee(&entry);
        let (gold_reward, xp_reward) = TIER_PIT.compute_rewards(fee, entry.xp_to_next, 5);
        let net_gold = gold_reward as i64 - fee as i64;
        assert!(
            net_gold >= 0 && net_gold <= 25,
            "net_gold = {} (fee={}, reward={})",
            net_gold,
            fee,
            gold_reward
        );
        let xp_pct = xp_reward as f64 / entry.xp_to_next as f64;
        assert!(
            xp_pct >= 0.50 && xp_pct <= 0.70,
            "xp_pct = {} (xp_reward={}, threshold={})",
            xp_pct,
            xp_reward,
            entry.xp_to_next
        );
    }

    // --- Chest milestones ---

    #[test]
    fn pit_chests_at_round_5() {
        let chests = TIER_PIT.collect_chests(5);
        assert_eq!(chests.len(), 1);
        assert_eq!(chests[0].round, 5);
        assert_eq!(chests[0].danger, 2);
    }

    #[test]
    fn pit_no_chests_before_round_5() {
        let chests = TIER_PIT.collect_chests(4);
        assert!(chests.is_empty());
    }

    #[test]
    fn gauntlet_chests_progression() {
        let c5 = TIER_GAUNTLET.collect_chests(5);
        assert_eq!(c5.len(), 1);
        assert_eq!(c5[0].danger, 2);

        let c10 = TIER_GAUNTLET.collect_chests(10);
        assert_eq!(c10.len(), 2);
        assert_eq!(c10[1].danger, 4);
    }

    #[test]
    fn godslayer_chest_milestones() {
        let c = TIER_GODSLAYER.collect_chests(50);
        assert_eq!(c.len(), 4);
        assert_eq!(c[0], PendingChest { round: 10, danger: 4 });
        assert_eq!(c[1], PendingChest { round: 20, danger: 6 });
        assert_eq!(c[2], PendingChest { round: 40, danger: 8 });
        assert_eq!(c[3], PendingChest { round: 50, danger: 9 });
    }

    #[test]
    fn chests_past_last_milestone_no_duplicates() {
        let c = TIER_GODSLAYER.collect_chests(100);
        assert_eq!(c.len(), 4);
    }

    // --- Snapshot from character ---

    #[test]
    fn snapshot_copies_character_fields() {
        let mut c = make_character(10, 1, 500);
        c.hp = 80;
        c.max_hp = 100;
        let snap = ArenaEntrySnapshot::from_character(&c);
        assert_eq!(snap.level, 10);
        assert_eq!(snap.prestige, 1);
        assert_eq!(snap.gold, 500);
        assert_eq!(snap.hp, 80);
        assert_eq!(snap.max_hp, 100);
        assert_eq!(snap.xp_to_next, c.xp_to_next);
        assert_eq!(snap.attack_power, c.attack_power());
        assert_eq!(snap.defense, c.defense());
    }

    // --- Arena tiers array ordering ---

    #[test]
    fn arena_tiers_has_five_tiers() {
        assert_eq!(ARENA_TIERS.len(), 5);
        assert_eq!(ARENA_TIERS[0].name, "The Pit");
        assert_eq!(ARENA_TIERS[4].name, "Godslayer's Court");
    }

    use crate::state::GameState;

    #[test]
    fn apply_commit_defeat_subtracts_fee_and_sets_hp() {
        let mut game = GameState::new(make_character(10, 0, 500));
        game.character.max_hp = 100;
        game.character.hp = 50;
        game.character.best_tournament_round = 0;

        let commit = ArenaCommit {
            outcome: ArenaOutcome::Defeat { rounds_cleared: 2 },
            fee: 50,
            gold_reward: 0,
            xp_reward: 0,
            items: vec![],
            gold_from_overflow: 0,
            kills: 0,
            best_round: Some(2),
            tournament_wins_increment: 0,
            hp_set: Some(25),
            journal_msg: "KO".to_string(),
        };

        apply_arena_commit(&mut game, &commit);

        assert_eq!(game.character.gold, 450);
        assert_eq!(game.character.hp, 25);
        assert_eq!(game.character.best_tournament_round, 2);
        assert_eq!(game.character.deaths, 0);
        assert_eq!(game.character.kills, 0);
        assert_eq!(game.character.xp, 0);
    }

    #[test]
    fn apply_commit_cash_out_adds_rewards() {
        let mut game = GameState::new(make_character(10, 0, 500));
        game.character.max_hp = 100;
        game.character.hp = 40;
        game.character.best_tournament_round = 0;
        game.character.kills = 5;

        let commit = ArenaCommit {
            outcome: ArenaOutcome::CashOut { rounds_cleared: 3 },
            fee: 50,
            gold_reward: 100,
            xp_reward: 30,
            items: vec![],
            gold_from_overflow: 0,
            kills: 3,
            best_round: Some(3),
            tournament_wins_increment: 0,
            hp_set: Some(45),
            journal_msg: "Cash out".to_string(),
        };

        apply_arena_commit(&mut game, &commit);

        assert_eq!(game.character.gold, 550);
        assert_eq!(game.character.hp, 45);
        assert_eq!(game.character.best_tournament_round, 3);
        assert_eq!(game.character.kills, 8);
        assert_eq!(game.character.deaths, 0);
    }

    #[test]
    fn apply_commit_victory_godslayer_increments_wins() {
        let mut game = GameState::new(make_character(150, 3, 5000));
        game.character.best_tournament_round = 0;
        game.character.tournament_wins = 0;

        let commit = ArenaCommit {
            outcome: ArenaOutcome::Victory { rounds_cleared: 50 },
            fee: 1000,
            gold_reward: 200,
            xp_reward: 50,
            items: vec![],
            gold_from_overflow: 0,
            kills: 50,
            best_round: Some(50),
            tournament_wins_increment: 1,
            hp_set: Some(80),
            journal_msg: "Victory".to_string(),
        };

        apply_arena_commit(&mut game, &commit);

        assert_eq!(game.character.tournament_wins, 1);
        assert_eq!(game.character.best_tournament_round, 50);
    }

    #[test]
    fn apply_commit_victory_pit_does_not_increment_wins() {
        let mut game = GameState::new(make_character(10, 0, 500));
        game.character.tournament_wins = 0;

        let commit = ArenaCommit {
            outcome: ArenaOutcome::Victory { rounds_cleared: 5 },
            fee: 50,
            gold_reward: 100,
            xp_reward: 30,
            items: vec![],
            gold_from_overflow: 0,
            kills: 5,
            best_round: Some(5),
            tournament_wins_increment: 0,
            hp_set: Some(80),
            journal_msg: "Victory".to_string(),
        };

        apply_arena_commit(&mut game, &commit);

        assert_eq!(game.character.tournament_wins, 0);
    }

    #[test]
    fn apply_commit_no_heal_on_level_up() {
        let mut game = GameState::new(make_character(1, 0, 100));
        game.character.xp = 20;
        game.character.xp_to_next = 25;
        game.character.max_hp = 34;
        game.character.hp = 10;

        let commit = ArenaCommit {
            outcome: ArenaOutcome::CashOut { rounds_cleared: 1 },
            fee: 10,
            gold_reward: 20,
            xp_reward: 10,
            items: vec![],
            gold_from_overflow: 0,
            kills: 1,
            best_round: Some(1),
            tournament_wins_increment: 0,
            hp_set: Some(10),
            journal_msg: "Cash out".to_string(),
        };

        apply_arena_commit(&mut game, &commit);

        assert_eq!(game.character.level, 2);
        assert_eq!(game.character.hp, 10);
        assert!(game.character.max_hp > 34);
    }

    #[test]
    fn apply_commit_level_up_adds_only_arena_journal_entry() {
        let mut game = GameState::new(make_character(1, 0, 100));
        game.character.xp = 20;
        game.character.xp_to_next = 25;
        game.character.max_hp = 34;
        game.character.hp = 10;

        let commit = ArenaCommit {
            outcome: ArenaOutcome::CashOut { rounds_cleared: 1 },
            fee: 10,
            gold_reward: 20,
            xp_reward: 10,
            items: vec![],
            gold_from_overflow: 0,
            kills: 1,
            best_round: Some(1),
            tournament_wins_increment: 0,
            hp_set: Some(10),
            journal_msg: "Arena cash-out".to_string(),
        };

        apply_arena_commit(&mut game, &commit);

        assert_eq!(game.character.level, 2);
        assert_eq!(game.journal.len(), 1);
        assert!(matches!(
            game.journal[0].event_type,
            crate::journal::EventType::Tournament
        ));
    }

    #[test]
    fn build_commit_defeat_zeros_rewards() {
        let c = make_character(10, 0, 500);
        let entry = ArenaEntrySnapshot::from_character(&c);
        let run = ArenaRun {
            tier: TIER_PIT,
            entry: entry.clone(),
            entry_fee: 100,
            rounds_cleared: 3,
            current_hp: 30,
            pending: PendingRewards::default(),
        };

        let commit = build_commit(&c, &run, ArenaOutcome::Defeat { rounds_cleared: 3 }, 25);
        assert_eq!(commit.gold_reward, 0);
        assert_eq!(commit.xp_reward, 0);
        assert!(commit.items.is_empty());
        assert_eq!(commit.kills, 0);
        assert_eq!(commit.tournament_wins_increment, 0);
    }

    #[test]
    fn build_commit_victory_godslayer_increments_wins() {
        let c = make_character(150, 3, 5000);
        let entry = ArenaEntrySnapshot::from_character(&c);
        let run = ArenaRun {
            tier: TIER_GODSLAYER,
            entry: entry.clone(),
            entry_fee: 2500,
            rounds_cleared: 50,
            current_hp: 80,
            pending: PendingRewards::default(),
        };

        let commit = build_commit(&c, &run, ArenaOutcome::Victory { rounds_cleared: 50 }, 80);
        assert_eq!(commit.tournament_wins_increment, 1);
    }

    #[test]
    fn build_commit_victory_pit_no_win_increment() {
        let c = make_character(10, 0, 500);
        let entry = ArenaEntrySnapshot::from_character(&c);
        let run = ArenaRun {
            tier: TIER_PIT,
            entry: entry.clone(),
            entry_fee: 100,
            rounds_cleared: 5,
            current_hp: 80,
            pending: PendingRewards::default(),
        };

        let commit = build_commit(&c, &run, ArenaOutcome::Victory { rounds_cleared: 5 }, 80);
        assert_eq!(commit.tournament_wins_increment, 0);
    }

    #[test]
    fn apply_commit_chest_overflow_converts_to_half_gold() {
        use crate::character::{Item, ItemSlot, Rarity};

        let mut game = GameState::new(make_character(10, 0, 500));
        game.character.max_hp = 100;
        game.character.hp = 80;
        game.character.best_tournament_round = 0;

        for i in 0..20 {
            game.character.inventory.push(Item {
                name: format!("Legendary {}", i),
                slot: ItemSlot::Weapon,
                power: 50 + i as i32,
                rarity: Rarity::Legendary,
            });
        }

        let chest_item = Item {
            name: "Rusty Dagger".to_string(),
            slot: ItemSlot::Weapon,
            power: 2,
            rarity: Rarity::Common,
        };
        let sell_value = crate::loot::item_price(&chest_item) / 2;

        let commit = ArenaCommit {
            outcome: ArenaOutcome::CashOut { rounds_cleared: 3 },
            fee: 50,
            gold_reward: 100,
            xp_reward: 0,
            items: vec![chest_item],
            gold_from_overflow: 0,
            kills: 3,
            best_round: Some(3),
            tournament_wins_increment: 0,
            hp_set: Some(80),
            journal_msg: "Cash out with overflow".to_string(),
        };

        apply_arena_commit(&mut game, &commit);

        let expected_gold = 500u32.saturating_sub(50).saturating_add(100).saturating_add(sell_value);
        assert_eq!(game.character.gold, expected_gold, "expected {} gold (base 500 - fee 50 + reward 100 + overflow {}), got {}", expected_gold, sell_value, game.character.gold);
        assert_eq!(game.character.inventory.len(), 20);
    }

    // --- Tier unlock gaps ---

    #[test]
    fn godslayer_locked_at_150_prestige_2() {
        let c = make_character(150, 2, 100);
        assert!(!TIER_GODSLAYER.is_unlocked(&c));
    }

    // --- Fee edge cases ---

    #[test]
    fn fee_invalid_index_returns_zero() {
        let fake = ArenaTier {
            index: 99,
            name: "Fake Tier",
            max_rounds: 1,
            min_level: 0,
            min_prestige: 0,
            or_unlock: false,
            reward_bands: &[],
            chest_milestones: &[],
        };
        let entry = make_snapshot(1, 0, 10, 25);
        assert_eq!(fake.compute_fee(&entry), 0);
    }

    // --- Interpolation gaps ---

    #[test]
    fn colosseum_rewards_before_first_milestone() {
        // First milestone at r5. At r3, interpolate from (0,0).
        // gold: 30 * 3 / 5 = 18
        // xp: 20 * 3 / 5 = 12
        assert_eq!(TIER_COLOSSEUM.reward_percentages_at_round(3), (18, 12));
    }

    #[test]
    fn abyssal_rewards_interpolation_round_12() {
        // Between r10 (55/30) and r15 (100/60), span=5, step=2
        // gold: 55 + (100-55)*2/5 = 55 + 18 = 73
        // xp: 30 + (60-30)*2/5 = 30 + 12 = 42
        assert_eq!(TIER_ABYSSAL.reward_percentages_at_round(12), (73, 42));
    }

    #[test]
    fn godslayer_rewards_before_first_milestone() {
        // First milestone at r10. At r5, interpolate from (0,0).
        // gold: 25 * 5 / 10 = 12
        // xp: 15 * 5 / 10 = 7
        assert_eq!(TIER_GODSLAYER.reward_percentages_at_round(5), (12, 7));
    }

    // --- Reward computation edge cases ---

    #[test]
    fn compute_rewards_round_zero() {
        let (g, x) = TIER_PIT.compute_rewards(100, 200, 0);
        assert_eq!(g, 0);
        assert_eq!(x, 0);
    }

    // --- Snapshot with gear ---

    #[test]
    fn snapshot_copies_equipped_stats() {
        let mut c = make_character(10, 1, 500);
        c.equip(Item {
            name: "Sword".to_string(),
            slot: ItemSlot::Weapon,
            power: 15,
            rarity: Rarity::Rare,
        });
        c.equip(Item {
            name: "Plate".to_string(),
            slot: ItemSlot::Armor,
            power: 10,
            rarity: Rarity::Uncommon,
        });
        c.equip(Item {
            name: "Ring".to_string(),
            slot: ItemSlot::Ring,
            power: 5,
            rarity: Rarity::Common,
        });

        let snap = ArenaEntrySnapshot::from_character(&c);
        assert_eq!(snap.attack_power, c.attack_power());
        assert_eq!(snap.defense, c.defense());
        assert!(snap.attack_power > 20);
        assert!(snap.defense > 10);
    }

    // --- PendingRewards ---

    #[test]
    fn pending_rewards_is_empty_when_default() {
        let p = PendingRewards::default();
        assert!(p.is_empty());
    }

    #[test]
    fn pending_rewards_is_not_empty_with_gold() {
        let mut p = PendingRewards::default();
        p.gold = 1;
        assert!(!p.is_empty());
    }

    // --- Chest collection edge cases ---

    #[test]
    fn chests_collect_at_round_zero() {
        let c = TIER_PIT.collect_chests(0);
        assert!(c.is_empty());
    }

    // --- KO semantics gaps ---

    #[test]
    fn apply_commit_defeat_no_best_round_update() {
        let mut game = GameState::new(make_character(10, 0, 500));
        game.character.max_hp = 100;
        game.character.best_tournament_round = 5;

        let commit = ArenaCommit {
            outcome: ArenaOutcome::Defeat { rounds_cleared: 3 },
            fee: 50,
            gold_reward: 0,
            xp_reward: 0,
            items: vec![],
            gold_from_overflow: 0,
            kills: 0,
            best_round: None,
            tournament_wins_increment: 0,
            hp_set: Some(25),
            journal_msg: "KO".to_string(),
        };

        apply_arena_commit(&mut game, &commit);

        assert_eq!(game.character.best_tournament_round, 5);
        assert_eq!(game.character.gold, 450);
    }

    #[test]
    fn apply_commit_defeat_does_not_grant_rewards() {
        let mut game = GameState::new(make_character(10, 0, 500));
        game.character.max_hp = 100;
        game.character.xp = 50;
        game.character.kills = 10;

        let commit = ArenaCommit {
            outcome: ArenaOutcome::Defeat { rounds_cleared: 2 },
            fee: 50,
            gold_reward: 100,
            xp_reward: 50,
            items: vec![Item {
                name: "Sword".to_string(),
                slot: ItemSlot::Weapon,
                power: 5,
                rarity: Rarity::Common,
            }],
            gold_from_overflow: 0,
            kills: 5,
            best_round: None,
            tournament_wins_increment: 0,
            hp_set: Some(25),
            journal_msg: "KO".to_string(),
        };

        apply_arena_commit(&mut game, &commit);

        assert_eq!(game.character.xp, 50);
        assert_eq!(game.character.kills, 10);
        assert_eq!(game.character.inventory.len(), 0);
        assert_eq!(game.character.gold, 450);
    }

    // --- Best round preservation ---

    #[test]
    fn apply_commit_preserves_existing_best_round() {
        let mut game = GameState::new(make_character(10, 0, 500));
        game.character.max_hp = 100;
        game.character.best_tournament_round = 10;

        let commit = ArenaCommit {
            outcome: ArenaOutcome::CashOut { rounds_cleared: 5 },
            fee: 50,
            gold_reward: 100,
            xp_reward: 0,
            items: vec![],
            gold_from_overflow: 0,
            kills: 5,
            best_round: Some(5),
            tournament_wins_increment: 0,
            hp_set: Some(80),
            journal_msg: "Cash out".to_string(),
        };

        apply_arena_commit(&mut game, &commit);

        assert_eq!(game.character.best_tournament_round, 10);
    }

    // --- Cash-out with items that fit ---

    #[test]
    fn apply_commit_cash_out_items_fit_no_overflow() {
        let mut game = GameState::new(make_character(10, 0, 500));
        game.character.max_hp = 100;
        game.character.hp = 80;
        game.character.best_tournament_round = 0;

        let item = Item {
            name: "Iron Sword".to_string(),
            slot: ItemSlot::Weapon,
            power: 10,
            rarity: Rarity::Uncommon,
        };

        let commit = ArenaCommit {
            outcome: ArenaOutcome::CashOut { rounds_cleared: 3 },
            fee: 50,
            gold_reward: 100,
            xp_reward: 0,
            items: vec![item.clone()],
            gold_from_overflow: 0,
            kills: 3,
            best_round: Some(3),
            tournament_wins_increment: 0,
            hp_set: Some(80),
            journal_msg: "Cash out".to_string(),
        };

        apply_arena_commit(&mut game, &commit);

        assert_eq!(game.character.inventory.len(), 1);
        assert_eq!(game.character.inventory[0].name, "Iron Sword");
        assert_eq!(game.character.gold, 550);
    }

    // --- Compatibility: EventType::Tournament ---

    #[test]
    fn apply_commit_appends_tournament_journal_entry() {
        let mut game = GameState::new(make_character(10, 0, 500));
        game.character.max_hp = 100;

        let commit = ArenaCommit {
            outcome: ArenaOutcome::CashOut { rounds_cleared: 3 },
            fee: 50,
            gold_reward: 100,
            xp_reward: 0,
            items: vec![],
            gold_from_overflow: 0,
            kills: 3,
            best_round: Some(3),
            tournament_wins_increment: 0,
            hp_set: Some(80),
            journal_msg: "Arena cash-out in The Pit.".to_string(),
        };

        apply_arena_commit(&mut game, &commit);

        assert_eq!(game.journal.len(), 1);
        assert!(matches!(game.journal[0].event_type, crate::journal::EventType::Tournament));
        assert_eq!(game.journal[0].message, "Arena cash-out in The Pit.");
    }

    // --- Seeded compact combat log ---

    #[test]
    fn seeded_compact_combat_log_collapses() {
        use rand::SeedableRng;
        use rand::rngs::StdRng;
        use crate::character::Class;

        let mut rng = StdRng::seed_from_u64(42);
        let entry = ArenaEntrySnapshot {
            level: 1,
            xp_to_next: 25,
            hp: 100,
            max_hp: 100,
            attack_power: 1,
            defense: 20,
            prestige: 0,
            gold: 10,
        };
        let mut enemy = ArenaEnemy {
            name: "Test Bug".to_string(),
            hp: 55,
            max_hp: 55,
            attack: 7,
        };

        let result = run_compact_combat(&entry, entry.hp, &mut enemy, &Class::Warrior, &mut rng);

        assert!(
            result.total_turns >= 10,
            "Expected long fight, got {} turns",
            result.total_turns
        );
        assert_eq!(
            result.exchanges.len(),
            7,
            "Expected compacted log (7 lines), got {}",
            result.exchanges.len()
        );
        let has_summary = result.exchanges.iter().any(|ex| ex.plain.contains("more exchanges"));
        assert!(has_summary, "Expected summary line in compacted combat log");
    }
}

use crate::character::Item;
use crate::journal::{EventType, JournalEntry};
use crate::state::GameState;
use colored::*;
use rand::Rng;

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

const MAX_ROUNDS: u32 = 50;

#[derive(Debug)]
struct TournamentEnemy {
    name: String,
    hp: i32,
    max_hp: i32,
    attack: i32,
}

fn entry_fee(level: u32, total_prestiges: u32) -> u32 {
    150 + level * 20 + total_prestiges * 250
}

fn generate_enemy(round: u32, player_level: u32, rng: &mut rand::rngs::ThreadRng) -> TournamentEnemy {
    let name = ENEMY_NAMES[rng.gen_range(0..ENEMY_NAMES.len())].to_string();
    let hp = 30 + round as i32 * 12 + (player_level as i32 / 3);
    let attack = 10 + round as i32 * 4 + (player_level as i32 / 4);
    TournamentEnemy {
        name,
        hp,
        max_hp: hp,
        attack,
    }
}

fn round_gold(round: u32) -> u32 {
    round * 25 + 50
}

fn round_xp(round: u32) -> u32 {
    round * 20 + 30
}

fn loot_danger_for_round(round: u32) -> u32 {
    match round {
        1 => 1,
        2 => 2,
        3..=4 => 3,
        5..=7 => 4,
        8..=11 => 5,
        12..=16 => 6,
        17..=23 => 7,
        24..=32 => 8,
        _ => 9,
    }
}

fn roll_tournament_loot(round: u32) -> Item {
    crate::loot::roll_loot_scaled(loot_danger_for_round(round))
}

fn should_keep_tournament_loot(character: &crate::character::Character, item: &crate::character::Item) -> bool {
    const MAX_INVENTORY: usize = 20;
    if character.inventory.len() < MAX_INVENTORY {
        return true;
    }
    let weakest_droppable = character
        .inventory
        .iter()
        .filter(|i| i.rarity.is_droppable())
        .min_by_key(|i| i.power);
    match weakest_droppable {
        Some(weakest) => item.power > weakest.power,
        None => false,
    }
}

pub fn run_tournament(game: &mut GameState) {
    let mut rng = rand::thread_rng();
    let fee = entry_fee(game.character.level, game.character.total_prestiges);

    if game.character.gold < fee {
        eprintln!(
            "{} Not enough gold! Tournament entry costs {} gold, you have {}.",
            "⚠️".yellow(),
            format!("{}", fee).yellow().bold(),
            format!("{}", game.character.gold).yellow()
        );
        return;
    }

    game.character.gold -= fee;
    game.character.hp = game.character.max_hp;

    if let Err(e) = crate::state::save(game) {
        eprintln!("{} Failed to save: {}", "❌".bold(), e.red());
        return;
    }

    let mut round: u32 = 1;
    let mut rounds_cleared: u32 = 0;
    let mut total_gold: u32 = 0;
    let mut total_xp: u32 = 0;
    let mut baseline_win_awarded = false;
    let mut loot_names: Vec<String> = Vec::new();

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
        "🏟️  THE TERMINAL GAUNTLET".yellow().bold(),
        "║".yellow().bold()
    );
    eprintln!(
        "{} {} {}",
        "║".yellow().bold(),
        format!(
            "   Entry fee: {} gold  HP restored to full",
            fee
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
        let mut enemy = generate_enemy(round, game.character.level, &mut rng);

                let (_plain, colored) =
                    crate::messages::tournament_round_intro(&game.character.class, round, &enemy.name);
        eprintln!("{} {}", "⚔️".bold(), colored);
        eprintln!("{}", "─".repeat(40).dimmed());

        let mut combat_turn = 0;
        loop {
            let player_power = game.character.attack_power();
            let player_defense = game.character.defense();

            let hit_roll: i32 = rng.gen_range(1..=20);
            if hit_roll != 1 && hit_roll + player_power > 10 {
                let dmg = rng.gen_range((player_power / 2).max(1)..=player_power.max(1));
                enemy.hp -= dmg;
                let display_enemy_hp = enemy.hp.max(0);
                let (_plain, colored) = crate::messages::tournament_player_hit(
                    &game.character.class,
                    &enemy.name,
                    dmg,
                    display_enemy_hp,
                    enemy.max_hp,
                );
                eprintln!("   {}", colored);
            } else {
                let (_plain, colored) =
                    crate::messages::tournament_player_miss(&game.character.class, &enemy.name);
                eprintln!("   {}", colored);
            }

            let dodge_roll: i32 = rng.gen_range(1..=20);
            if dodge_roll > (8 + player_defense / 2) || dodge_roll == 20 {
                let dmg = (enemy.attack - player_defense).max(1);
                game.character.hp -= dmg;
                let display_hp = game.character.hp.max(0);

                let (_plain, colored) = crate::messages::tournament_enemy_hit(
                    &enemy.name,
                    dmg,
                    display_hp,
                    game.character.max_hp,
                    combat_turn,
                );
                eprintln!("   {}", colored);
            } else {
                let (_plain, colored) =
                    crate::messages::tournament_enemy_miss(&enemy.name, combat_turn);
                eprintln!("   {}", colored);
            }
            combat_turn += 1;

            if game.character.hp <= 0 {
                if game.permadeath {
                    crate::display::print_permadeath_eulogy(&game.character, &enemy.name);
                    let path = crate::state::save_path();
                    let _ = std::fs::remove_file(&path);
                    std::process::exit(0);
                }

                game.character.hp = (game.character.max_hp / 4).max(1);

                let (_plain, colored) =
                    crate::messages::tournament_ko(rounds_cleared, total_gold, total_xp);
                eprintln!("{}", "─".repeat(40).dimmed());
                eprintln!("{} {}", "💀".bold(), colored);

                let journal_msg = format!(
                    "Tournament KO after {} rounds. Earned {} gold, {} XP. Best round: {}",
                    rounds_cleared, total_gold, total_xp, game.character.best_tournament_round
                );
                game.add_journal(JournalEntry::new(EventType::Tournament, journal_msg));
                return;
            }

            if enemy.hp <= 0 {
                rounds_cleared = round;
                if round > game.character.best_tournament_round {
                    game.character.best_tournament_round = round;
                }
                game.character.kills += 1;

                let gold_reward = round_gold(round);
                let xp_reward = round_xp(round);
                game.character.gold = game.character.gold.saturating_add(gold_reward);
                total_gold = total_gold.saturating_add(gold_reward);

                let at_max_level = game.character.level >= crate::character::MAX_LEVEL;
                let actual_xp = if at_max_level { 0u32 } else { xp_reward };

                let leveled = game.character.gain_xp(xp_reward);
                if leveled {
                    crate::events::emit_level_up(game);
                }
                total_xp = total_xp.saturating_add(actual_xp);

                let loot = roll_tournament_loot(round);
                let loot_name = loot.name.clone();
                let loot_rarity = loot.rarity.clone();
                let loot_power = loot.power;
                let kept = if should_keep_tournament_loot(&game.character, &loot) {
                    crate::events::add_to_inventory_pub(game, loot)
                } else {
                    let msg = crate::events::full_inventory_message(&loot);
                    eprintln!("{} {}", "📦".bold(), msg.yellow().italic());
                    false
                };
                if kept {
                    loot_names.push(loot_name.clone());
                }

                let (_plain, colored) = if at_max_level && kept {
                    crate::messages::tournament_round_reward_max_level(round, gold_reward, &loot_name, &loot_rarity, loot_power)
                } else if at_max_level && !kept {
                    crate::messages::tournament_round_reward_max_level_no_loot(round, gold_reward)
                } else if kept {
                    crate::messages::tournament_round_reward(round, gold_reward, actual_xp, &loot_name, &loot_rarity, loot_power)
                } else {
                    crate::messages::tournament_round_reward_no_loot(round, gold_reward, actual_xp)
                };
                eprintln!("   {}", colored);

                if round == 5 && !baseline_win_awarded {
                    baseline_win_awarded = true;
                    let (_plain, colored) = crate::messages::tournament_baseline_win(round);
                    eprintln!("   {}", colored);
                }

                eprintln!();
                round += 1;

                if round > MAX_ROUNDS {
                    game.character.tournament_wins += 1;
                    let (_plain, colored) =
                        crate::messages::tournament_victory(rounds_cleared, total_gold, total_xp);
                    eprintln!("{}", "─".repeat(40).dimmed());
                    eprintln!("{} {}", "🏆".bold(), colored);
                    let journal_msg = format!(
                        "Tournament CHAMPION! Survived all {} rounds. Earned {} gold, {} XP. Best round: {}",
                        MAX_ROUNDS, total_gold, total_xp, game.character.best_tournament_round
                    );
                    game.add_journal(JournalEntry::new(EventType::Tournament, journal_msg));
                    return;
                }

                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_fee_formula() {
        assert_eq!(entry_fee(1, 0), 170);
        assert_eq!(entry_fee(10, 0), 350);
        assert_eq!(entry_fee(10, 2), 850);
    }

    #[test]
    fn enemy_scales_with_round_and_level() {
        let mut rng = rand::thread_rng();
        let e1 = generate_enemy(1, 1, &mut rng);
        let e5 = generate_enemy(5, 10, &mut rng);
        let e10 = generate_enemy(10, 20, &mut rng);
        assert!(e1.hp < e5.hp);
        assert!(e5.hp < e10.hp);
        assert!(e1.attack < e5.attack);
        assert!(e5.attack < e10.attack);
    }

    #[test]
    fn loot_danger_progression() {
        assert_eq!(loot_danger_for_round(1), 1);
        assert_eq!(loot_danger_for_round(2), 2);
        assert_eq!(loot_danger_for_round(5), 4);
        assert_eq!(loot_danger_for_round(15), 6);
        assert_eq!(loot_danger_for_round(30), 8);
        assert_eq!(loot_danger_for_round(50), 9);
    }

    #[test]
    fn round_rewards_scale() {
        assert!(round_gold(2) < round_gold(5));
        assert!(round_gold(5) < round_gold(10));
        assert!(round_xp(2) < round_xp(5));
        assert!(round_xp(5) < round_xp(10));
    }

    #[test]
    fn max_rounds_is_50() {
        assert_eq!(MAX_ROUNDS, 50);
    }
}

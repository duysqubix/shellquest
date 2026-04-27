use crate::display;
use crate::journal::{EventType, JournalEntry};
use crate::loot::roll_loot;
use crate::state::GameState;
use crate::zones::{zone_from_path, travel_message};
use colored::*;
use rand::Rng;

/// Scales base XP by zone danger level.
/// danger 1 = 1.0×, danger 2 = 1.25×, danger 3 = 1.5×, danger 4 = 1.75×, danger 5 = 2.0×
fn scaled_xp(base: u32, danger: u32) -> u32 {
    let multiplier = 1.0 + (danger.saturating_sub(1) as f32) * 0.25;
    ((base as f32) * multiplier).round() as u32
}

/// Returns 1.5 if the player's class has affinity with the given base command, else 1.0.
fn affinity_multiplier(class: &crate::character::Class, cmd: &str) -> f32 {
    use crate::character::Class;
    let affinities: &[&str] = match class {
        Class::Wizard      => &["python", "python3", "node", "ruby", "vim", "nvim", "emacs", "man", "tldr", "jupyter"],
        Class::Warrior     => &["cargo", "make", "cmake", "gcc", "g++", "ninja", "meson", "mvn", "gradle"],
        Class::Rogue       => &["grep", "rg", "ag", "ssh", "find", "fd", "ls", "eza", "locate"],
        Class::Ranger      => &["curl", "wget", "http", "docker", "kubectl", "ansible", "terraform", "helm"],
        Class::Necromancer => &["kill", "pkill", "killall", "rm", "del", "git", "shred"],
    };
    if affinities.iter().any(|&a| cmd == a || cmd.starts_with(&format!("{} ", a))) {
        1.5
    } else {
        1.0
    }
}

/// Apply both zone scaling and class affinity to a base XP amount.
fn final_xp(base: u32, danger: u32, class: &crate::character::Class, cmd: &str) -> u32 {
    let zone_scaled = scaled_xp(base, danger);
    (zone_scaled as f32 * affinity_multiplier(class, cmd)).round() as u32
}

pub fn tick(state: &mut GameState, command: &str, cwd: &str, exit_code: i32) {
    state.character.commands_run += 1;
    let mut rng = rand::thread_rng();

    if exit_code != 0 {
        if rng.gen_ratio(1, 4) {
            handle_trap(state, &mut rng);
        }
        return;
    }

    // Command-specific events
    let cmd_lower = command.to_lowercase();
    let cmd_base = cmd_lower.split_whitespace().next().unwrap_or("");
    let zone = zone_from_path(cwd);

    match cmd_base {
        "cd" => {
            if rng.gen_ratio(1, 3) {
                handle_travel(state, cwd);
            }
        }
        "git" => {
            if cmd_lower.contains("commit") {
                handle_craft(state, &mut rng, &zone, &cmd_lower);
            } else if cmd_lower.contains("push") {
                handle_quest(state, &mut rng, &zone, &cmd_lower);
            } else if rng.gen_ratio(1, 5) {
                handle_discovery(state, &mut rng, &zone, &cmd_lower);
            }
        }
        "cargo" | "make" | "npm" | "yarn" | "pnpm" => {
            if cmd_lower.contains("build") || cmd_lower.contains("compile") {
                handle_forge(state, &mut rng, &zone, &cmd_lower);
            } else if rng.gen_ratio(1, 5) {
                handle_discovery(state, &mut rng, &zone, &cmd_lower);
            }
        }
        "rm" | "del" => {
            if rng.gen_ratio(1, 3) {
                handle_angry_spirit(state, &mut rng, &zone, &cmd_lower);
            }
        }
        "cat" | "bat" | "less" | "more" => {
            if rng.gen_ratio(1, 10) {
                handle_familiar(state, &mut rng);
            }
        }
        "ls" | "find" | "fd" => {
            if rng.gen_ratio(1, 5) {
                handle_search_loot(state, &mut rng, cwd);
            }
        }
        "ssh" | "curl" | "wget" => {
            if rng.gen_ratio(1, 4) {
                handle_portal(state, &mut rng, &zone, &cmd_lower);
            }
        }
        "sudo" => {
            if rng.gen_ratio(1, 4) {
                handle_power_surge(state, &mut rng, &zone, &cmd_lower);
            }
        }
        "docker" | "podman" | "docker-compose" => {
            if cmd_lower.contains("build") {
                if rng.gen_ratio(1, 3) {
                    handle_container_forge(state, &mut rng, cwd);
                }
            } else if cmd_lower.contains("run") || cmd_lower.contains("exec") {
                if rng.gen_ratio(1, 3) {
                    handle_summon(state, &mut rng, "container golem");
                }
            } else if cmd_lower.contains("pull") {
                if rng.gen_ratio(1, 4) {
                    handle_docker_pull(state, &mut rng);
                }
            } else if cmd_lower.contains("stop") || cmd_lower.contains("kill") || cmd_lower.contains("rm") {
                if rng.gen_ratio(1, 3) {
                    handle_docker_banish(state, &mut rng);
                }
            } else if cmd_lower.contains("compose") {
                if rng.gen_ratio(1, 3) {
                    handle_docker_orchestra(state, &mut rng, &zone, &cmd_lower);
                }
            } else if rng.gen_ratio(1, 4) {
                handle_summon(state, &mut rng, "container golem");
            }
        }
        "python" | "python3" | "node" | "ruby" | "lua" => {
            if rng.gen_ratio(1, 5) {
                handle_incantation(state, &mut rng, &zone, &cmd_lower);
            }
        }
        "pip" | "pip3" | "gem" | "composer" => {
            if rng.gen_ratio(1, 4) {
                handle_alchemy(state, &mut rng);
            }
        }
        "vim" | "nvim" | "emacs" | "nano" | "code" | "hx" => {
            if rng.gen_ratio(1, 8) {
                handle_meditation(state, &mut rng, &zone, &cmd_lower);
            }
        }
        "grep" | "rg" | "ag" | "ack" => {
            if rng.gen_ratio(1, 4) {
                handle_scrying(state, &mut rng, cwd);
            }
        }
        "test" | "pytest" | "jest" | "vitest" | "mocha" => {
            if rng.gen_ratio(1, 4) {
                handle_trial(state, &mut rng);
            }
        }
        "cp" | "mv" | "rsync" => {
            if rng.gen_ratio(1, 5) {
                handle_telekinesis(state, &mut rng);
            }
        }
        "chmod" | "chown" | "chgrp" => {
            if rng.gen_ratio(1, 4) {
                handle_enchant(state, &mut rng);
            }
        }
        "top" | "htop" | "btm" | "ps" => {
            if rng.gen_ratio(1, 5) {
                handle_omniscience(state, &mut rng);
            }
        }
        "kill" | "killall" | "pkill" => {
            if rng.gen_ratio(1, 3) {
                handle_banish(state, &mut rng, &zone, &cmd_lower);
            }
        }
        "tar" | "zip" | "unzip" | "gzip" => {
            if rng.gen_ratio(1, 4) {
                handle_treasure_chest(state, &mut rng, cwd);
            }
        }
        "echo" | "printf" => {
            if rng.gen_ratio(1, 10) {
                handle_echo_spell(state, &mut rng);
            }
        }
        "man" | "tldr" | "help" => {
            if rng.gen_ratio(1, 4) {
                handle_ancient_tome(state, &mut rng, &zone, &cmd_lower);
            }
        }
        _ => {
            // Generic random encounter ~15% of the time
            if rng.gen_ratio(3, 20) {
                handle_random_encounter(state, &mut rng, &zone, &cmd_lower);
            }
        }
    }

    // Passive healing over time
    if state.character.hp < state.character.max_hp && rng.gen_ratio(1, passive_heal_denominator()) {
        state.character.heal(1);
    }

    // Boss tick (runs every tick if a boss is active)
    crate::boss::tick_boss(state);

    // Passive boss spawn check (very rare world event)
    crate::boss::maybe_spawn(state);
}

fn handle_trap(state: &mut GameState, rng: &mut impl Rng) {
    let damage = rng.gen_range(1..=3);
    let gold_before = state.character.gold;
    let died = state.character.take_damage(damage);
    if died {
        if state.permadeath {
            crate::display::print_permadeath_eulogy(&state.character, "a trap");
            let path = crate::state::save_path();
            let _ = std::fs::remove_file(&path);
            std::process::exit(0);
        }
        state.character.die();
        let gold_loss = gold_before * 15 / 100;
        let (plain, colored) =
            crate::messages::death_normal(&state.character.class, "a trap", gold_loss);
        display::print_combat_lose(&colored, true);
        state.add_journal(JournalEntry::new(EventType::Death, plain));
        return;
    }
    let (plain, colored) = crate::messages::trap(
        &state.character.class,
        damage,
        state.character.hp,
        state.character.max_hp,
    );
    display::print_trap(&colored);
    state.add_journal(JournalEntry::new(EventType::Combat, plain));
}

fn handle_travel(state: &mut GameState, cwd: &str) {
    let zone = zone_from_path(cwd);
    let plain = travel_message(&zone);
    let colored = format!("You enter {}... {}",
        display::color_zone(zone.name, &zone),
        zone.description.italic().dimmed());
    display::print_travel(&colored, &zone);
    state.add_journal(JournalEntry::new(EventType::Travel, plain));
}

fn handle_craft(state: &mut GameState, rng: &mut impl Rng, zone: &crate::zones::Zone, cmd: &str) {
    let base_xp = rng.gen_range(15..=35);
    let xp = final_xp(base_xp, zone.danger_level, &state.character.class, cmd);
    let leveled = state.character.gain_xp(xp);
    let (plain, colored) = crate::messages::craft(&state.character.class, xp);
    display::print_craft(&colored);
    state.add_journal(JournalEntry::new(EventType::Craft, plain));
    check_level_up(state, leveled);
}

pub(crate) fn emit_level_up(state: &mut GameState) {
    let (plain, colored) = crate::messages::level_up(
        &state.character.class,
        state.character.level,
        &state.character.title,
    );
    display::print_level_up(&colored);
    state.add_journal(crate::journal::JournalEntry::new(crate::journal::EventType::LevelUp, plain));
}

fn check_level_up(state: &mut GameState, leveled: bool) {
    if leveled { emit_level_up(state); }
}

fn handle_quest(state: &mut GameState, rng: &mut impl Rng, zone: &crate::zones::Zone, cmd: &str) {
    let base_xp = rng.gen_range(15..=35);
    let xp = final_xp(base_xp, zone.danger_level, &state.character.class, cmd);
    let gold = rng.gen_range(5..=20);
    let leveled = state.character.gain_xp(xp);
    state.character.gold += gold;
    let (plain, colored) = crate::messages::quest(&state.character.class, xp, gold);
    display::print_quest(&colored);
    state.add_journal(JournalEntry::new(EventType::Quest, plain));
    check_level_up(state, leveled);
}

fn handle_discovery(state: &mut GameState, rng: &mut impl Rng, zone: &crate::zones::Zone, cmd: &str) {
    let base_xp = rng.gen_range(8..=20);
    let xp = final_xp(base_xp, zone.danger_level, &state.character.class, cmd);
    let leveled = state.character.gain_xp(xp);
    let discoveries = [
        "an ancient code comment from a forgotten developer",
        "a hidden TODO that grants wisdom",
        "a deprecated scroll of knowledge",
        "a mysterious FIXME glowing with arcane energy",
        "a secret .env file buried in the ruins",
    ];
    let detail = discoveries[rng.gen_range(0..discoveries.len())];
    let (plain, colored) = crate::messages::discovery(&state.character.class, detail, xp);
    display::print_discovery(&colored);
    state.add_journal(JournalEntry::new(EventType::Discovery, plain));
    check_level_up(state, leveled);
}

fn handle_forge(state: &mut GameState, rng: &mut impl Rng, zone: &crate::zones::Zone, cmd: &str) {
    if rng.gen_ratio(1, 3) {
        let item = roll_loot(zone.danger_level);
        let xp = 0;
        let (plain, colored) = crate::messages::forge_loot(&state.character.class, &item.name, item.power, xp);
        display::print_loot(&colored, &item.rarity);
        state.add_journal(JournalEntry::new(EventType::Craft, plain));
        add_to_inventory(state, item);
    } else {
        let base_xp = rng.gen_range(8..=20);
        let xp = final_xp(base_xp, zone.danger_level, &state.character.class, cmd);
        let leveled = state.character.gain_xp(xp);
        let (plain, colored) = crate::messages::forge_xp(&state.character.class, xp);
        display::print_craft(&colored);
        state.add_journal(JournalEntry::new(EventType::Craft, plain));
        check_level_up(state, leveled);
    }
}

fn handle_angry_spirit(state: &mut GameState, rng: &mut impl Rng, zone: &crate::zones::Zone, cmd: &str) {
    let (name, base_atk, base_xp) = random_monster(rng);
    let scale = encounter_scale_for_danger(zone.danger_level);
    let atk = (base_atk as f32 * scale).round().max(1.0) as i32;
    let xp = (base_xp as f32 * scale).max(5.0) as u32;
    let profile = if rng.gen_ratio(1, 8) {
        apply_elite_pressure(&name, atk, xp, zone.danger_level)
    } else {
        EncounterProfile { name, attack: atk, xp, elite: false }
    };

    if profile.elite {
        combat_elite(state, rng, zone, cmd, &profile.name, profile.attack, profile.xp);
    } else {
        combat(state, rng, zone, cmd, &profile.name, profile.attack, profile.xp);
    }
}

fn handle_familiar(state: &mut GameState, rng: &mut impl Rng) {
    let familiars = ["curious cat", "friendly daemon", "pixel sprite", "tame penguin", "binary beetle"];
    let heal = rng.gen_range(2..=4);
    state.character.heal(heal);
    let creature = familiars[rng.gen_range(0..familiars.len())];
    let (plain, colored) = crate::messages::familiar(
        &state.character.class,
        creature,
        heal,
        state.character.hp,
        state.character.max_hp,
    );
    display::print_familiar(&colored);
    state.add_journal(JournalEntry::new(EventType::Discovery, plain));
}

fn handle_search_loot(state: &mut GameState, rng: &mut impl Rng, cwd: &str) {
    let zone = zone_from_path(cwd);
    let gold = rng.gen_range(1..=5) * zone.danger_level;
    state.character.gold += gold;
    let plain = format!("You search the area and find {} gold coins!", gold);
    let colored = format!("You {} the area and find {} {}!",
        "search".cyan(), format!("{}", gold).yellow().bold(), "gold coins".yellow());
    display::print_gold(&colored);
    state.add_journal(JournalEntry::new(EventType::Loot, plain));
}

fn handle_portal(state: &mut GameState, rng: &mut impl Rng, zone: &crate::zones::Zone, cmd: &str) {
    let base_xp = rng.gen_range(10..=20);
    let xp = final_xp(base_xp, zone.danger_level, &state.character.class, cmd);
    let leveled = state.character.gain_xp(xp);
    let (plain, colored) = crate::messages::portal(&state.character.class, xp);
    display::print_portal(&colored);
    state.add_journal(JournalEntry::new(EventType::Travel, plain));
    check_level_up(state, leveled);
}

fn handle_power_surge(state: &mut GameState, rng: &mut impl Rng, zone: &crate::zones::Zone, cmd: &str) {
    let base_xp = rng.gen_range(15..=30);
    let xp = final_xp(base_xp, zone.danger_level, &state.character.class, cmd);
    let leveled = state.character.gain_xp(xp);
    let (plain, colored) = crate::messages::power_surge(&state.character.class, xp);
    display::print_power(&colored);
    state.add_journal(JournalEntry::new(EventType::Discovery, plain));
    check_level_up(state, leveled);
}

fn handle_summon(state: &mut GameState, rng: &mut impl Rng, creature: &str) {
    let xp = rng.gen_range(10..=20);
    let leveled = state.character.gain_xp(xp);
    let msg = format!("You summon a {}! It fights by your side briefly. +{} XP", creature, xp);
    display::print_portal(&msg);
    state.add_journal(JournalEntry::new(EventType::Discovery, msg));
    check_level_up(state, leveled);
}

fn handle_incantation(state: &mut GameState, rng: &mut impl Rng, zone: &crate::zones::Zone, cmd: &str) {
    let base_xp = rng.gen_range(8..=18);
    let xp = final_xp(base_xp, zone.danger_level, &state.character.class, cmd);
    let leveled = state.character.gain_xp(xp);
    let lang = cmd.split_whitespace().next().unwrap_or("script");
    let (plain, colored) = crate::messages::incantation(&state.character.class, lang, xp);
    display::print_discovery(&colored);
    state.add_journal(JournalEntry::new(EventType::Discovery, plain));
    check_level_up(state, leveled);
}

fn handle_alchemy(state: &mut GameState, rng: &mut impl Rng) {
    if rng.gen_ratio(1, 3) {
        let item = roll_loot(2);
        let msg = format!("Your package install transmutes into: {} (+{} {}) [{}]", item.name, item.power, item.slot, item.rarity);
        display::print_loot(&msg, &item.rarity);
        state.add_journal(JournalEntry::new(EventType::Loot, msg));
        add_to_inventory(state, item);
    } else {
        let xp = rng.gen_range(5..=15);
        let leveled = state.character.gain_xp(xp);
        let msg = format!("The alchemist's cauldron bubbles! Dependencies resolve into power! +{} XP", xp);
        display::print_craft(&msg);
        state.add_journal(JournalEntry::new(EventType::Craft, msg));
        check_level_up(state, leveled);
    }
}

fn handle_meditation(state: &mut GameState, rng: &mut impl Rng, zone: &crate::zones::Zone, cmd: &str) {
    let heal = rng.gen_range(3..=7);
    let base_xp = rng.gen_range(5..=10);
    state.character.heal(heal);
    let xp = final_xp(base_xp, zone.danger_level, &state.character.class, cmd);
    let leveled = state.character.gain_xp(xp);
    let editor = cmd.split_whitespace().next().unwrap_or("editor");
    let (plain, colored) = crate::messages::meditation(
        &state.character.class,
        editor,
        heal,
        xp,
        state.character.hp,
        state.character.max_hp,
    );
    display::print_familiar(&colored);
    state.add_journal(JournalEntry::new(EventType::Discovery, plain));
    check_level_up(state, leveled);
}

fn handle_scrying(state: &mut GameState, rng: &mut impl Rng, cwd: &str) {
    let zone = zone_from_path(cwd);
    if rng.gen_ratio(1, 3) {
        let item = roll_loot(zone.danger_level);
        let msg = format!("Your search reveals a hidden treasure: {} (+{} {}) [{}]", item.name, item.power, item.slot, item.rarity);
        display::print_loot(&msg, &item.rarity);
        state.add_journal(JournalEntry::new(EventType::Loot, msg));
        add_to_inventory(state, item);
    } else {
        let xp = rng.gen_range(8..=16);
        let leveled = state.character.gain_xp(xp);
        let msg = format!("Your scrying reveals hidden patterns in the codebase! +{} XP", xp);
        display::print_discovery(&msg);
        state.add_journal(JournalEntry::new(EventType::Discovery, msg));
        check_level_up(state, leveled);
    }
}

fn handle_trial(state: &mut GameState, rng: &mut impl Rng) {
    let xp = rng.gen_range(12..=25);
    let leveled = state.character.gain_xp(xp);
    let msgs = [
        "You enter the Proving Grounds! All assertions hold true!",
        "The trial by test completes! Your code stands unbroken!",
        "The test oracle nods in approval! Green across the board!",
    ];
    let msg = format!("{} +{} XP", msgs[rng.gen_range(0..msgs.len())], xp);
    display::print_quest(&msg);
    state.add_journal(JournalEntry::new(EventType::Quest, msg));
    check_level_up(state, leveled);
}

fn handle_telekinesis(state: &mut GameState, rng: &mut impl Rng) {
    let xp = rng.gen_range(5..=12);
    let leveled = state.character.gain_xp(xp);
    let msg = "You move files with the power of your mind! Bytes rearrange at your command!";
    let full_msg = format!("{} +{} XP", msg, xp);
    display::print_discovery(&full_msg);
    state.add_journal(JournalEntry::new(EventType::Discovery, full_msg));
    check_level_up(state, leveled);
}

fn handle_enchant(state: &mut GameState, rng: &mut impl Rng) {
    let xp = rng.gen_range(10..=20);
    let leveled = state.character.gain_xp(xp);
    let msgs = [
        "You enchant the file with new permissions! It glows with arcane authority!",
        "You reshape the ownership runes! The filesystem bows to your will!",
    ];
    let msg = format!("{} +{} XP", msgs[rng.gen_range(0..msgs.len())], xp);
    display::print_power(&msg);
    state.add_journal(JournalEntry::new(EventType::Discovery, msg));
    check_level_up(state, leveled);
}

fn handle_omniscience(state: &mut GameState, rng: &mut impl Rng) {
    let xp = rng.gen_range(5..=10);
    let leveled = state.character.gain_xp(xp);
    let msg = format!("You peer into the process table... all running spirits are revealed to you! +{} XP", xp);
    display::print_discovery(&msg);
    state.add_journal(JournalEntry::new(EventType::Discovery, msg));
    check_level_up(state, leveled);
}

fn handle_banish(state: &mut GameState, rng: &mut impl Rng, zone: &crate::zones::Zone, cmd: &str) {
    let base_xp = rng.gen_range(15..=25);
    let xp = final_xp(base_xp, zone.danger_level, &state.character.class, cmd);
    let gold = rng.gen_range(3..=10);
    let leveled = state.character.gain_xp(xp);
    state.character.gold += gold;
    state.character.kills += 1;
    let targets = [
        "rogue process",
        "runaway daemon",
        "zombie worker",
    ];
    let target = targets[rng.gen_range(0..targets.len())];
    let (plain, colored) = crate::messages::banish(&state.character.class, target, xp, gold);
    display::print_combat_win(&colored);
    state.add_journal(JournalEntry::new(EventType::Combat, plain));
    check_level_up(state, leveled);
}

fn handle_treasure_chest(state: &mut GameState, _rng: &mut impl Rng, cwd: &str) {
    let zone = zone_from_path(cwd);
    let item = roll_loot(zone.danger_level + 1);
    let msg = format!("You crack open an archive! Inside you find: {} (+{} {}) [{}]", item.name, item.power, item.slot, item.rarity);
    display::print_loot(&msg, &item.rarity);
    state.add_journal(JournalEntry::new(EventType::Loot, msg));
    add_to_inventory(state, item);
}

fn handle_echo_spell(state: &mut GameState, rng: &mut impl Rng) {
    let heal = rng.gen_range(1..=3);
    state.character.heal(heal);
    let msg = format!("Your words echo through the terminal void... the resonance heals you! +{} HP", heal);
    display::print_familiar(&msg);
    state.add_journal(JournalEntry::new(EventType::Discovery, msg));
}

fn handle_ancient_tome(state: &mut GameState, rng: &mut impl Rng, zone: &crate::zones::Zone, cmd: &str) {
    let base_xp = rng.gen_range(10..=22);
    let xp = final_xp(base_xp, zone.danger_level, &state.character.class, cmd);
    let leveled = state.character.gain_xp(xp);
    let subject = cmd.split_whitespace().next().unwrap_or("manual");
    let (plain, colored) = crate::messages::ancient_tome(&state.character.class, subject, xp);
    display::print_discovery(&colored);
    state.add_journal(JournalEntry::new(EventType::Discovery, plain));
    check_level_up(state, leveled);
}

fn handle_container_forge(state: &mut GameState, rng: &mut impl Rng, cwd: &str) {
    let zone = zone_from_path(cwd);
    if rng.gen_ratio(1, 2) {
        let item = roll_loot(zone.danger_level + 1);
        let msg = format!("The container forge blazes! Layers fuse into: {} (+{} {}) [{}]", item.name, item.power, item.slot, item.rarity);
        display::print_loot(&msg, &item.rarity);
        state.add_journal(JournalEntry::new(EventType::Craft, msg));
        add_to_inventory(state, item);
    } else {
        let xp = rng.gen_range(12..=25);
        let leveled = state.character.gain_xp(xp);
        let msg = format!("The image builds layer by layer! Each instruction tempers your resolve! +{} XP", xp);
        display::print_craft(&msg);
        state.add_journal(JournalEntry::new(EventType::Craft, msg));
        check_level_up(state, leveled);
    }
}

fn handle_docker_pull(state: &mut GameState, rng: &mut impl Rng) {
    let xp = rng.gen_range(8..=18);
    let leveled = state.character.gain_xp(xp);
    let msgs = [
        "You pull an image from the Container Registry of the Cloud Realm!",
        "Layers materialize from the ether! The image manifests before you!",
        "The registry yields its treasures! A fresh image appears!",
    ];
    let msg = format!("{} +{} XP", msgs[rng.gen_range(0..msgs.len())], xp);
    display::print_portal(&msg);
    state.add_journal(JournalEntry::new(EventType::Discovery, msg));
    check_level_up(state, leveled);
}

fn handle_docker_banish(state: &mut GameState, rng: &mut impl Rng) {
    let xp = rng.gen_range(10..=20);
    let gold = rng.gen_range(2..=8);
    let leveled = state.character.gain_xp(xp);
    state.character.gold += gold;
    state.character.kills += 1;
    let msgs = [
        "You banish the container to the void! Its resources return to the host!",
        "SIGTERM! The container dissolves into freed memory!",
        "You prune the fallen container! Its ephemeral storage scatters!",
    ];
    let msg = format!("{} +{} XP, +{} gold", msgs[rng.gen_range(0..msgs.len())], xp, gold);
    display::print_combat_win(&msg);
    state.add_journal(JournalEntry::new(EventType::Combat, msg));
    check_level_up(state, leveled);
}

fn handle_docker_orchestra(state: &mut GameState, rng: &mut impl Rng, zone: &crate::zones::Zone, cmd: &str) {
    let base_xp = rng.gen_range(15..=30);
    let xp = final_xp(base_xp, zone.danger_level, &state.character.class, cmd);
    let gold = rng.gen_range(5..=15);
    let leveled = state.character.gain_xp(xp);
    state.character.gold += gold;
    let (plain, colored) = crate::messages::docker_orchestra(&state.character.class, xp, gold);
    display::print_quest(&colored);
    state.add_journal(JournalEntry::new(EventType::Quest, plain));
    check_level_up(state, leveled);
}

fn handle_random_encounter(state: &mut GameState, rng: &mut impl Rng, zone: &crate::zones::Zone, cmd: &str) {
    let roll: u32 = rng.gen_range(1..=100);

    match roll {
        1..=40 => {
            // Combat encounter
            let (name, base_atk, base_xp) = random_monster_for_zone(rng, zone);
            let profile = if rng.gen_ratio(1, 8) {
                apply_elite_pressure(&name, base_atk, base_xp, zone.danger_level)
            } else {
                EncounterProfile { name, attack: base_atk, xp: base_xp, elite: false }
            };

            if profile.elite {
                combat_elite(state, rng, zone, cmd, &profile.name, profile.attack, profile.xp);
            } else {
                combat(state, rng, zone, cmd, &profile.name, profile.attack, profile.xp);
            }
        }
        41..=65 => {
            // Find loot
            let item = roll_loot(zone.danger_level);
            let msg = format!("You found: {} (+{} {}) [{}]", item.name, item.power, item.slot, item.rarity);
            display::print_loot(&msg, &item.rarity);
            state.add_journal(JournalEntry::new(EventType::Loot, msg));
            add_to_inventory(state, item);
        }
        66..=80 => {
            // Find gold
            let gold = rng.gen_range(1..=8) * zone.danger_level;
            state.character.gold += gold;
            let msg = format!("You found {} gold coins hidden in the path!", gold);
            display::print_gold(&msg);
            state.add_journal(JournalEntry::new(EventType::Loot, msg));
        }
        81..=85 => {
            // XP discovery
            let xp = rng.gen_range(5..=15);
            let leveled = state.character.gain_xp(xp);
            let msg = format!("You gain insight from your surroundings. +{} XP", xp);
            display::print_discovery(&msg);
            state.add_journal(JournalEntry::new(EventType::Discovery, msg));
            check_level_up(state, leveled);
        }
        86..=90 => {
            // Heal
            let heal = rng.gen_range(2..=6);
            state.character.heal(heal);
            let msg = format!("You find a quiet spot to rest. +{} HP. HP: {}/{}", heal, state.character.hp, state.character.max_hp);
            display::print_familiar(&msg);
            state.add_journal(JournalEntry::new(EventType::Discovery, msg));
        }
        _ => {
            // XP discovery
            let xp = rng.gen_range(5..=15);
            let leveled = state.character.gain_xp(xp);
            let msg = format!("You gain insight from your surroundings. +{} XP", xp);
            display::print_discovery(&msg);
            state.add_journal(JournalEntry::new(EventType::Discovery, msg));
            check_level_up(state, leveled);
        }
    }
}

// Used by Task 2 to replace the hardcoded passive-heal gen_ratio gate.
fn passive_heal_denominator() -> u32 {
    10
}

// Used by Task 3 to scale monster attack by zone danger.
fn encounter_scale_for_danger(danger: u32) -> f32 {
    match danger {
        1 => 0.9,
        2 => 1.1,
        3 => 1.4,
        4 => 1.8,
        _ => 2.2,
    }
}

// Used by Tasks 3 and 4 to represent normal and elite encounter profiles.
struct EncounterProfile {
    name: String,
    attack: i32,
    xp: u32,
    elite: bool,
}

// Used by Task 4 to create elite encounters.
fn apply_elite_pressure(name: &str, base_attack: i32, base_xp: u32, danger: u32) -> EncounterProfile {
    let attack_multiplier = 1.6 * (1.0 + (danger.saturating_sub(1) as f32) * 0.15);

    EncounterProfile {
        name: format!("Enraged {}", name),
        attack: ((base_attack as f32) * attack_multiplier).round() as i32,
        xp: ((base_xp as f32) * 2.0).round() as u32,
        elite: true,
    }
}

fn random_monster(rng: &mut impl Rng) -> (String, i32, u32) {
    let monsters = [
        ("Segfault Specter", 8, 15),
        ("Null Pointer Wraith", 6, 10),
        ("Off-by-One Ogre", 10, 20),
        ("Race Condition Rat", 5, 8),
        ("Deadlock Demon", 12, 25),
        ("Memory Leak Slime", 7, 12),
        ("Buffer Overflow Beast", 14, 30),
        ("Syntax Error Snake", 4, 6),
        ("Infinite Loop Imp", 6, 10),
        ("Dependency Hell Hound", 11, 22),
    ];
    let m = monsters[rng.gen_range(0..monsters.len())];
    (m.0.to_string(), m.1, m.2)
}

fn random_monster_for_zone(rng: &mut impl Rng, zone: &crate::zones::Zone) -> (String, i32, u32) {
    let (name, base_atk, base_xp) = random_monster(rng);
    let scale = encounter_scale_for_danger(zone.danger_level);
    let atk = (base_atk as f32 * scale).round().max(1.0) as i32;
    let xp = (base_xp as f32 * scale).max(5.0) as u32;
    (name, atk, xp)
}

fn combat(
    state: &mut GameState,
    rng: &mut impl Rng,
    zone: &crate::zones::Zone,
    cmd: &str,
    monster_name: &str,
    monster_atk: i32,
    xp_reward: u32,
) {
    let player_power = state.character.attack_power();
    let player_defense = state.character.defense();
    let hit_roll: i32 = rng.gen_range(1..=20);
    let dodge_roll: i32 = rng.gen_range(1..=20);
    let final_reward = final_xp(xp_reward, zone.danger_level, &state.character.class, cmd);

    let player_hits = hit_roll + player_power > 10;
    let monster_hits = dodge_roll > (8 + player_defense / 2);

    if player_hits && !monster_hits {
        state.character.kills += 1;
        let leveled = state.character.gain_xp(final_reward);
        let (plain, colored) = crate::messages::combat_win(&state.character.class, monster_name, final_reward);
        display::print_combat_win(&colored);
        state.add_journal(JournalEntry::new(EventType::Combat, plain));
        check_level_up(state, leveled);
    } else if player_hits && monster_hits {
        let damage = (monster_atk - player_defense / 3).max(1);
        let gold_before = state.character.gold;
        let died = state.character.take_damage(damage);
        if !died {
            state.character.kills += 1;
            let leveled = state.character.gain_xp(final_reward);
            let (plain, colored) = crate::messages::combat_tough(&state.character.class, monster_name, damage, final_reward);
            display::print_combat_tough(&colored, false);
            state.add_journal(JournalEntry::new(EventType::Combat, plain));
            check_level_up(state, leveled);
        } else {
            if state.permadeath {
                crate::display::print_permadeath_eulogy(&state.character, monster_name);
                let path = crate::state::save_path();
                let _ = std::fs::remove_file(&path);
                std::process::exit(0);
            } else {
                state.character.die();
                let gold_loss = gold_before * 15 / 100;
                let (plain, colored) = crate::messages::death_normal(
                    &state.character.class,
                    monster_name,
                    gold_loss,
                );
                display::print_combat_lose(&colored, true);
                state.add_journal(JournalEntry::new(EventType::Death, plain));
            }
        }
    } else if !player_hits && monster_hits {
        let damage = (monster_atk - player_defense / 4).max(1);
        let gold_before = state.character.gold;
        let died = state.character.take_damage(damage);
        if died {
            if state.permadeath {
                crate::display::print_permadeath_eulogy(&state.character, monster_name);
                let path = crate::state::save_path();
                let _ = std::fs::remove_file(&path);
                std::process::exit(0);
            } else {
                state.character.die();
                let gold_loss = gold_before * 15 / 100;
                let (plain, colored) = crate::messages::death_normal(
                    &state.character.class,
                    monster_name,
                    gold_loss,
                );
                display::print_combat_lose(&colored, true);
                state.add_journal(JournalEntry::new(EventType::Death, plain));
            }
        } else {
            let (plain, colored) = crate::messages::combat_lose(&state.character.class, monster_name, damage);
            display::print_combat_lose(&colored, false);
            state.add_journal(JournalEntry::new(EventType::Combat, plain));
        }
    } else {
        let (plain, colored) = crate::messages::combat_draw(&state.character.class, monster_name);
        display::print_combat_draw(&colored);
        state.add_journal(JournalEntry::new(EventType::Combat, plain));
    }
}

fn combat_elite(
    state: &mut GameState,
    rng: &mut impl Rng,
    zone: &crate::zones::Zone,
    cmd: &str,
    monster_name: &str,
    monster_atk: i32,
    xp_reward: u32,
) {
    let player_power = state.character.attack_power();
    let player_defense = state.character.defense();
    let hit_roll: i32 = rng.gen_range(1..=20);
    let dodge_roll: i32 = rng.gen_range(1..=20);
    let final_reward = final_xp(xp_reward, zone.danger_level, &state.character.class, cmd);

    let player_hits = hit_roll + player_power > 10;
    let monster_hits = dodge_roll > (8 + player_defense / 2);

    if player_hits && !monster_hits {
        state.character.kills += 1;
        let leveled = state.character.gain_xp(final_reward);
        let (plain, colored) = crate::messages::combat_elite_win(&state.character.class, monster_name, final_reward);
        display::print_combat_win(&colored);
        state.add_journal(JournalEntry::new(EventType::Combat, plain));
        check_level_up(state, leveled);
    } else if player_hits && monster_hits {
        let damage = (monster_atk - player_defense / 3).max(1);
        let gold_before = state.character.gold;
        let died = state.character.take_damage(damage);
        if !died {
            state.character.kills += 1;
            let leveled = state.character.gain_xp(final_reward);
            let (plain, colored) = crate::messages::combat_elite_tough(&state.character.class, monster_name, damage, final_reward);
            display::print_combat_tough(&colored, false);
            state.add_journal(JournalEntry::new(EventType::Combat, plain));
            check_level_up(state, leveled);
        } else if state.permadeath {
            crate::display::print_permadeath_eulogy(&state.character, monster_name);
            let path = crate::state::save_path();
            let _ = std::fs::remove_file(&path);
            std::process::exit(0);
        } else {
            state.character.die();
            let gold_loss = gold_before * 15 / 100;
            let (plain, colored) = crate::messages::death_normal(
                &state.character.class,
                monster_name,
                gold_loss,
            );
            display::print_combat_lose(&colored, true);
            state.add_journal(JournalEntry::new(EventType::Death, plain));
        }
    } else if !player_hits && monster_hits {
        let damage = (monster_atk - player_defense / 4).max(1);
        let gold_before = state.character.gold;
        let died = state.character.take_damage(damage);
        if died {
            if state.permadeath {
                crate::display::print_permadeath_eulogy(&state.character, monster_name);
                let path = crate::state::save_path();
                let _ = std::fs::remove_file(&path);
                std::process::exit(0);
            } else {
                state.character.die();
                let gold_loss = gold_before * 15 / 100;
                let (plain, colored) = crate::messages::death_normal(
                    &state.character.class,
                    monster_name,
                    gold_loss,
                );
                display::print_combat_lose(&colored, true);
                state.add_journal(JournalEntry::new(EventType::Death, plain));
            }
        } else {
            let (plain, colored) = crate::messages::combat_elite_lose(&state.character.class, monster_name, damage);
            display::print_combat_lose(&colored, false);
            state.add_journal(JournalEntry::new(EventType::Combat, plain));
        }
    } else {
        let (plain, colored) = crate::messages::combat_draw(&state.character.class, monster_name);
        display::print_combat_draw(&colored);
        state.add_journal(JournalEntry::new(EventType::Combat, plain));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::{Character, Class, Item, ItemSlot, Race, Rarity};
    use crate::state::GameState;

    fn make_state() -> GameState {
        GameState::new(Character::new("Test".to_string(), Class::Warrior, Race::Human))
    }

    fn make_item(name: &str, slot: ItemSlot, power: i32, rarity: Rarity) -> Item {
        Item { name: name.to_string(), slot, power, rarity }
    }

    #[test]
    fn add_to_inventory_adds_item_when_space_available() {
        let mut state = make_state();
        add_to_inventory(&mut state, make_item("Sword", ItemSlot::Weapon, 5, Rarity::Common));
        assert_eq!(state.character.inventory.len(), 1);
        assert_eq!(state.character.inventory[0].name, "Sword");
    }

    #[test]
    fn add_to_inventory_drops_weakest_droppable_when_full() {
        let mut state = make_state();
        for i in 0..20 {
            state.character.inventory.push(make_item(&format!("Common {}", i), ItemSlot::Weapon, i as i32 + 1, Rarity::Common));
        }
        add_to_inventory(&mut state, make_item("New Sword", ItemSlot::Weapon, 99, Rarity::Rare));
        assert_eq!(state.character.inventory.len(), 20);
        assert!(state.character.inventory.iter().any(|i| i.name == "New Sword"));
        assert!(!state.character.inventory.iter().any(|i| i.name == "Common 0"));
    }

    #[test]
    fn add_to_inventory_does_not_drop_epics_when_full() {
        let mut state = make_state();
        for i in 0..20 {
            state.character.inventory.push(make_item(&format!("Epic {}", i), ItemSlot::Weapon, i as i32 + 1, Rarity::Epic));
        }
        add_to_inventory(&mut state, make_item("New Sword", ItemSlot::Weapon, 5, Rarity::Common));
        assert_eq!(state.character.inventory.len(), 20);
        assert!(!state.character.inventory.iter().any(|i| i.name == "New Sword"));
        assert_eq!(state.character.inventory.iter().filter(|i| matches!(i.rarity, Rarity::Epic)).count(), 20);
    }

    #[test]
    fn add_to_inventory_does_not_drop_legendaries_when_full() {
        let mut state = make_state();
        for i in 0..20 {
            state.character.inventory.push(make_item(&format!("Legendary {}", i), ItemSlot::Weapon, i as i32 + 1, Rarity::Legendary));
        }
        add_to_inventory(&mut state, make_item("Common Sword", ItemSlot::Weapon, 5, Rarity::Common));
        assert_eq!(state.character.inventory.len(), 20);
        assert!(!state.character.inventory.iter().any(|i| i.name == "Common Sword"));
        assert_eq!(state.character.inventory.iter().filter(|i| matches!(i.rarity, Rarity::Legendary)).count(), 20);
    }

    #[test]
    fn add_to_inventory_drops_weakest_droppable_from_mixed_inventory() {
        let mut state = make_state();
        for i in 0..18 {
            state.character.inventory.push(make_item(&format!("Epic {}", i), ItemSlot::Weapon, 50 + i as i32, Rarity::Epic));
        }
        state.character.inventory.push(make_item("Weak Common", ItemSlot::Weapon, 1, Rarity::Common));
        state.character.inventory.push(make_item("Medium Rare", ItemSlot::Weapon, 10, Rarity::Rare));
        add_to_inventory(&mut state, make_item("New Epic", ItemSlot::Weapon, 99, Rarity::Epic));
        assert_eq!(state.character.inventory.len(), 20);
        assert!(state.character.inventory.iter().any(|i| i.name == "New Epic"));
        assert!(!state.character.inventory.iter().any(|i| i.name == "Weak Common"));
        assert!(state.character.inventory.iter().any(|i| i.name == "Medium Rare"));
    }

    #[test]
    fn add_to_inventory_drops_rare_before_uncommon_if_rare_is_weaker() {
        let mut state = make_state();
        for i in 0..18 {
            state.character.inventory.push(make_item(&format!("Epic {}", i), ItemSlot::Weapon, 50 + i as i32, Rarity::Epic));
        }
        state.character.inventory.push(make_item("Strong Uncommon", ItemSlot::Weapon, 20, Rarity::Uncommon));
        state.character.inventory.push(make_item("Weak Rare", ItemSlot::Weapon, 5, Rarity::Rare));
        add_to_inventory(&mut state, make_item("New Weapon", ItemSlot::Weapon, 99, Rarity::Legendary));
        assert_eq!(state.character.inventory.len(), 20);
        assert!(state.character.inventory.iter().any(|i| i.name == "New Weapon"));
        assert!(!state.character.inventory.iter().any(|i| i.name == "Weak Rare"));
        assert!(state.character.inventory.iter().any(|i| i.name == "Strong Uncommon"));
    }

    #[test]
    fn scaled_xp_danger_1_returns_base() {
        assert_eq!(scaled_xp(20, 1), 20);
    }

    #[test]
    fn scaled_xp_danger_3_returns_150_percent() {
        assert_eq!(scaled_xp(20, 3), 30);
    }

    #[test]
    fn scaled_xp_danger_5_returns_double() {
        assert_eq!(scaled_xp(20, 5), 40);
    }

    #[test]
    fn affinity_multiplier_wizard_python_returns_1_5() {
        use crate::character::Class;
        assert_eq!(affinity_multiplier(&Class::Wizard, "python"), 1.5);
    }

    #[test]
    fn affinity_multiplier_warrior_no_affinity_returns_1_0() {
        use crate::character::Class;
        assert_eq!(affinity_multiplier(&Class::Warrior, "ls"), 1.0);
    }

    #[test]
    fn final_xp_applies_both_bonuses() {
        use crate::character::Class;
        // Wizard in danger-3 zone with python: base 20 * 1.5 (zone) * 1.5 (affinity) = 45
        assert_eq!(final_xp(20, 3, &Class::Wizard, "python"), 45);
    }

    #[test]
    fn passive_heal_denominator_is_greater_than_four() {
        assert_eq!(passive_heal_denominator(), 10);
    }

    #[test]
    fn encounter_scale_increases_with_danger() {
        assert_eq!(encounter_scale_for_danger(1), 0.9_f32);
        assert_eq!(encounter_scale_for_danger(3), 1.4_f32);
        assert_eq!(encounter_scale_for_danger(5), 2.2_f32);
        assert!(encounter_scale_for_danger(5) > encounter_scale_for_danger(1));
    }

    #[test]
    fn encounter_scale_danger_1_below_base() {
        assert_eq!(encounter_scale_for_danger(1), 0.9_f32);
    }

    #[test]
    fn high_danger_encounter_hits_harder_than_home_zone() {
        let low = encounter_scale_for_danger(1);
        let high = encounter_scale_for_danger(5);
        assert!(
            high > low * 2.0,
            "danger 5 ({}) should be more than 2× danger 1 ({})",
            high,
            low
        );
    }

    #[test]
    fn elite_modifier_raises_attack_and_reward() {
        let elite = apply_elite_pressure("Deadlock Demon", 12, 25, 4);
        assert_eq!(elite.attack, 28);
        assert_eq!(elite.xp, 50);
        assert!(elite.elite);
    }

    #[test]
    fn elite_modifier_prefixes_name() {
        let elite = apply_elite_pressure("Segfault Specter", 8, 15, 3);
        assert!(elite.name.starts_with("Enraged "),
            "Expected name to start with 'Enraged ', got: {}", elite.name);
        assert_eq!(elite.xp, 30);
    }

    #[test]
    fn elite_profile_marks_name_and_reward() {
        let elite = apply_elite_pressure("Segfault Specter", 8, 15, 3);
        assert!(
            elite.name.starts_with("Enraged "),
            "Expected 'Enraged ' prefix, got: {}",
            elite.name
        );
        assert!(elite.xp > 15);
        assert!(elite.elite);
    }
}

pub(crate) fn full_inventory_message(item: &crate::character::Item) -> String {
    use crate::character::ItemSlot;
    let n = &item.name;
    let mut rng = rand::thread_rng();
    let msgs: &[&str] = match item.slot {
        ItemSlot::Weapon => &[
            "{} reaches for your belt, but your legendary arsenal has no vacancy.",
            "Your mythic blades hold council and vote to reject {}. Legends only.",
            "{} dissolves — twenty legendary weapons leave no room.",
        ],
        ItemSlot::Armor => &[
            "Your legendary armor rack is full — {} finds no peg to hang on.",
            "Twenty legendary suits crowd your wardrobe. {} crumples sadly into the ether.",
            "{} tries to squeeze in, but your hall of legendary protection refuses.",
        ],
        ItemSlot::Ring => &[
            "Every finger already bears a legendary band. {} rolls sadly away into the void.",
            "Your mythic jewelry box is sealed. {} vanishes with a soft chime.",
            "{} spins longingly, but your legend-grade ring collection has no vacancy.",
        ],
        ItemSlot::Potion => &[
            "Your legendary pouch rejects {}. It bubbles sadly and evaporates.",
            "{} dissolves before you can grab it — your pack brims with legendary brews.",
            "Twenty epic concoctions stare down {}. It doesn't belong here. Poof.",
        ],
    };
    let idx = rng.gen_range(0..msgs.len());
    msgs[idx].replace("{}", n)
}

fn add_to_inventory(state: &mut GameState, item: crate::character::Item) -> bool {
    const MAX_INVENTORY: usize = 20;
    if state.character.inventory.len() < MAX_INVENTORY {
        state.character.inventory.push(item);
        return true;
    }

    let weakest = state
        .character
        .inventory
        .iter()
        .filter(|i| i.rarity.is_droppable())
        .min_by_key(|i| i.power);

    let should_replace = match weakest {
        Some(w) => item.power > w.power,
        None => false,
    };

    if should_replace {
        let idx = state
            .character
            .inventory
            .iter()
            .enumerate()
            .filter(|(_, i)| i.rarity.is_droppable())
            .min_by_key(|(_, i)| i.power)
            .map(|(idx, _)| idx)
            .unwrap();
        let dropped = state.character.inventory.remove(idx);
        eprintln!(
            "{} {} [{}] was discarded to make room for {}!",
            "🗑️ ".bold(),
            dropped.name.dimmed(),
            format!("{}", dropped.rarity).dimmed(),
            display::color_item_inline(&item.name, &item.rarity)
        );
        state.character.inventory.push(item);
        true
    } else {
        let msg = full_inventory_message(&item);
        eprintln!("{} {}", "📦".bold(), msg.yellow().italic());
        false
    }
}

pub(crate) fn add_to_inventory_pub(state: &mut GameState, item: crate::character::Item) -> bool {
    add_to_inventory(state, item)
}

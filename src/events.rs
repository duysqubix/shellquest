use crate::display;
use crate::journal::{EventType, JournalEntry};
use crate::loot::roll_loot;
use crate::state::GameState;
use crate::zones::{zone_from_path, travel_message};
use colored::*;
use rand::Rng;

fn color_level_up(c: &crate::character::Character) -> String {
    format!("{} You are now level {}! Title: {}",
        "LEVEL UP!".yellow().bold(),
        format!("{}", c.level).green().bold(),
        c.title.cyan().bold().italic())
}

pub fn tick(state: &mut GameState, command: &str, cwd: &str, exit_code: i32) {
    state.character.commands_run += 1;
    let mut rng = rand::thread_rng();

    // Failed commands = traps
    if exit_code != 0 {
        handle_trap(state, &mut rng);
        return;
    }

    // Command-specific events
    let cmd_lower = command.to_lowercase();
    let cmd_base = cmd_lower.split_whitespace().next().unwrap_or("");

    match cmd_base {
        "cd" => {
            if rng.gen_ratio(1, 3) {
                handle_travel(state, cwd);
            }
        }
        "git" => {
            if cmd_lower.contains("commit") {
                handle_craft(state, &mut rng);
            } else if cmd_lower.contains("push") {
                handle_quest(state, &mut rng);
            } else if rng.gen_ratio(1, 5) {
                handle_discovery(state, &mut rng);
            }
        }
        "cargo" | "make" | "npm" | "yarn" | "pnpm" => {
            if cmd_lower.contains("build") || cmd_lower.contains("compile") {
                handle_forge(state, &mut rng, cwd);
            } else if rng.gen_ratio(1, 5) {
                handle_discovery(state, &mut rng);
            }
        }
        "rm" | "del" => {
            if rng.gen_ratio(1, 3) {
                handle_angry_spirit(state, &mut rng);
            }
        }
        "cat" | "bat" | "less" | "more" => {
            if rng.gen_ratio(1, 6) {
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
                handle_portal(state, &mut rng);
            }
        }
        "sudo" => {
            if rng.gen_ratio(1, 4) {
                handle_power_surge(state, &mut rng);
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
                    handle_docker_orchestra(state, &mut rng);
                }
            } else if rng.gen_ratio(1, 4) {
                handle_summon(state, &mut rng, "container golem");
            }
        }
        "python" | "python3" | "node" | "ruby" | "lua" => {
            if rng.gen_ratio(1, 5) {
                handle_incantation(state, &mut rng);
            }
        }
        "pip" | "pip3" | "gem" | "composer" => {
            if rng.gen_ratio(1, 4) {
                handle_alchemy(state, &mut rng);
            }
        }
        "vim" | "nvim" | "emacs" | "nano" | "code" | "hx" => {
            if rng.gen_ratio(1, 5) {
                handle_meditation(state, &mut rng);
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
                handle_banish(state, &mut rng);
            }
        }
        "tar" | "zip" | "unzip" | "gzip" => {
            if rng.gen_ratio(1, 4) {
                handle_treasure_chest(state, &mut rng, cwd);
            }
        }
        "echo" | "printf" => {
            if rng.gen_ratio(1, 6) {
                handle_echo_spell(state, &mut rng);
            }
        }
        "man" | "tldr" | "help" => {
            if rng.gen_ratio(1, 4) {
                handle_ancient_tome(state, &mut rng);
            }
        }
        _ => {
            // Generic random encounter ~15% of the time
            if rng.gen_ratio(3, 20) {
                handle_random_encounter(state, &mut rng, cwd);
            }
        }
    }

    // Passive healing over time
    if state.character.hp < state.character.max_hp && rng.gen_ratio(1, 4) {
        state.character.heal(1);
    }
}

fn handle_trap(state: &mut GameState, rng: &mut impl Rng) {
    let damage = rng.gen_range(1..=3);
    let died = state.character.take_damage(damage);
    let plain = if died {
        format!("You triggered a trap! Took {} damage and fell in battle... Respawning with a gold penalty.", damage)
    } else {
        format!("You stumble on a trap! Took {} damage. HP: {}/{}", damage, state.character.hp, state.character.max_hp)
    };
    let colored = if died {
        format!("{} a {}! Took {} damage and {} Respawning with a {} penalty.",
            "You triggered".red(), "trap".red().bold(), display::color_damage(damage),
            "fell in battle...".red().bold(), "gold".yellow())
    } else {
        format!("{} a {}! Took {} damage. {}",
            "You stumble on".red(), "trap".red().bold(), display::color_damage(damage),
            display::color_hp(state.character.hp, state.character.max_hp))
    };
    display::print_trap(&colored);
    let event_type = if died { EventType::Death } else { EventType::Combat };
    state.add_journal(JournalEntry::new(event_type, plain));
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

fn handle_craft(state: &mut GameState, rng: &mut impl Rng) {
    let xp = rng.gen_range(10..=25);
    let leveled = state.character.gain_xp(xp);
    let plain = format!("You committed your work to the archives! +{} XP", xp);
    let colored = format!("{} your work to the {}! {}",
        "You committed".cyan(), "archives".cyan().bold(), display::color_xp(xp));
    display::print_craft(&colored);
    state.add_journal(JournalEntry::new(EventType::Craft, plain));
    if leveled {
        let lvl_msg = format!("LEVEL UP! You are now level {}! Title: {}", state.character.level, state.character.title);
        display::print_level_up(&color_level_up(&state.character));
        state.add_journal(JournalEntry::new(EventType::LevelUp, lvl_msg));
    }
}

fn check_level_up(state: &mut GameState, leveled: bool) {
    if leveled {
        let plain = format!("LEVEL UP! You are now level {}! Title: {}", state.character.level, state.character.title);
        display::print_level_up(&color_level_up(&state.character));
        state.add_journal(JournalEntry::new(EventType::LevelUp, plain));
    }
}

fn handle_quest(state: &mut GameState, rng: &mut impl Rng) {
    let xp = rng.gen_range(15..=35);
    let gold = rng.gen_range(5..=20);
    let leveled = state.character.gain_xp(xp);
    state.character.gold += gold;
    let plain = format!("Quest complete! You pushed your code to the realm! +{} XP, +{} gold", xp, gold);
    let colored = format!("{} You {} your code to the {}! {} {}",
        "Quest complete!".yellow().bold(),
        "pushed".green().bold(), "realm".cyan().bold(),
        display::color_xp(xp), display::color_gold(gold));
    display::print_quest(&colored);
    state.add_journal(JournalEntry::new(EventType::Quest, plain));
    check_level_up(state, leveled);
}

fn handle_discovery(state: &mut GameState, rng: &mut impl Rng) {
    let xp = rng.gen_range(5..=15);
    let leveled = state.character.gain_xp(xp);
    let discoveries_plain = [
        "You found an ancient code comment from a forgotten developer!",
        "You discovered a hidden TODO that grants wisdom!",
        "You unearthed a deprecated scroll of knowledge!",
        "A mysterious FIXME glows with arcane energy!",
        "You found a secret .env file buried in the ruins!",
    ];
    let discoveries_color = [
        format!("You found an {} from a {} developer!", "ancient code comment".magenta().bold(), "forgotten".dimmed()),
        format!("You discovered a {} that grants {}!", "hidden TODO".magenta().bold(), "wisdom".cyan()),
        format!("You unearthed a {} of knowledge!", "deprecated scroll".magenta().bold()),
        format!("A mysterious {} glows with {} energy!", "FIXME".red().bold(), "arcane".magenta()),
        format!("You found a {} {} buried in the ruins!", "secret".red(), ".env file".yellow().bold()),
    ];
    let idx = rng.gen_range(0..discoveries_plain.len());
    let plain = format!("{} +{} XP", discoveries_plain[idx], xp);
    let colored = format!("{} {}", discoveries_color[idx], display::color_xp(xp));
    display::print_discovery(&colored);
    state.add_journal(JournalEntry::new(EventType::Discovery, plain));
    check_level_up(state, leveled);
}

fn handle_forge(state: &mut GameState, rng: &mut impl Rng, cwd: &str) {
    let zone = zone_from_path(cwd);
    if rng.gen_ratio(1, 3) {
        let item = roll_loot(zone.danger_level);
        let plain = format!("The forge burns hot! You crafted: {} (+{} {}) [{}]", item.name, item.power, item.slot, item.rarity);
        let colored = format!("The {} burns hot! You crafted: {} ({} {}) [{}]",
            "forge".red().bold(),
            display::color_item_inline(&item.name, &item.rarity),
            format!("+{}", item.power).white().bold(),
            format!("{}", item.slot).dimmed(),
            format!("{}", item.rarity).dimmed());
        display::print_loot(&colored, &item.rarity);
        state.add_journal(JournalEntry::new(EventType::Craft, plain));
        add_to_inventory(state, item);
    } else {
        let xp = rng.gen_range(8..=20);
        state.character.gain_xp(xp);
        let plain = format!("The build completes! The heat tempers your resolve. +{} XP", xp);
        let colored = format!("The {} completes! The {} tempers your resolve. {}",
            "build".cyan().bold(), "heat".red(), display::color_xp(xp));
        display::print_craft(&colored);
        state.add_journal(JournalEntry::new(EventType::Craft, plain));
    }
}

fn handle_angry_spirit(state: &mut GameState, rng: &mut impl Rng) {
    let monster = random_monster(rng);
    combat(state, rng, &monster.0, monster.1, monster.2);
}

fn handle_familiar(state: &mut GameState, rng: &mut impl Rng) {
    let familiars = ["a curious cat", "a friendly daemon", "a pixel sprite", "a tame penguin", "a binary beetle"];
    let heal = rng.gen_range(3..=8);
    state.character.heal(heal);
    let idx = rng.gen_range(0..familiars.len());
    let plain = format!("You befriend {}! It heals you for {} HP. HP: {}/{}", familiars[idx], heal, state.character.hp, state.character.max_hp);
    let colored = format!("You befriend {}! It {} you for {} HP. {}",
        familiars[idx].green().bold(),
        "heals".green(), format!("{}", heal).green().bold(),
        display::color_hp(state.character.hp, state.character.max_hp));
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

fn handle_portal(state: &mut GameState, rng: &mut impl Rng) {
    let xp = rng.gen_range(10..=20);
    let leveled = state.character.gain_xp(xp);
    let plain = format!("You opened a portal to a remote realm! The journey grants you +{} XP", xp);
    let colored = format!("You opened a {} to a {}! The journey grants you {}",
        "portal".cyan().bold(), "remote realm".blue().bold(), display::color_xp(xp));
    display::print_portal(&colored);
    state.add_journal(JournalEntry::new(EventType::Travel, plain));
    check_level_up(state, leveled);
}

fn handle_power_surge(state: &mut GameState, rng: &mut impl Rng) {
    let xp = rng.gen_range(15..=30);
    let leveled = state.character.gain_xp(xp);
    let plain = format!("You invoke the power of SUDO! Raw energy courses through you! +{} XP", xp);
    let colored = format!("You invoke the power of {}! {} courses through you! {}",
        "SUDO".red().bold().on_black(), "Raw energy".red().bold(), display::color_xp(xp));
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

fn handle_incantation(state: &mut GameState, rng: &mut impl Rng) {
    let xp = rng.gen_range(8..=18);
    let leveled = state.character.gain_xp(xp);
    let spells = [
        "You chant an interpreted incantation! The code spirits answer!",
        "You invoke a scripting ritual! Power surges through the REPL!",
        "You cast eval()! Reality bends to your will!",
        "You weave a dynamic spell! Variables dance in the air!",
    ];
    let msg = format!("{} +{} XP", spells[rng.gen_range(0..spells.len())], xp);
    display::print_discovery(&msg);
    state.add_journal(JournalEntry::new(EventType::Discovery, msg));
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

fn handle_meditation(state: &mut GameState, rng: &mut impl Rng) {
    let heal = rng.gen_range(5..=12);
    let xp = rng.gen_range(5..=10);
    state.character.heal(heal);
    let leveled = state.character.gain_xp(xp);
    let msgs = [
        "You enter the editor trance... inner peace flows through your keystrokes.",
        "You meditate within the buffer... your mind and code become one.",
        "The sacred editor calms your spirit. Modal enlightenment achieved.",
    ];
    let msg = format!("{} +{} HP, +{} XP", msgs[rng.gen_range(0..msgs.len())], heal, xp);
    display::print_familiar(&msg);
    state.add_journal(JournalEntry::new(EventType::Discovery, msg));
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

fn handle_banish(state: &mut GameState, rng: &mut impl Rng) {
    let xp = rng.gen_range(15..=25);
    let gold = rng.gen_range(3..=10);
    let leveled = state.character.gain_xp(xp);
    state.character.gold += gold;
    state.character.kills += 1;
    let msgs = [
        "You banish a rogue process to the void!",
        "SIGKILL! The daemon is vanquished instantly!",
        "You send the process to /dev/null! It shall not return!",
    ];
    let msg = format!("{} +{} XP, +{} gold", msgs[rng.gen_range(0..msgs.len())], xp, gold);
    display::print_combat_win(&msg);
    state.add_journal(JournalEntry::new(EventType::Combat, msg));
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
    let heal = rng.gen_range(2..=5);
    state.character.heal(heal);
    let msg = format!("Your words echo through the terminal void... the resonance heals you! +{} HP", heal);
    display::print_familiar(&msg);
    state.add_journal(JournalEntry::new(EventType::Discovery, msg));
}

fn handle_ancient_tome(state: &mut GameState, rng: &mut impl Rng) {
    let xp = rng.gen_range(10..=22);
    let leveled = state.character.gain_xp(xp);
    let msgs = [
        "You consult the ancient man pages! Forbidden knowledge flows into you!",
        "The tome of documentation reveals its secrets! Understanding dawns!",
        "You study the sacred scrolls! The wisdom of the ancients empowers you!",
    ];
    let msg = format!("{} +{} XP", msgs[rng.gen_range(0..msgs.len())], xp);
    display::print_discovery(&msg);
    state.add_journal(JournalEntry::new(EventType::Discovery, msg));
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

fn handle_docker_orchestra(state: &mut GameState, rng: &mut impl Rng) {
    let xp = rng.gen_range(15..=30);
    let gold = rng.gen_range(5..=15);
    let leveled = state.character.gain_xp(xp);
    state.character.gold += gold;
    let msgs = [
        "You conduct the container orchestra! Services rise in harmony!",
        "Compose magic weaves through the stack! All services hum in unison!",
        "The orchestration ritual completes! A symphony of microservices plays!",
    ];
    let msg = format!("{} +{} XP, +{} gold", msgs[rng.gen_range(0..msgs.len())], xp, gold);
    display::print_quest(&msg);
    state.add_journal(JournalEntry::new(EventType::Quest, msg));
    check_level_up(state, leveled);
}

fn handle_random_encounter(state: &mut GameState, rng: &mut impl Rng, cwd: &str) {
    let zone = zone_from_path(cwd);
    let roll: u32 = rng.gen_range(1..=100);

    match roll {
        1..=40 => {
            // Combat encounter
            let monster = random_monster_for_zone(rng, &zone);
            combat(state, rng, &monster.0, monster.1, monster.2);
        }
        41..=60 => {
            // Find loot
            let item = roll_loot(zone.danger_level);
            let msg = format!("You found: {} (+{} {}) [{}]", item.name, item.power, item.slot, item.rarity);
            display::print_loot(&msg, &item.rarity);
            state.add_journal(JournalEntry::new(EventType::Loot, msg));
            add_to_inventory(state, item);
        }
        61..=75 => {
            // Find gold
            let gold = rng.gen_range(1..=8) * zone.danger_level;
            state.character.gold += gold;
            let msg = format!("You found {} gold coins hidden in the path!", gold);
            display::print_gold(&msg);
            state.add_journal(JournalEntry::new(EventType::Loot, msg));
        }
        76..=90 => {
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
    let scale = zone.danger_level as f32 / 2.0;
    let atk = (base_atk as f32 * scale).max(1.0) as i32;
    let xp = (base_xp as f32 * scale).max(5.0) as u32;
    (name, atk, xp)
}

fn combat(state: &mut GameState, rng: &mut impl Rng, monster_name: &str, monster_atk: i32, xp_reward: u32) {
    let player_power = state.character.attack_power();
    let player_defense = state.character.defense();
    let hit_roll: i32 = rng.gen_range(1..=20);
    let dodge_roll: i32 = rng.gen_range(1..=20);

    let player_hits = hit_roll + player_power > 10;
    let monster_hits = dodge_roll > (10 + player_defense);
    let mname = display::color_monster(monster_name);

    if player_hits && !monster_hits {
        state.character.kills += 1;
        let leveled = state.character.gain_xp(xp_reward);
        let plain = format!("A {} appears! You strike true! Victory! +{} XP", monster_name, xp_reward);
        let colored = format!("A {} appears! You {}! {}! {}",
            mname, "strike true".green().bold(), "Victory".green().bold(), display::color_xp(xp_reward));
        display::print_combat_win(&colored);
        state.add_journal(JournalEntry::new(EventType::Combat, plain));
        check_level_up(state, leveled);
    } else if player_hits && monster_hits {
        let damage = (monster_atk - player_defense).max(1);
        let died = state.character.take_damage(damage);
        if !died {
            state.character.kills += 1;
            state.character.gain_xp(xp_reward);
        }
        let plain = if died {
            format!("A {} appears! You trade blows... you fall! Lost some gold. Respawning...", monster_name)
        } else {
            format!("A {} appears! Tough fight! You win but took {} damage. +{} XP. HP: {}/{}", monster_name, damage, xp_reward, state.character.hp, state.character.max_hp)
        };
        let colored = if died {
            format!("A {} appears! You {}... {} Lost some {}. {}...",
                mname, "trade blows".yellow(), "you fall!".red().bold(), "gold".yellow(), "Respawning".dimmed())
        } else {
            format!("A {} appears! {} You win but took {} damage. {} {}",
                mname, "Tough fight!".yellow().bold(), display::color_damage(damage),
                display::color_xp(xp_reward), display::color_hp(state.character.hp, state.character.max_hp))
        };
        display::print_combat_tough(&colored, died);
        let event_type = if died { EventType::Death } else { EventType::Combat };
        state.add_journal(JournalEntry::new(event_type, plain));
    } else if !player_hits && monster_hits {
        let damage = (monster_atk - player_defense / 2).max(1);
        let died = state.character.take_damage(damage);
        let plain = if died {
            format!("A {} appears! It overwhelms you! You fall... Lost some gold. Respawning...", monster_name)
        } else {
            format!("A {} appears! It hits you for {} damage! You flee. HP: {}/{}", monster_name, damage, state.character.hp, state.character.max_hp)
        };
        let colored = if died {
            format!("A {} appears! It {}! {} Lost some {}. {}...",
                mname, "overwhelms you".red().bold(), "You fall...".red().bold(), "gold".yellow(), "Respawning".dimmed())
        } else {
            format!("A {} appears! It {} you for {} damage! You {}. {}",
                mname, "hits".red().bold(), display::color_damage(damage),
                "flee".yellow(), display::color_hp(state.character.hp, state.character.max_hp))
        };
        display::print_combat_lose(&colored, died);
        let event_type = if died { EventType::Death } else { EventType::Combat };
        state.add_journal(JournalEntry::new(event_type, plain));
    } else {
        let plain = format!("A {} appears! You circle each other... it retreats into the shadows.", monster_name);
        let colored = format!("A {} appears! You {}... it {} into the shadows.",
            mname, "circle each other".dimmed(), "retreats".dimmed().italic());
        display::print_combat_draw(&colored);
        state.add_journal(JournalEntry::new(EventType::Combat, plain));
    }
}

fn add_to_inventory(state: &mut GameState, item: crate::character::Item) {
    const MAX_INVENTORY: usize = 20;
    if state.character.inventory.len() >= MAX_INVENTORY {
        // Drop lowest-power item to make room
        if let Some(min_idx) = state.character.inventory.iter().enumerate().min_by_key(|(_, i)| i.power).map(|(idx, _)| idx) {
            let dropped = state.character.inventory.remove(min_idx);
            if item.power <= dropped.power {
                // New item is worse, just drop it instead
                state.character.inventory.push(dropped);
                return;
            }
        }
    }
    state.character.inventory.push(item);
}

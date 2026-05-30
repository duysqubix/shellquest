use crate::display;
use crate::journal::{EventType, JournalEntry};
use crate::loot::roll_loot;
use crate::state::GameState;
use crate::zones::{is_void_zone, travel_message, void_depth, zone_from_path};
use colored::*;
use rand::Rng;
use serde::Serialize;

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
        Class::Wizard => &[
            "python", "python3", "node", "ruby", "vim", "nvim", "emacs", "man", "tldr", "jupyter",
            "sed", "awk", "perl", "jq", "yq", "lua", "php", "r", "julia", "ghci", "history", "fc",
        ],
        Class::Warrior => &[
            "cargo", "make", "cmake", "gcc", "g++", "ninja", "meson", "mvn", "gradle",
            "clang", "rustc", "go", "javac", "dotnet", "bazel", "buck", "xcodebuild",
            "eslint", "prettier", "black", "ruff", "gofmt", "rustfmt", "clippy",
        ],
        Class::Rogue => &[
            "grep", "rg", "ag", "ssh", "find", "fd", "ls", "eza", "locate",
            "sort", "uniq", "cut", "tr", "wc", "head", "tail", "which", "whereis", "tree",
            "diff", "comm",
        ],
        Class::Ranger => &[
            "curl", "wget", "http", "docker", "kubectl", "ansible", "terraform", "helm",
            "ping", "dig", "nslookup", "host", "traceroute", "scp", "sftp", "nc", "netcat",
            "nmap", "netstat", "ss", "ifconfig", "ip", "vagrant", "packer",
        ],
        Class::Necromancer => &[
            "kill", "pkill", "killall", "rm", "del", "git", "shred",
            "systemctl", "service", "launchctl", "shutdown", "reboot", "halt", "dd", "wipe",
        ],
    };
    if affinities
        .iter()
        .any(|&a| cmd == a || cmd.starts_with(&format!("{} ", a)))
    {
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

const HOME_HEAL_INTERVAL_SECS: i64 = 30;
const HOME_HEAL_MAX_ACCUMULATED_SECS: i64 = 30 * 60;

const VOID_ENCOUNTER_NUMERATOR: u32 = 2;
const VOID_ENCOUNTER_DENOMINATOR: u32 = 3;
const VOID_MIN_COMBAT_DEPTH: u32 = 1;

/// Void mobs scale from their roster baseline with both character level and maze depth.
/// HP = base_hp + level * 6 + depth * 12.
/// ATK = base_attack + floor(level / 2) + depth * 3.
/// XP = base_xp + level * 2 + depth * 10, before normal zone/affinity XP multipliers.
const VOID_HP_PER_LEVEL: i32 = 6;
const VOID_HP_PER_DEPTH: i32 = 12;
const VOID_ATTACK_LEVEL_DIVISOR: u32 = 2;
const VOID_ATTACK_PER_DEPTH: i32 = 3;
const VOID_XP_PER_LEVEL: u32 = 2;
const VOID_XP_PER_DEPTH: u32 = 10;
const VOID_DEX_FLOOR: i32 = 6;
const VOID_DEX_LEVEL_DIVISOR: u32 = 2;
const VOID_DEX_LEVEL_OFFSET: i32 = -5;

#[derive(Debug, Clone, Copy)]
struct VoidMonsterTemplate {
    name: &'static str,
    flavor: &'static str,
    base_hp: i32,
    base_attack: i32,
    base_xp: u32,
}

#[derive(Debug, Clone)]
struct VoidMob {
    name: String,
    hp: i32,
    attack: i32,
    xp: u32,
}

const VOID_MONSTER_ROSTER: &[VoidMonsterTemplate] = &[
    VoidMonsterTemplate {
        name: "Null Wraith",
        flavor: "wearing your last prompt as a face",
        base_hp: 16,
        base_attack: 5,
        base_xp: 22,
    },
    VoidMonsterTemplate {
        name: "Echo Eater",
        flavor: "chewing on commands that never returned",
        base_hp: 18,
        base_attack: 6,
        base_xp: 24,
    },
    VoidMonsterTemplate {
        name: "Orphaned Inode",
        flavor: "rattling with unlinked memories",
        base_hp: 22,
        base_attack: 5,
        base_xp: 26,
    },
    VoidMonsterTemplate {
        name: "Black Cursor",
        flavor: "blinking where your exit should be",
        base_hp: 14,
        base_attack: 8,
        base_xp: 23,
    },
    VoidMonsterTemplate {
        name: "TTY Lurker",
        flavor: "breathing static into the terminal line",
        base_hp: 20,
        base_attack: 7,
        base_xp: 28,
    },
    VoidMonsterTemplate {
        name: "Broken Prompt",
        flavor: "asking questions with no shell left to answer",
        base_hp: 24,
        base_attack: 8,
        base_xp: 32,
    },
    VoidMonsterTemplate {
        name: "Rift-Mouthed Daemon",
        flavor: "opening symlinks in its throat",
        base_hp: 18,
        base_attack: 9,
        base_xp: 30,
    },
    VoidMonsterTemplate {
        name: "Zero-Width Horror",
        flavor: "standing between two path separators",
        base_hp: 28,
        base_attack: 6,
        base_xp: 34,
    },
    VoidMonsterTemplate {
        name: "Portal Maw",
        flavor: "gnashing at the home-rift threshold",
        base_hp: 30,
        base_attack: 7,
        base_xp: 36,
    },
    VoidMonsterTemplate {
        name: "Severed Pipe",
        flavor: "leaking commands into nowhere",
        base_hp: 16,
        base_attack: 10,
        base_xp: 29,
    },
    VoidMonsterTemplate {
        name: "Entropy Shade",
        flavor: "unmaking directory names one glyph at a time",
        base_hp: 22,
        base_attack: 9,
        base_xp: 33,
    },
    VoidMonsterTemplate {
        name: "Dead-Sector Seraph",
        flavor: "singing hymns from corrupted blocks",
        base_hp: 32,
        base_attack: 6,
        base_xp: 38,
    },
    VoidMonsterTemplate {
        name: "Forkbomb Revenant",
        flavor: "multiplying every shadow behind it",
        base_hp: 20,
        base_attack: 11,
        base_xp: 35,
    },
    VoidMonsterTemplate {
        name: "Untracked Phantom",
        flavor: "hovering outside every commit's memory",
        base_hp: 18,
        base_attack: 8,
        base_xp: 31,
    },
    VoidMonsterTemplate {
        name: "Heap Specter",
        flavor: "dragging freed memory through the rift",
        base_hp: 26,
        base_attack: 9,
        base_xp: 37,
    },
    VoidMonsterTemplate {
        name: "Cursed Symlink",
        flavor: "pointing at a room that points back",
        base_hp: 24,
        base_attack: 10,
        base_xp: 39,
    },
    VoidMonsterTemplate {
        name: "Static Devourer",
        flavor: "eating the signal from severed sessions",
        base_hp: 34,
        base_attack: 8,
        base_xp: 42,
    },
    VoidMonsterTemplate {
        name: "The Unmapped",
        flavor: "standing where no path component should exist",
        base_hp: 28,
        base_attack: 11,
        base_xp: 45,
    },
];

pub fn tick(state: &mut GameState, command: &str, cwd: &str, exit_code: i32) {
    let mut rng = rand::thread_rng();
    tick_with_rng(state, command, cwd, exit_code, &mut rng);
}

fn tick_with_rng(
    state: &mut GameState,
    command: &str,
    cwd: &str,
    exit_code: i32,
    mut rng: &mut impl Rng,
) {
    state.character.commands_run += 1;

    let now = chrono::Utc::now();
    handle_passive_healer(state, cwd, now);

    if exit_code != 0 {
        if rng.gen_ratio(1, 4) {
            handle_trap(state, rng);
        }
        return;
    }

    // Command-specific events
    let cmd_lower = command.to_lowercase();
    let cmd_base = cmd_lower.split_whitespace().next().unwrap_or("");
    let zone = zone_from_path(cwd);

    if is_void_zone(&zone) && rng.gen_ratio(VOID_ENCOUNTER_NUMERATOR, VOID_ENCOUNTER_DENOMINATOR) {
        handle_void_encounter(state, rng, &zone, cwd, &cmd_lower);
        handle_post_command_tick(state, rng);
        return;
    }

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
                    handle_container_forge(state, &mut rng, cwd, &cmd_lower);
                }
            } else if cmd_lower.contains("run") || cmd_lower.contains("exec") {
                if rng.gen_ratio(1, 3) {
                    handle_summon(state, &mut rng, "container golem");
                }
            } else if cmd_lower.contains("pull") {
                if rng.gen_ratio(1, 4) {
                    handle_docker_pull(state, &mut rng);
                }
            } else if cmd_lower.contains("stop")
                || cmd_lower.contains("kill")
                || cmd_lower.contains("rm")
            {
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
                handle_trial(state, &mut rng, &zone, &cmd_lower);
            }
        }
        "cp" | "mv" | "rsync" => {
            if rng.gen_ratio(1, 5) {
                handle_telekinesis(state, &mut rng, &zone, &cmd_lower);
            }
        }
        "chmod" | "chown" | "chgrp" => {
            if rng.gen_ratio(1, 4) {
                handle_enchant(state, &mut rng, &zone, &cmd_lower);
            }
        }
        "top" | "htop" | "btm" | "ps" => {
            if rng.gen_ratio(1, 5) {
                handle_omniscience(state, &mut rng, &zone, &cmd_lower);
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
        // Text divination / data inspection -> Rogue/Ranger flavor
        "sort" | "uniq" | "cut" | "tr" | "wc" | "head" | "tail" | "diff" | "comm"
        | "nmap" | "netstat" | "ss" | "ifconfig" | "ip" => {
            if rng.gen_ratio(1, 5) {
                handle_omniscience(state, &mut rng, &zone, &cmd_lower);
            }
        }
        // Locating / mapping the filesystem -> Rogue
        "which" | "whereis" | "tree" => {
            if rng.gen_ratio(1, 5) {
                handle_search_loot(state, &mut rng, cwd);
            }
        }
        // Network reconnaissance / scrying -> Ranger
        "ping" | "dig" | "nslookup" | "host" | "traceroute" => {
            if rng.gen_ratio(1, 4) {
                handle_portal(state, &mut rng, &zone, &cmd_lower);
            }
        }
        // Moving data across the wire -> Ranger
        "scp" | "sftp" | "nc" | "netcat" => {
            if rng.gen_ratio(1, 5) {
                handle_telekinesis(state, &mut rng, &zone, &cmd_lower);
            }
        }
        // Provisioning images / machines -> Ranger
        "vagrant" | "packer" => {
            if rng.gen_ratio(1, 3) {
                handle_container_forge(state, &mut rng, cwd, &cmd_lower);
            }
        }
        // Transmutation: reshape text/data -> Wizard (CUSTOM flavor)
        "sed" | "awk" | "perl" | "jq" | "yq" => {
            if rng.gen_ratio(1, 4) {
                handle_transmute(state, &mut rng, &zone, &cmd_lower);
            }
        }
        // Interpreted/arcane languages -> Wizard
        "php" | "r" | "julia" | "ghci" | "perl6" | "raku" => {
            if rng.gen_ratio(1, 5) {
                handle_incantation(state, &mut rng, &zone, &cmd_lower);
            }
        }
        // Commune with your past -> Wizard (CUSTOM flavor)
        "history" | "fc" => {
            if rng.gen_ratio(1, 4) {
                handle_commune(state, &mut rng, &zone, &cmd_lower);
            }
        }
        // Compilers / builders -> Warrior
        "clang" | "rustc" | "go" | "javac" | "dotnet" | "bazel" | "buck" | "xcodebuild" => {
            if rng.gen_ratio(1, 4) {
                handle_forge(state, &mut rng, &zone, &cmd_lower);
            }
        }
        // Linters / formatters: prove your code -> Warrior
        "eslint" | "prettier" | "black" | "ruff" | "gofmt" | "rustfmt" | "clippy" => {
            if rng.gen_ratio(1, 4) {
                handle_trial(state, &mut rng, &zone, &cmd_lower);
            }
        }
        // Command the daemons -> Necromancer (CUSTOM flavor)
        "systemctl" | "service" | "launchctl" => {
            if rng.gen_ratio(1, 3) {
                handle_command_daemon(state, &mut rng, &zone, &cmd_lower);
            }
        }
        // Apocalyptic power -> Necromancer
        "shutdown" | "reboot" | "halt" | "poweroff" => {
            if rng.gen_ratio(1, 4) {
                handle_power_surge(state, &mut rng, &zone, &cmd_lower);
            }
        }
        // Raw disk power: high risk -> Necromancer (CUSTOM flavor)
        "dd" | "wipe" => {
            if rng.gen_ratio(1, 3) {
                handle_raw_power(state, &mut rng, cwd);
            }
        }
        // Exploring volumes -> neutral
        "df" | "du" | "free" | "mount" | "umount" | "lsblk" => {
            if rng.gen_ratio(1, 4) {
                handle_treasure_chest(state, &mut rng, cwd);
            }
        }
        // Conjuring files into being -> neutral
        "mkdir" | "rmdir" | "touch" | "ln" => {
            if rng.gen_ratio(1, 6) {
                handle_telekinesis(state, &mut rng, &zone, &cmd_lower);
            }
        }
        // Cleanse the screen -> neutral (CUSTOM flavor)
        "clear" | "reset" | "tput" => {
            if rng.gen_ratio(1, 8) {
                handle_cleanse(state, &mut rng);
            }
        }
        // Binding power to a name -> neutral
        "alias" | "export" | "env" | "source" | "set" | "unset" => {
            if rng.gen_ratio(1, 6) {
                handle_enchant(state, &mut rng, &zone, &cmd_lower);
            }
        }
        // Splitting your focus -> neutral
        "tmux" | "screen" | "zellij" => {
            if rng.gen_ratio(1, 8) {
                handle_meditation(state, &mut rng, &zone, &cmd_lower);
            }
        }
        // Brewing packages -> neutral
        "brew" | "apt" | "apt-get" | "dnf" | "yum" | "pacman" | "port" => {
            if rng.gen_ratio(1, 4) {
                handle_alchemy(state, &mut rng);
            }
        }
        _ => {
            // Generic random encounter ~15% of the time
            if rng.gen_ratio(3, 20) {
                handle_random_encounter(state, &mut rng, &zone, &cmd_lower);
            }
        }
    }

    handle_post_command_tick(state, rng);
}

fn trap_damage(max_hp: i32, rng: &mut impl Rng) -> i32 {
    let pct: f32 = rng.gen_range(0.03..0.06);
    ((max_hp as f32 * pct).round() as i32).max(1)
}

fn handle_post_command_tick(state: &mut GameState, rng: &mut impl Rng) {
    if state.character.hp < state.character.max_hp && rng.gen_ratio(1, passive_heal_denominator()) {
        state.character.heal(1);
    }

    crate::boss::tick_boss(state);
    crate::boss::maybe_spawn(state);
}

fn scale_void_mob(template: &VoidMonsterTemplate, player_level: u32, depth: u32) -> VoidMob {
    let level = player_level.max(1);
    let depth = depth.max(VOID_MIN_COMBAT_DEPTH);
    let level_attack = (level / VOID_ATTACK_LEVEL_DIVISOR) as i32;
    let name = format!("{} ({})", template.name, template.flavor);

    VoidMob {
        name,
        hp: template.base_hp + (level as i32 * VOID_HP_PER_LEVEL) + (depth as i32 * VOID_HP_PER_DEPTH),
        attack: template.base_attack + level_attack + (depth as i32 * VOID_ATTACK_PER_DEPTH),
        xp: template.base_xp + (level * VOID_XP_PER_LEVEL) + (depth * VOID_XP_PER_DEPTH),
    }
}

fn random_void_mob(rng: &mut impl Rng, player_level: u32, depth: u32) -> VoidMob {
    let template = &VOID_MONSTER_ROSTER[rng.gen_range(0..VOID_MONSTER_ROSTER.len())];
    scale_void_mob(template, player_level, depth)
}

fn void_enemy_dex_mod(player_level: u32, depth: u32, prestiges: u32) -> i32 {
    let level = player_level.max(1);
    let depth = depth.max(VOID_MIN_COMBAT_DEPTH) as i32;
    let scaled = (level / VOID_DEX_LEVEL_DIVISOR) as i32 + depth + VOID_DEX_LEVEL_OFFSET;
    scaled.max(VOID_DEX_FLOOR + depth) + prestiges as i32
}

fn handle_void_encounter(
    state: &mut GameState,
    rng: &mut impl Rng,
    zone: &crate::zones::Zone,
    cwd: &str,
    cmd: &str,
) {
    let depth = void_depth(cwd).unwrap_or_default().max(VOID_MIN_COMBAT_DEPTH);
    let mob = random_void_mob(rng, state.character.level, depth);
    let enemy_dex_mod = void_enemy_dex_mod(
        state.character.level,
        depth,
        state.character.total_prestiges,
    );

    combat(
        state,
        rng,
        zone,
        cmd,
        &mob.name,
        mob.attack,
        mob.hp,
        mob.xp,
        false,
        enemy_dex_mod,
    );
}

fn handle_trap(state: &mut GameState, rng: &mut impl Rng) {
    let damage = trap_damage(state.character.max_hp, rng);
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
    let colored = format!(
        "You enter {}... {}",
        display::color_zone(zone.name, &zone),
        zone.description.italic().dimmed()
    );
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
    state.add_journal(crate::journal::JournalEntry::new(
        crate::journal::EventType::LevelUp,
        plain,
    ));
}

fn check_level_up(state: &mut GameState, leveled: bool) {
    if leveled {
        emit_level_up(state);
    }
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

fn handle_discovery(
    state: &mut GameState,
    rng: &mut impl Rng,
    zone: &crate::zones::Zone,
    cmd: &str,
) {
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
        let (plain, colored) =
            crate::messages::forge_loot(&state.character.class, &item.name, item.power, xp);
        display::print_loot(&colored, &item.rarity);
        state.add_journal(JournalEntry::new(EventType::Craft, plain));
        add_to_inventory(state, item, false);
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

fn handle_angry_spirit(
    state: &mut GameState,
    rng: &mut impl Rng,
    zone: &crate::zones::Zone,
    cmd: &str,
) {
    let (name, atk, hp, xp, tier) = random_monster_for_zone(rng, zone);
    let enemy_dex_mod = tier_dex_mod(tier) + state.character.total_prestiges as i32;
    let profile = if rng.gen_ratio(1, 8) {
        apply_elite_pressure(&name, atk, hp, xp, zone.danger_level)
    } else {
        EncounterProfile {
            name,
            attack: atk,
            hp,
            xp,
            elite: false,
        }
    };

    combat(
        state,
        rng,
        zone,
        cmd,
        &profile.name,
        profile.attack,
        profile.hp,
        profile.xp,
        profile.elite,
        enemy_dex_mod,
    );
}

fn handle_familiar(state: &mut GameState, rng: &mut impl Rng) {
    let familiars = [
        "curious cat",
        "friendly daemon",
        "pixel sprite",
        "tame penguin",
        "binary beetle",
    ];
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

fn handle_passive_healer(state: &mut GameState, cwd: &str, now: chrono::DateTime<chrono::Utc>) {
    if state.active_boss.is_some() {
        return;
    }

    let last = match state.last_heal_at {
        Some(t) if t <= now => t,
        _ => {
            state.last_heal_at = Some(now);
            return;
        }
    };

    let raw_elapsed = (now - last).num_seconds().max(0);
    let (effective_last, elapsed) = if raw_elapsed > HOME_HEAL_MAX_ACCUMULATED_SECS {
        (
            now - chrono::Duration::seconds(HOME_HEAL_MAX_ACCUMULATED_SECS),
            HOME_HEAL_MAX_ACCUMULATED_SECS,
        )
    } else {
        (last, raw_elapsed)
    };

    let heals_due = (elapsed / HOME_HEAL_INTERVAL_SECS) as i32;
    let missing = (state.character.max_hp - state.character.hp).max(0);
    let applied = heals_due.min(missing);

    if applied == 0 {
        if missing == 0 {
            state.last_heal_at = Some(now);
        }
        return;
    }

    state.character.heal(applied);

    let consumed_secs = (applied as i64) * HOME_HEAL_INTERVAL_SECS;
    let new_last = effective_last + chrono::Duration::seconds(consumed_secs);
    state.last_heal_at = Some(if state.character.hp >= state.character.max_hp {
        now
    } else {
        new_last
    });

    let home = dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    if home.is_empty() || cwd != home {
        return;
    }

    let (plain, colored) = crate::messages::healer(
        &state.character.class,
        applied,
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
    let colored = format!(
        "You {} the area and find {} {}!",
        "search".cyan(),
        format!("{}", gold).yellow().bold(),
        "gold coins".yellow()
    );
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

fn handle_power_surge(
    state: &mut GameState,
    rng: &mut impl Rng,
    zone: &crate::zones::Zone,
    cmd: &str,
) {
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
    let msg = format!(
        "You summon a {}! It fights by your side briefly. +{} XP",
        creature, xp
    );
    display::print_portal(&msg);
    state.add_journal(JournalEntry::new(EventType::Discovery, msg));
    check_level_up(state, leveled);
}

fn handle_incantation(
    state: &mut GameState,
    rng: &mut impl Rng,
    zone: &crate::zones::Zone,
    cmd: &str,
) {
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
        let msg = format!(
            "Your package install transmutes into: {} (+{} {}) [{}]",
            item.name, item.power, item.slot, item.rarity
        );
        display::print_loot(&msg, &item.rarity);
        state.add_journal(JournalEntry::new(EventType::Loot, msg));
        add_to_inventory(state, item, false);
    } else {
        let xp = rng.gen_range(5..=15);
        let leveled = state.character.gain_xp(xp);
        let msg = format!(
            "The alchemist's cauldron bubbles! Dependencies resolve into power! +{} XP",
            xp
        );
        display::print_craft(&msg);
        state.add_journal(JournalEntry::new(EventType::Craft, msg));
        check_level_up(state, leveled);
    }
}

/// Transmutation: reshaping text/data with sed/awk/jq. Wizard-flavored. Loot or XP.
fn handle_transmute(state: &mut GameState, rng: &mut impl Rng, zone: &crate::zones::Zone, cmd: &str) {
    if rng.gen_ratio(1, 3) {
        let item = roll_loot(zone.danger_level);
        let msg = format!(
            "Your transmutation reshapes the stream into: {} (+{} {}) [{}]",
            item.name, item.power, item.slot, item.rarity
        );
        display::print_loot(&msg, &item.rarity);
        state.add_journal(JournalEntry::new(EventType::Loot, msg));
        add_to_inventory(state, item, false);
    } else {
        let base_xp = rng.gen_range(8..=16);
        let xp = final_xp(base_xp, zone.danger_level, &state.character.class, cmd);
        let leveled = state.character.gain_xp(xp);
        let tool = cmd.split_whitespace().next().unwrap_or("sed");
        let (plain, colored) = crate::messages::transmute(&state.character.class, tool, xp);
        display::print_discovery(&colored);
        state.add_journal(JournalEntry::new(EventType::Discovery, plain));
        check_level_up(state, leveled);
    }
}

/// Commune with your past via `history`. Wizard-flavored. XP.
fn handle_commune(state: &mut GameState, rng: &mut impl Rng, zone: &crate::zones::Zone, cmd: &str) {
    let base_xp = rng.gen_range(10..=22);
    let xp = final_xp(base_xp, zone.danger_level, &state.character.class, cmd);
    let leveled = state.character.gain_xp(xp);
    let (plain, colored) = crate::messages::commune(&state.character.class, xp);
    display::print_discovery(&colored);
    state.add_journal(JournalEntry::new(EventType::Discovery, plain));
    check_level_up(state, leveled);
}

/// Command a daemon (systemctl/service). Necromancer-flavored. Combat + gold.
fn handle_command_daemon(
    state: &mut GameState,
    rng: &mut impl Rng,
    zone: &crate::zones::Zone,
    cmd: &str,
) {
    let base_xp = rng.gen_range(15..=25);
    let xp = final_xp(base_xp, zone.danger_level, &state.character.class, cmd);
    let gold = rng.gen_range(3..=10);
    let leveled = state.character.gain_xp(xp);
    state.character.gold += gold;
    let drained = state.character.signature_on_kill();
    let (plain, colored) = crate::messages::command_daemon(&state.character.class, xp, gold);
    display::print_quest(&colored);
    state.add_journal(JournalEntry::new(EventType::Quest, plain));
    if drained > 0 {
        display::print_soul_drain(drained, state.character.hp, state.character.max_hp);
    }
    check_level_up(state, leveled);
}

/// Raw disk power (dd/wipe): a gamble. Big loot, or it bites back. Necromancer-flavored.
fn handle_raw_power(state: &mut GameState, rng: &mut impl Rng, cwd: &str) {
    let zone = zone_from_path(cwd);
    if rng.gen_ratio(1, 2) {
        // The power answers: rare-tier loot from raw disk blocks.
        let item = roll_loot(zone.danger_level + 1);
        let item_desc = format!(
            "{} (+{} {}) [{}]",
            item.name, item.power, item.slot, item.rarity
        );
        let (plain, colored) =
            crate::messages::raw_power_loot(&state.character.class, &item_desc);
        display::print_loot(&colored, &item.rarity);
        state.add_journal(JournalEntry::new(EventType::Loot, plain));
        add_to_inventory(state, item, false);
    } else {
        // It bites back: 2-5% max-hp self-damage (never lethal here).
        let dmg = ((state.character.max_hp as f32 * rng.gen_range(0.02..0.05)).round() as i32).max(1);
        let dmg = dmg.min(state.character.hp - 1).max(0);
        let _ = state.character.take_damage(dmg);
        let (plain, colored) = crate::messages::raw_power_backfire(
            &state.character.class,
            dmg,
            state.character.hp,
            state.character.max_hp,
        );
        display::print_trap(&colored);
        state.add_journal(JournalEntry::new(EventType::Combat, plain));
    }
}

/// Cleanse the screen (clear/reset): a small restorative breath. Neutral.
fn handle_cleanse(state: &mut GameState, rng: &mut impl Rng) {
    let heal = rng.gen_range(1..=3);
    state.character.heal(heal);
    let (plain, colored) = crate::messages::cleanse(
        &state.character.class,
        heal,
        state.character.hp,
        state.character.max_hp,
    );
    display::print_familiar(&colored);
    state.add_journal(JournalEntry::new(EventType::Discovery, plain));
}

fn handle_meditation(
    state: &mut GameState,
    rng: &mut impl Rng,
    zone: &crate::zones::Zone,
    cmd: &str,
) {
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
        let msg = format!(
            "Your search reveals a hidden treasure: {} (+{} {}) [{}]",
            item.name, item.power, item.slot, item.rarity
        );
        display::print_loot(&msg, &item.rarity);
        state.add_journal(JournalEntry::new(EventType::Loot, msg));
        add_to_inventory(state, item, false);
    } else {
        let xp = rng.gen_range(8..=16);
        let leveled = state.character.gain_xp(xp);
        let msg = format!(
            "Your scrying reveals hidden patterns in the codebase! +{} XP",
            xp
        );
        display::print_discovery(&msg);
        state.add_journal(JournalEntry::new(EventType::Discovery, msg));
        check_level_up(state, leveled);
    }
}

fn handle_trial(state: &mut GameState, rng: &mut impl Rng, zone: &crate::zones::Zone, cmd: &str) {
    let base_xp = rng.gen_range(12..=25);
    let xp = final_xp(base_xp, zone.danger_level, &state.character.class, cmd);
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

fn handle_telekinesis(
    state: &mut GameState,
    rng: &mut impl Rng,
    zone: &crate::zones::Zone,
    cmd: &str,
) {
    let base_xp = rng.gen_range(5..=12);
    let xp = final_xp(base_xp, zone.danger_level, &state.character.class, cmd);
    let leveled = state.character.gain_xp(xp);
    let msg = "You move files with the power of your mind! Bytes rearrange at your command!";
    let full_msg = format!("{} +{} XP", msg, xp);
    display::print_discovery(&full_msg);
    state.add_journal(JournalEntry::new(EventType::Discovery, full_msg));
    check_level_up(state, leveled);
}

fn handle_enchant(state: &mut GameState, rng: &mut impl Rng, zone: &crate::zones::Zone, cmd: &str) {
    let base_xp = rng.gen_range(10..=20);
    let xp = final_xp(base_xp, zone.danger_level, &state.character.class, cmd);
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

fn handle_omniscience(
    state: &mut GameState,
    rng: &mut impl Rng,
    zone: &crate::zones::Zone,
    cmd: &str,
) {
    let base_xp = rng.gen_range(5..=10);
    let xp = final_xp(base_xp, zone.danger_level, &state.character.class, cmd);
    let leveled = state.character.gain_xp(xp);
    let msg = format!(
        "You peer into the process table... all running spirits are revealed to you! +{} XP",
        xp
    );
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
    register_kill(state);
    let targets = ["rogue process", "runaway daemon", "zombie worker"];
    let target = targets[rng.gen_range(0..targets.len())];
    let (plain, colored) = crate::messages::banish(&state.character.class, target, xp, gold);
    display::print_combat_win(&colored);
    state.add_journal(JournalEntry::new(EventType::Combat, plain));
    check_level_up(state, leveled);
}

fn handle_treasure_chest(state: &mut GameState, _rng: &mut impl Rng, cwd: &str) {
    let zone = zone_from_path(cwd);
    let item = roll_loot(zone.danger_level + 1);
    let msg = format!(
        "You crack open an archive! Inside you find: {} (+{} {}) [{}]",
        item.name, item.power, item.slot, item.rarity
    );
    display::print_loot(&msg, &item.rarity);
    state.add_journal(JournalEntry::new(EventType::Loot, msg));
    add_to_inventory(state, item, false);
}

fn handle_echo_spell(state: &mut GameState, rng: &mut impl Rng) {
    let heal = rng.gen_range(1..=3);
    state.character.heal(heal);
    let msg = format!(
        "Your words echo through the terminal void... the resonance heals you! +{} HP",
        heal
    );
    display::print_familiar(&msg);
    state.add_journal(JournalEntry::new(EventType::Discovery, msg));
}

fn handle_ancient_tome(
    state: &mut GameState,
    rng: &mut impl Rng,
    zone: &crate::zones::Zone,
    cmd: &str,
) {
    let base_xp = rng.gen_range(10..=22);
    let xp = final_xp(base_xp, zone.danger_level, &state.character.class, cmd);
    let leveled = state.character.gain_xp(xp);
    let subject = cmd.split_whitespace().next().unwrap_or("manual");
    let (plain, colored) = crate::messages::ancient_tome(&state.character.class, subject, xp);
    display::print_discovery(&colored);
    state.add_journal(JournalEntry::new(EventType::Discovery, plain));
    check_level_up(state, leveled);
}

fn handle_container_forge(state: &mut GameState, rng: &mut impl Rng, cwd: &str, cmd: &str) {
    let zone = zone_from_path(cwd);
    if rng.gen_ratio(1, 2) {
        let item = roll_loot(zone.danger_level + 1);
        let msg = format!(
            "The container forge blazes! Layers fuse into: {} (+{} {}) [{}]",
            item.name, item.power, item.slot, item.rarity
        );
        display::print_loot(&msg, &item.rarity);
        state.add_journal(JournalEntry::new(EventType::Craft, msg));
        add_to_inventory(state, item, false);
    } else {
        let base_xp = rng.gen_range(12..=25);
        let xp = final_xp(base_xp, zone.danger_level, &state.character.class, cmd);
        let leveled = state.character.gain_xp(xp);
        let msg = format!(
            "The image builds layer by layer! Each instruction tempers your resolve! +{} XP",
            xp
        );
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
    register_kill(state);
    let msgs = [
        "You banish the container to the void! Its resources return to the host!",
        "SIGTERM! The container dissolves into freed memory!",
        "You prune the fallen container! Its ephemeral storage scatters!",
    ];
    let msg = format!(
        "{} +{} XP, +{} gold",
        msgs[rng.gen_range(0..msgs.len())],
        xp,
        gold
    );
    display::print_combat_win(&msg);
    state.add_journal(JournalEntry::new(EventType::Combat, msg));
    check_level_up(state, leveled);
}

fn handle_docker_orchestra(
    state: &mut GameState,
    rng: &mut impl Rng,
    zone: &crate::zones::Zone,
    cmd: &str,
) {
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

fn handle_random_encounter(
    state: &mut GameState,
    rng: &mut impl Rng,
    zone: &crate::zones::Zone,
    cmd: &str,
) {
    let roll: u32 = rng.gen_range(1..=100);

    match roll {
        1..=40 => {
            let (name, base_atk, base_hp, base_xp, tier) = random_monster_for_zone(rng, zone);
            let enemy_dex_mod = tier_dex_mod(tier) + state.character.total_prestiges as i32;
            let profile = if rng.gen_ratio(1, 8) {
                apply_elite_pressure(&name, base_atk, base_hp, base_xp, zone.danger_level)
            } else {
                EncounterProfile {
                    name,
                    attack: base_atk,
                    hp: base_hp,
                    xp: base_xp,
                    elite: false,
                }
            };
            combat(
                state,
                rng,
                zone,
                cmd,
                &profile.name,
                profile.attack,
                profile.hp,
                profile.xp,
                profile.elite,
                enemy_dex_mod,
            );
        }
        41..=65 => {
            // Find loot
            let item = roll_loot(zone.danger_level);
            let msg = format!(
                "You found: {} (+{} {}) [{}]",
                item.name, item.power, item.slot, item.rarity
            );
            display::print_loot(&msg, &item.rarity);
            state.add_journal(JournalEntry::new(EventType::Loot, msg));
            add_to_inventory(state, item, false);
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
            let msg = format!(
                "You find a quiet spot to rest. +{} HP. HP: {}/{}",
                heal, state.character.hp, state.character.max_hp
            );
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

fn register_kill(state: &mut GameState) {
    state.character.kills += 1;
    let drained = state.character.signature_on_kill();
    if drained > 0 {
        crate::display::print_soul_drain(drained, state.character.hp, state.character.max_hp);
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

struct EncounterProfile {
    name: String,
    attack: i32,
    hp: i32,
    xp: u32,
    elite: bool,
}

pub const ELITE_ATTACK_BASE_MULT: f64 = 1.6;
pub const ELITE_ATTACK_PER_DANGER: f64 = 0.15;
pub const ELITE_HP_MULT: f64 = 1.5;
pub const ELITE_XP_MULT: f64 = 2.0;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct EliteModifiers {
    pub attack_base_mult: f64,
    pub attack_per_danger: f64,
    pub hp_mult: f64,
    pub xp_mult: f64,
}

pub fn elite_modifiers() -> EliteModifiers {
    EliteModifiers {
        attack_base_mult: ELITE_ATTACK_BASE_MULT,
        attack_per_danger: ELITE_ATTACK_PER_DANGER,
        hp_mult: ELITE_HP_MULT,
        xp_mult: ELITE_XP_MULT,
    }
}

fn apply_elite_pressure(
    name: &str,
    base_attack: i32,
    base_hp: i32,
    base_xp: u32,
    danger: u32,
) -> EncounterProfile {
    let attack_multiplier = (ELITE_ATTACK_BASE_MULT as f32)
        * (1.0 + (danger.saturating_sub(1) as f32) * (ELITE_ATTACK_PER_DANGER as f32));

    EncounterProfile {
        name: format!("Enraged {}", name),
        attack: ((base_attack as f32) * attack_multiplier).round() as i32,
        hp: ((base_hp as f32) * (ELITE_HP_MULT as f32)).round() as i32,
        xp: ((base_xp as f32) * (ELITE_XP_MULT as f32)).round() as u32,
        elite: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MonsterTier {
    Vermin,
    Bruiser,
    Hunter,
    Horror,
    BossAdjacent,
}

impl MonsterTier {
    pub fn as_str(self) -> &'static str {
        match self {
            MonsterTier::Vermin => "Vermin",
            MonsterTier::Bruiser => "Bruiser",
            MonsterTier::Hunter => "Hunter",
            MonsterTier::Horror => "Horror",
            MonsterTier::BossAdjacent => "BossAdjacent",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct MonsterEntry {
    pub name: &'static str,
    pub tier: MonsterTier,
    pub hp: i32,
    pub attack: i32,
    pub xp: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonsterInfo {
    pub name: String,
    pub tier: String,
    pub hp: i32,
    pub attack: i32,
    pub xp: u32,
}

pub const MONSTER_POOL: &[MonsterEntry] = &[
    MonsterEntry {
        name: "Symlink Slug",
        tier: MonsterTier::Vermin,
        hp: 1,
        attack: 2,
        xp: 4,
    },
    MonsterEntry {
        name: "Dotfile Dust-Mite",
        tier: MonsterTier::Vermin,
        hp: 2,
        attack: 2,
        xp: 5,
    },
    MonsterEntry {
        name: "Cache Cricket",
        tier: MonsterTier::Vermin,
        hp: 1,
        attack: 3,
        xp: 4,
    },
    MonsterEntry {
        name: "Tempfile Tadpole",
        tier: MonsterTier::Vermin,
        hp: 2,
        attack: 3,
        xp: 6,
    },
    MonsterEntry {
        name: "Log-File Larva",
        tier: MonsterTier::Vermin,
        hp: 3,
        attack: 2,
        xp: 7,
    },
    MonsterEntry {
        name: "Socket Snail",
        tier: MonsterTier::Vermin,
        hp: 2,
        attack: 4,
        xp: 6,
    },
    MonsterEntry {
        name: "Inode Inchworm",
        tier: MonsterTier::Vermin,
        hp: 1,
        attack: 4,
        xp: 5,
    },
    MonsterEntry {
        name: "Permission Pupa",
        tier: MonsterTier::Vermin,
        hp: 3,
        attack: 3,
        xp: 8,
    },
    MonsterEntry {
        name: "Zombie Process Zealot",
        tier: MonsterTier::Bruiser,
        hp: 10,
        attack: 5,
        xp: 14,
    },
    MonsterEntry {
        name: "Background Job Brawler",
        tier: MonsterTier::Bruiser,
        hp: 8,
        attack: 6,
        xp: 12,
    },
    MonsterEntry {
        name: "Daemon Duelist",
        tier: MonsterTier::Bruiser,
        hp: 12,
        attack: 5,
        xp: 16,
    },
    MonsterEntry {
        name: "Shell-Script Shaman",
        tier: MonsterTier::Bruiser,
        hp: 9,
        attack: 7,
        xp: 15,
    },
    MonsterEntry {
        name: "Pipe-Line Pugilist",
        tier: MonsterTier::Bruiser,
        hp: 11,
        attack: 6,
        xp: 17,
    },
    MonsterEntry {
        name: "Env-Var Vandal",
        tier: MonsterTier::Bruiser,
        hp: 10,
        attack: 8,
        xp: 18,
    },
    MonsterEntry {
        name: "Cron-Job Crusader",
        tier: MonsterTier::Bruiser,
        hp: 14,
        attack: 5,
        xp: 19,
    },
    MonsterEntry {
        name: "Std-Error Specter",
        tier: MonsterTier::Bruiser,
        hp: 15,
        attack: 7,
        xp: 20,
    },
    MonsterEntry {
        name: "Heap-Alloc Hound",
        tier: MonsterTier::Hunter,
        hp: 30,
        attack: 9,
        xp: 25,
    },
    MonsterEntry {
        name: "Stack-Trace Stalker",
        tier: MonsterTier::Hunter,
        hp: 28,
        attack: 10,
        xp: 24,
    },
    MonsterEntry {
        name: "Pointer Panther",
        tier: MonsterTier::Hunter,
        hp: 32,
        attack: 11,
        xp: 28,
    },
    MonsterEntry {
        name: "Overflow Basilisk",
        tier: MonsterTier::Hunter,
        hp: 35,
        attack: 12,
        xp: 30,
    },
    MonsterEntry {
        name: "Garbage-Collector Gryphon",
        tier: MonsterTier::Hunter,
        hp: 40,
        attack: 8,
        xp: 32,
    },
    MonsterEntry {
        name: "Memory-Leak Manticore",
        tier: MonsterTier::Hunter,
        hp: 38,
        attack: 13,
        xp: 34,
    },
    MonsterEntry {
        name: "Segfault Shark",
        tier: MonsterTier::Hunter,
        hp: 36,
        attack: 14,
        xp: 33,
    },
    MonsterEntry {
        name: "Dangling-Pointer Dingo",
        tier: MonsterTier::Hunter,
        hp: 25,
        attack: 14,
        xp: 22,
    },
    MonsterEntry {
        name: "Kernel-Panic Kraken",
        tier: MonsterTier::Horror,
        hp: 90,
        attack: 18,
        xp: 60,
    },
    MonsterEntry {
        name: "Interrupt-Handler Hydra",
        tier: MonsterTier::Horror,
        hp: 80,
        attack: 20,
        xp: 55,
    },
    MonsterEntry {
        name: "System-Call Siren",
        tier: MonsterTier::Horror,
        hp: 70,
        attack: 22,
        xp: 50,
    },
    MonsterEntry {
        name: "Page-Fault Phantom",
        tier: MonsterTier::Horror,
        hp: 65,
        attack: 21,
        xp: 45,
    },
    MonsterEntry {
        name: "Deadlock Dragon",
        tier: MonsterTier::Horror,
        hp: 100,
        attack: 14,
        xp: 65,
    },
    MonsterEntry {
        name: "Race-Condition Reaper",
        tier: MonsterTier::Horror,
        hp: 60,
        attack: 22,
        xp: 40,
    },
    MonsterEntry {
        name: "Root-Kit Revenant",
        tier: MonsterTier::Horror,
        hp: 75,
        attack: 19,
        xp: 52,
    },
    MonsterEntry {
        name: "Scheduler-Shadow Stalker",
        tier: MonsterTier::Horror,
        hp: 85,
        attack: 17,
        xp: 58,
    },
    MonsterEntry {
        name: "The Monolith Manifestation",
        tier: MonsterTier::BossAdjacent,
        hp: 200,
        attack: 25,
        xp: 110,
    },
    MonsterEntry {
        name: "Architectural-Debt Archon",
        tier: MonsterTier::BossAdjacent,
        hp: 180,
        attack: 28,
        xp: 100,
    },
    MonsterEntry {
        name: "The Legacy-Code Lich",
        tier: MonsterTier::BossAdjacent,
        hp: 220,
        attack: 22,
        xp: 120,
    },
    MonsterEntry {
        name: "Distributed-System Djinn",
        tier: MonsterTier::BossAdjacent,
        hp: 170,
        attack: 30,
        xp: 95,
    },
    MonsterEntry {
        name: "The Microservice Medusa",
        tier: MonsterTier::BossAdjacent,
        hp: 160,
        attack: 29,
        xp: 90,
    },
    MonsterEntry {
        name: "Cloud-Native Chimera",
        tier: MonsterTier::BossAdjacent,
        hp: 190,
        attack: 26,
        xp: 105,
    },
    MonsterEntry {
        name: "The Immutable-State Idol",
        tier: MonsterTier::BossAdjacent,
        hp: 210,
        attack: 24,
        xp: 115,
    },
    MonsterEntry {
        name: "Entropy-Engine Elemental",
        tier: MonsterTier::BossAdjacent,
        hp: 150,
        attack: 30,
        xp: 80,
    },
];

pub fn tiers_for_danger(danger_level: u32) -> &'static [MonsterTier] {
    match danger_level {
        1 => &[MonsterTier::Vermin],
        2 => &[MonsterTier::Vermin, MonsterTier::Bruiser],
        3 => &[MonsterTier::Bruiser, MonsterTier::Hunter],
        4 => &[MonsterTier::Hunter, MonsterTier::Horror],
        _ => &[MonsterTier::Horror, MonsterTier::BossAdjacent],
    }
}

pub fn monster_bestiary() -> Vec<MonsterInfo> {
    MONSTER_POOL
        .iter()
        .map(|monster| MonsterInfo {
            name: monster.name.to_string(),
            tier: monster.tier.as_str().to_string(),
            hp: monster.hp,
            attack: monster.attack,
            xp: monster.xp,
        })
        .collect()
}

pub fn monster_tier_order() -> Vec<&'static str> {
    vec![
        MonsterTier::Vermin.as_str(),
        MonsterTier::Bruiser.as_str(),
        MonsterTier::Hunter.as_str(),
        MonsterTier::Horror.as_str(),
        MonsterTier::BossAdjacent.as_str(),
    ]
}

pub fn tier_danger() -> Vec<(u32, Vec<&'static str>)> {
    (1..=5)
        .map(|danger| {
            let tiers = tiers_for_danger(danger)
                .iter()
                .map(|tier| tier.as_str())
                .collect();
            (danger, tiers)
        })
        .collect()
}

pub fn tier_dex_mod(tier: MonsterTier) -> i32 {
    match tier {
        MonsterTier::Vermin => 0,
        MonsterTier::Bruiser => 2,
        MonsterTier::Hunter => 4,
        MonsterTier::Horror => 6,
        MonsterTier::BossAdjacent => 8,
    }
}

fn random_monster_in_tiers(
    rng: &mut impl Rng,
    allowed: &[MonsterTier],
) -> Option<&'static MonsterEntry> {
    let pool: Vec<&MonsterEntry> = MONSTER_POOL
        .iter()
        .filter(|m| allowed.contains(&m.tier))
        .collect();
    if pool.is_empty() {
        None
    } else {
        Some(pool[rng.gen_range(0..pool.len())])
    }
}

fn random_monster_for_zone(
    rng: &mut impl Rng,
    zone: &crate::zones::Zone,
) -> (String, i32, i32, u32, MonsterTier) {
    let entry = random_monster_in_tiers(rng, tiers_for_danger(zone.danger_level))
        .or_else(|| {
            random_monster_in_tiers(
                rng,
                &[
                    MonsterTier::Hunter,
                    MonsterTier::Bruiser,
                    MonsterTier::Vermin,
                ],
            )
        })
        .expect("MONSTER_POOL must contain at least one entry");
    let scale = encounter_scale_for_danger(zone.danger_level);
    let atk = ((entry.attack as f32 * scale).round() as i32).max(1);
    let xp = ((entry.xp as f32 * scale).round() as u32).max(5);
    (entry.name.to_string(), atk, entry.hp, xp, entry.tier)
}

const COMBAT_MAX_TURNS: u32 = 30;

fn combat(
    state: &mut GameState,
    rng: &mut impl Rng,
    zone: &crate::zones::Zone,
    cmd: &str,
    monster_name: &str,
    monster_atk: i32,
    monster_hp_initial: i32,
    xp_reward: u32,
    is_elite: bool,
    enemy_dex_mod: i32,
) {
    let mut monster_hp = monster_hp_initial.max(1);
    let mut total_damage_taken: i32 = 0;
    let mut total_damage_dealt: i32 = 0;
    let mut first_strike = true;
    let final_reward = final_xp(xp_reward, zone.danger_level, &state.character.class, cmd);
    let player_class = state.character.class.clone();

    for _turn in 1..=COMBAT_MAX_TURNS {
        let player_power = state.character.attack_power();
        let player_int = state.character.intelligence;
        let player_str = state.character.strength;
        let player_hp_now = state.character.hp;
        let player_max_hp = state.character.max_hp;

        let hit_roll: i32 = rng.gen_range(1..=20);
        let player_hits = match player_class {
            crate::character::Class::Rogue => hit_roll + player_power > 10 || hit_roll == 1,
            _ => hit_roll + player_power > 10,
        };

        if player_hits {
            let mut raw_dmg = rng.gen_range((player_power / 2).max(1)..=player_power.max(1));
            let (sig_bonus, _sig_label) = crate::character::signature_bonus(
                &player_class,
                player_int,
                player_str,
                player_hp_now,
                player_max_hp,
                first_strike,
            );
            raw_dmg += sig_bonus;
            let crit_threshold = (20 - player_int / 8).max(13);
            if hit_roll >= crit_threshold {
                raw_dmg *= 2;
            }
            monster_hp -= raw_dmg;
            total_damage_dealt += raw_dmg;
            first_strike = false;

            if monster_hp <= 0 {
                register_kill(state);
                let leveled = state.character.gain_xp(final_reward);
                state.character.signature_on_kill();
                let (plain, colored) = if total_damage_taken > 0 {
                    if is_elite {
                        crate::messages::combat_elite_tough(
                            &player_class,
                            monster_name,
                            total_damage_taken,
                            final_reward,
                        )
                    } else {
                        crate::messages::combat_tough(
                            &player_class,
                            monster_name,
                            total_damage_taken,
                            final_reward,
                        )
                    }
                } else if is_elite {
                    crate::messages::combat_elite_win(&player_class, monster_name, final_reward)
                } else {
                    crate::messages::combat_win(&player_class, monster_name, final_reward)
                };
                if total_damage_taken > 0 {
                    display::print_combat_tough(&colored, false);
                } else {
                    display::print_combat_win(&colored);
                }
                state.add_journal(JournalEntry::new(EventType::Combat, plain));
                check_level_up(state, leveled);
                crate::telemetry::emit_encounter(
                    "mob",
                    monster_name,
                    is_elite,
                    total_damage_dealt,
                    total_damage_taken,
                    "kill",
                    final_reward,
                    0,
                );
                return;
            }
        }

        let player_defense = state.character.defense();
        let dodge_roll: i32 = rng.gen_range(1..=20);
        if crate::character::attack_lands(dodge_roll, enemy_dex_mod, state.character.dex_mod()) {
            let damage = (monster_atk - player_defense / 3).max(1);
            total_damage_taken += damage;
            let gold_before = state.character.gold;
            let died = state.character.take_damage(damage);
            if died {
                if state.permadeath {
                    crate::display::print_permadeath_eulogy(&state.character, monster_name);
                    crate::telemetry::emit_encounter(
                        "mob",
                        monster_name,
                        is_elite,
                        total_damage_dealt,
                        total_damage_taken,
                        "death",
                        0,
                        0,
                    );
                    let path = crate::state::save_path();
                    let _ = std::fs::remove_file(&path);
                    std::process::exit(0);
                }
                state.character.die();
                let gold_loss = gold_before * 15 / 100;
                let (plain, colored) =
                    crate::messages::death_normal(&player_class, monster_name, gold_loss);
                display::print_combat_lose(&colored, true);
                state.add_journal(JournalEntry::new(EventType::Death, plain));
                crate::telemetry::emit_encounter(
                    "mob",
                    monster_name,
                    is_elite,
                    total_damage_dealt,
                    total_damage_taken,
                    "death",
                    0,
                    0,
                );
                return;
            }
        }
    }

    let (plain, colored) = crate::messages::combat_draw(&state.character.class, monster_name);
    display::print_combat_draw(&colored);
    state.add_journal(JournalEntry::new(EventType::Combat, plain));
    crate::telemetry::emit_encounter(
        "mob",
        monster_name,
        is_elite,
        total_damage_dealt,
        total_damage_taken,
        "draw",
        0,
        0,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::{Character, Class, Item, ItemSlot, Race, Rarity};
    use crate::state::GameState;

    fn make_state() -> GameState {
        GameState::new(Character::new(
            "Test".to_string(),
            Class::Warrior,
            Race::Human,
        ))
    }

    fn make_item(name: &str, slot: ItemSlot, power: i32, rarity: Rarity) -> Item {
        Item {
            name: name.to_string(),
            slot,
            power,
            rarity,
            enchant_level: 0,
        }
    }

    #[test]
    fn tiers_for_danger_one_is_vermin_only() {
        assert_eq!(tiers_for_danger(1), &[MonsterTier::Vermin]);
    }

    #[test]
    fn tiers_for_danger_three_includes_bruiser_and_hunter_not_vermin() {
        let tiers = tiers_for_danger(3);
        assert!(tiers.contains(&MonsterTier::Bruiser));
        assert!(tiers.contains(&MonsterTier::Hunter));
        assert!(!tiers.contains(&MonsterTier::Vermin));
    }

    #[test]
    fn tiers_for_danger_five_targets_horror_and_boss_adjacent() {
        let tiers = tiers_for_danger(5);
        assert!(tiers.contains(&MonsterTier::Horror));
        assert!(tiers.contains(&MonsterTier::BossAdjacent));
    }

    #[test]
    fn monster_pool_has_all_three_currently_populated_tiers() {
        let has_vermin = MONSTER_POOL
            .iter()
            .any(|m| matches!(m.tier, MonsterTier::Vermin));
        let has_bruiser = MONSTER_POOL
            .iter()
            .any(|m| matches!(m.tier, MonsterTier::Bruiser));
        let has_hunter = MONSTER_POOL
            .iter()
            .any(|m| matches!(m.tier, MonsterTier::Hunter));
        assert!(
            has_vermin,
            "MONSTER_POOL must contain at least one Vermin entry"
        );
        assert!(
            has_bruiser,
            "MONSTER_POOL must contain at least one Bruiser entry"
        );
        assert!(
            has_hunter,
            "MONSTER_POOL must contain at least one Hunter entry"
        );
    }

    #[test]
    fn random_monster_in_tiers_vermin_only_returns_vermin_entry() {
        let mut rng = rand::thread_rng();
        for _ in 0..50 {
            let entry = random_monster_in_tiers(&mut rng, &[MonsterTier::Vermin])
                .expect("Vermin pool must be non-empty");
            assert_eq!(entry.tier, MonsterTier::Vermin);
        }
    }

    #[test]
    fn random_monster_in_tiers_empty_allow_list_returns_none() {
        let mut rng = rand::thread_rng();
        let result = random_monster_in_tiers(&mut rng, &[]);
        assert!(
            result.is_none(),
            "empty allow-list must produce None for the caller to fall back"
        );
    }

    #[test]
    fn monster_pool_populates_all_five_tiers() {
        for tier in [
            MonsterTier::Vermin,
            MonsterTier::Bruiser,
            MonsterTier::Hunter,
            MonsterTier::Horror,
            MonsterTier::BossAdjacent,
        ] {
            let count = MONSTER_POOL.iter().filter(|m| m.tier == tier).count();
            assert!(
                count >= 1,
                "tier {:?} must have at least one entry (v1.22+ contract)",
                tier
            );
        }
    }

    #[test]
    fn monster_bestiary_has_forty_positive_entries_and_eight_per_tier() {
        let monsters = monster_bestiary();

        assert_eq!(monsters.len(), 40);
        for monster in &monsters {
            assert!(!monster.name.is_empty());
            assert!(monster.hp > 0);
            assert!(monster.attack > 0);
            assert!(monster.xp > 0);
        }

        for tier in ["Vermin", "Bruiser", "Hunter", "Horror", "BossAdjacent"] {
            let count = monsters
                .iter()
                .filter(|monster| monster.tier == tier)
                .count();
            assert_eq!(count, 8, "expected 8 {tier} monsters");
        }
    }

    #[test]
    fn random_monster_for_zone_danger_one_only_spawns_vermin() {
        let zone = crate::zones::Zone {
            name: "Test Home",
            description: "test",
            danger_level: 1,
            color: crate::zones::ZoneColor::Green,
        };
        let mut rng = rand::thread_rng();
        let vermin_attacks: Vec<i32> = MONSTER_POOL
            .iter()
            .filter(|m| matches!(m.tier, MonsterTier::Vermin))
            .map(|m| ((m.attack as f32 * encounter_scale_for_danger(1)).round() as i32).max(1))
            .collect();
        for _ in 0..100 {
            let (_name, atk, _hp, _xp, _tier) = random_monster_for_zone(&mut rng, &zone);
            assert!(
                vermin_attacks.contains(&atk),
                "danger-1 zone must only produce Vermin-tier monster ATK (got {})",
                atk
            );
        }
    }

    #[test]
    fn random_monster_for_zone_danger_five_falls_back_when_no_horror_yet() {
        let zone = crate::zones::Zone {
            name: "Test Abyss",
            description: "test",
            danger_level: 5,
            color: crate::zones::ZoneColor::Red,
        };
        let mut rng = rand::thread_rng();
        for _ in 0..50 {
            let (_name, atk, hp, _xp, _tier) = random_monster_for_zone(&mut rng, &zone);
            assert!(atk >= 1, "fallback must still produce a valid monster");
            assert!(hp >= 1, "fallback monster must have positive HP");
        }
    }

    #[test]
    fn trap_damage_at_max_hp_30_returns_one_or_two() {
        let mut rng = rand::thread_rng();
        for _ in 0..200 {
            let dmg = trap_damage(30, &mut rng);
            assert!(dmg >= 1 && dmg <= 2, "L1 trap dmg out of band: {}", dmg);
        }
    }

    #[test]
    fn trap_damage_at_max_hp_555_returns_band_17_to_33() {
        let mut rng = rand::thread_rng();
        for _ in 0..200 {
            let dmg = trap_damage(555, &mut rng);
            assert!(dmg >= 17 && dmg <= 34, "L100 trap dmg out of band: {}", dmg);
        }
    }

    #[test]
    fn trap_damage_always_at_least_one() {
        let mut rng = rand::thread_rng();
        for max in [1, 5, 10, 20, 30] {
            for _ in 0..50 {
                let dmg = trap_damage(max, &mut rng);
                assert!(dmg >= 1, "trap dmg must be >= 1 even at low HP {}", max);
            }
        }
    }

    #[test]
    fn add_to_inventory_adds_item_when_space_available() {
        let mut state = make_state();
        add_to_inventory(
            &mut state,
            make_item("Sword", ItemSlot::Weapon, 5, Rarity::Common),
            false,
        );
        assert_eq!(state.character.inventory.len(), 1);
        assert_eq!(state.character.inventory[0].name, "Sword");
    }

    #[test]
    fn add_to_inventory_drops_weakest_droppable_when_full() {
        let mut state = make_state();
        for i in 0..20 {
            state.character.inventory.push(make_item(
                &format!("Common {}", i),
                ItemSlot::Weapon,
                i as i32 + 1,
                Rarity::Common,
            ));
        }
        add_to_inventory(
            &mut state,
            make_item("New Sword", ItemSlot::Weapon, 99, Rarity::Rare),
            false,
        );
        assert_eq!(state.character.inventory.len(), 20);
        assert!(state
            .character
            .inventory
            .iter()
            .any(|i| i.name == "New Sword"));
        assert!(!state
            .character
            .inventory
            .iter()
            .any(|i| i.name == "Common 0"));
    }

    #[test]
    fn add_to_inventory_does_not_drop_epics_when_full() {
        let mut state = make_state();
        for i in 0..20 {
            state.character.inventory.push(make_item(
                &format!("Epic {}", i),
                ItemSlot::Weapon,
                i as i32 + 1,
                Rarity::Epic,
            ));
        }
        add_to_inventory(
            &mut state,
            make_item("New Sword", ItemSlot::Weapon, 5, Rarity::Common),
            false,
        );
        assert_eq!(state.character.inventory.len(), 20);
        assert!(!state
            .character
            .inventory
            .iter()
            .any(|i| i.name == "New Sword"));
        assert_eq!(
            state
                .character
                .inventory
                .iter()
                .filter(|i| matches!(i.rarity, Rarity::Epic))
                .count(),
            20
        );
    }

    #[test]
    fn add_to_inventory_does_not_drop_legendaries_when_full() {
        let mut state = make_state();
        for i in 0..20 {
            state.character.inventory.push(make_item(
                &format!("Legendary {}", i),
                ItemSlot::Weapon,
                i as i32 + 1,
                Rarity::Legendary,
            ));
        }
        add_to_inventory(
            &mut state,
            make_item("Common Sword", ItemSlot::Weapon, 5, Rarity::Common),
            false,
        );
        assert_eq!(state.character.inventory.len(), 20);
        assert!(!state
            .character
            .inventory
            .iter()
            .any(|i| i.name == "Common Sword"));
        assert_eq!(
            state
                .character
                .inventory
                .iter()
                .filter(|i| matches!(i.rarity, Rarity::Legendary))
                .count(),
            20
        );
    }

    #[test]
    fn add_to_inventory_drops_weakest_droppable_from_mixed_inventory() {
        let mut state = make_state();
        for i in 0..18 {
            state.character.inventory.push(make_item(
                &format!("Epic {}", i),
                ItemSlot::Weapon,
                50 + i as i32,
                Rarity::Epic,
            ));
        }
        state.character.inventory.push(make_item(
            "Weak Common",
            ItemSlot::Weapon,
            1,
            Rarity::Common,
        ));
        state.character.inventory.push(make_item(
            "Medium Rare",
            ItemSlot::Weapon,
            10,
            Rarity::Rare,
        ));
        add_to_inventory(
            &mut state,
            make_item("New Epic", ItemSlot::Weapon, 99, Rarity::Epic),
            false,
        );
        assert_eq!(state.character.inventory.len(), 20);
        assert!(state
            .character
            .inventory
            .iter()
            .any(|i| i.name == "New Epic"));
        assert!(!state
            .character
            .inventory
            .iter()
            .any(|i| i.name == "Weak Common"));
        assert!(state
            .character
            .inventory
            .iter()
            .any(|i| i.name == "Medium Rare"));
    }

    #[test]
    fn add_to_inventory_drops_rare_before_uncommon_if_rare_is_weaker() {
        let mut state = make_state();
        for i in 0..18 {
            state.character.inventory.push(make_item(
                &format!("Epic {}", i),
                ItemSlot::Weapon,
                50 + i as i32,
                Rarity::Epic,
            ));
        }
        state.character.inventory.push(make_item(
            "Strong Uncommon",
            ItemSlot::Weapon,
            20,
            Rarity::Uncommon,
        ));
        state
            .character
            .inventory
            .push(make_item("Weak Rare", ItemSlot::Weapon, 5, Rarity::Rare));
        add_to_inventory(
            &mut state,
            make_item("New Weapon", ItemSlot::Weapon, 99, Rarity::Legendary),
            false,
        );
        assert_eq!(state.character.inventory.len(), 20);
        assert!(state
            .character
            .inventory
            .iter()
            .any(|i| i.name == "New Weapon"));
        assert!(!state
            .character
            .inventory
            .iter()
            .any(|i| i.name == "Weak Rare"));
        assert!(state
            .character
            .inventory
            .iter()
            .any(|i| i.name == "Strong Uncommon"));
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
    fn affinity_multiplier_new_commands_by_class() {
        use crate::character::Class;
        // New affinity assignments from the expanded command set.
        assert_eq!(affinity_multiplier(&Class::Wizard, "sed"), 1.5);
        assert_eq!(affinity_multiplier(&Class::Wizard, "awk"), 1.5);
        assert_eq!(affinity_multiplier(&Class::Wizard, "history"), 1.5);
        assert_eq!(affinity_multiplier(&Class::Warrior, "rustc"), 1.5);
        assert_eq!(affinity_multiplier(&Class::Warrior, "eslint"), 1.5);
        assert_eq!(affinity_multiplier(&Class::Rogue, "sort"), 1.5);
        assert_eq!(affinity_multiplier(&Class::Rogue, "diff"), 1.5);
        assert_eq!(affinity_multiplier(&Class::Ranger, "ping"), 1.5);
        assert_eq!(affinity_multiplier(&Class::Ranger, "scp"), 1.5);
        assert_eq!(affinity_multiplier(&Class::Necromancer, "systemctl"), 1.5);
        assert_eq!(affinity_multiplier(&Class::Necromancer, "dd"), 1.5);
        // Neutral commands stay 1.0 for everyone.
        assert_eq!(affinity_multiplier(&Class::Wizard, "df"), 1.0);
        assert_eq!(affinity_multiplier(&Class::Rogue, "brew"), 1.0);
        // Cross-class: sed is Wizard's, not Warrior's.
        assert_eq!(affinity_multiplier(&Class::Warrior, "sed"), 1.0);
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
    fn void_mob_scaling_formula_pins_level_and_depth() {
        let low = scale_void_mob(&VOID_MONSTER_ROSTER[0], 1, 1);
        let high = scale_void_mob(&VOID_MONSTER_ROSTER[0], 50, 6);

        assert_eq!(low.hp, 34);
        assert_eq!(low.attack, 8);
        assert_eq!(high.hp, 388);
        assert_eq!(high.attack, 48);
        assert!(high.hp > low.hp);
        assert!(high.attack > low.attack);
    }

    #[test]
    fn void_roster_has_eighteen_distinct_templates() {
        use std::collections::HashSet;

        let names = VOID_MONSTER_ROSTER
            .iter()
            .map(|mob| mob.name)
            .collect::<HashSet<_>>();

        assert_eq!(VOID_MONSTER_ROSTER.len(), 18);
        assert_eq!(names.len(), VOID_MONSTER_ROSTER.len());
        assert!(VOID_MONSTER_ROSTER.iter().any(|mob| mob.base_attack >= 10));
        assert!(VOID_MONSTER_ROSTER.iter().any(|mob| mob.base_hp >= 30));
    }

    #[test]
    fn seeded_tick_in_void_invokes_void_combat() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;

        let mut state = make_state();
        state.character.level = 10;
        state.character.strength = 200;
        state.character.dexterity = 200;
        state.character.max_hp = 1_000;
        state.character.hp = 1_000;
        let mut rng = StdRng::seed_from_u64(2);

        tick_with_rng(
            &mut state,
            "ls",
            "/home/user/.shellquest/the_void/deep/path",
            0,
            &mut rng,
        );

        assert!(state.journal.iter().any(|entry| {
            matches!(entry.event_type, EventType::Combat)
                && VOID_MONSTER_ROSTER
                    .iter()
                    .any(|mob| entry.message.contains(mob.name))
        }));
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
        let elite = apply_elite_pressure("Deadlock Demon", 12, 33, 25, 4);
        assert_eq!(elite.attack, 28);
        assert_eq!(elite.xp, 50);
        assert!(elite.elite);
    }

    #[test]
    fn elite_modifier_prefixes_name() {
        let elite = apply_elite_pressure("Segfault Specter", 8, 14, 15, 3);
        assert!(
            elite.name.starts_with("Enraged "),
            "Expected name to start with 'Enraged ', got: {}",
            elite.name
        );
        assert_eq!(elite.xp, 30);
    }

    #[test]
    fn elite_profile_marks_name_and_reward() {
        let elite = apply_elite_pressure("Segfault Specter", 8, 14, 15, 3);
        assert!(
            elite.name.starts_with("Enraged "),
            "Expected 'Enraged ' prefix, got: {}",
            elite.name
        );
        assert!(elite.xp > 15);
        assert!(elite.elite);
    }

    #[test]
    fn elite_modifier_scales_hp() {
        let elite = apply_elite_pressure("Buffer Overflow Beast", 14, 38, 30, 5);
        assert_eq!(elite.hp, 57, "1.5x of 38 = 57");
    }

    #[test]
    fn tier_dex_mod_scales_with_tier() {
        assert_eq!(tier_dex_mod(MonsterTier::Vermin), 0);
        assert_eq!(tier_dex_mod(MonsterTier::Bruiser), 2);
        assert_eq!(tier_dex_mod(MonsterTier::Hunter), 4);
        assert_eq!(tier_dex_mod(MonsterTier::Horror), 6);
        assert_eq!(tier_dex_mod(MonsterTier::BossAdjacent), 8);
    }

    #[test]
    fn combat_multi_turn_against_high_hp_monster_increments_kill_and_grants_xp() {
        let mut state = make_state();
        state.character.hp = state.character.max_hp;
        let kills_before = state.character.kills;
        let xp_before = state.character.xp;
        let level_before = state.character.level;
        let zone = crate::zones::Zone {
            name: "Test Wasteland",
            description: "test",
            danger_level: 3,
            color: crate::zones::ZoneColor::Red,
        };
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        // Seeded so the multi-turn fight deterministically resolves (kill or
        // player downed) rather than flaking into a 30-turn draw under thread_rng.
        let mut rng = StdRng::seed_from_u64(1);
        combat(
            &mut state,
            &mut rng,
            &zone,
            "rm",
            "Test Hunter",
            12,
            100,
            25,
            false,
            4,
        );
        let killed = state.character.kills > kills_before;
        let leveled_or_died = state.character.level != level_before
            || state.character.hp < state.character.max_hp / 2;
        let gained_xp = state.character.xp > xp_before || state.character.level > level_before;
        assert!(
            killed || leveled_or_died,
            "combat() must resolve in either a kill or a death/severe-damage state, not silently exit"
        );
        if killed {
            assert!(
                gained_xp,
                "killing the monster must award XP (or have level-upped)"
            );
        }
    }

    #[test]
    fn combat_against_vermin_one_hp_resolves_in_one_player_turn_no_damage() {
        let mut state = make_state();
        state.character.hp = state.character.max_hp;
        let initial_hp = state.character.hp;
        let zone = crate::zones::Zone {
            name: "Test Home",
            description: "test",
            danger_level: 1,
            color: crate::zones::ZoneColor::Green,
        };
        let mut rng = rand::thread_rng();
        let mut hits_landing_no_damage = 0;
        for _ in 0..20 {
            state.character.hp = initial_hp;
            combat(
                &mut state,
                &mut rng,
                &zone,
                "rm",
                "Test Vermin",
                2,
                1,
                4,
                false,
                0,
            );
            if state.character.hp == initial_hp {
                hits_landing_no_damage += 1;
            }
        }
        assert!(
            hits_landing_no_damage >= 1,
            "vs 1-HP vermin a level-1 warrior should sometimes one-shot before the monster swings"
        );
    }

    #[test]
    fn high_armor_low_dex_player_can_still_be_hit_overworld() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        let mut state = make_state();
        state.character.dexterity = 8;
        state.character.max_hp = 1000;
        state.character.hp = 1000;
        state.character.equip(Item {
            name: "Bastion Plate".to_string(),
            slot: ItemSlot::Armor,
            power: 100,
            rarity: Rarity::Legendary,
            enchant_level: 0,
        });
        let zone = crate::zones::Zone {
            name: "Test Abyss",
            description: "test",
            danger_level: 4,
            color: crate::zones::ZoneColor::Red,
        };
        let mut rng = StdRng::seed_from_u64(7);
        combat(
            &mut state,
            &mut rng,
            &zone,
            "rm",
            "Test Hunter",
            12,
            500,
            25,
            false,
            6,
        );
        assert!(
            state.character.hp < 1000,
            "high-armor low-dex player must still take hits under the dex-vs-dex dodge model (was un-hittable before)"
        );
    }

    fn healer_state(hp_offset: i32) -> GameState {
        let mut state = make_state();
        state.character.hp = state.character.max_hp - hp_offset;
        state
    }

    fn minimal_boss() -> crate::boss::Boss {
        crate::boss::Boss {
            name: "Test Boss".to_string(),
            hp: 50,
            max_hp: 50,
            attack: 10,
            xp_reward: 100,
            gold_reward: 50,
            spawned_at: chrono::Utc::now(),
            dex_mod: 6,
            dmg_dealt_total: 0,
            dmg_taken_total: 0,
        }
    }

    fn within(
        actual: chrono::DateTime<chrono::Utc>,
        target: chrono::DateTime<chrono::Utc>,
        tolerance_secs: i64,
    ) -> bool {
        (actual - target).num_seconds().abs() <= tolerance_secs
    }

    #[test]
    fn first_visit_initializes_last_heal_at_and_does_not_heal() {
        let mut state = healer_state(5);
        let starting_hp = state.character.hp;
        let now = chrono::Utc::now();
        assert!(state.last_heal_at.is_none());

        handle_passive_healer(&mut state, "/tmp/anywhere", now);

        assert!(state.last_heal_at.is_some());
        assert_eq!(state.character.hp, starting_hp);
    }

    #[test]
    fn heals_one_hp_per_thirty_seconds() {
        let mut state = healer_state(5);
        let now = chrono::Utc::now();
        let last = now - chrono::Duration::seconds(30);
        state.last_heal_at = Some(last);
        let expected_hp = state.character.hp + 1;

        handle_passive_healer(&mut state, "/tmp/anywhere", now);

        assert_eq!(state.character.hp, expected_hp);
        let new_last = state.last_heal_at.expect("timer must be set");
        assert!(
            within(new_last, last + chrono::Duration::seconds(30), 1),
            "expected last_heal_at within 1s of last+30s, got delta {}",
            (new_last - (last + chrono::Duration::seconds(30))).num_seconds()
        );
    }

    #[test]
    fn heals_multiple_intervals_at_once() {
        let mut state = healer_state(5);
        let now = chrono::Utc::now();
        let last = now - chrono::Duration::seconds(90);
        state.last_heal_at = Some(last);
        let expected_hp = state.character.hp + 3;

        handle_passive_healer(&mut state, "/tmp/anywhere", now);

        assert_eq!(state.character.hp, expected_hp);
        let new_last = state.last_heal_at.expect("timer must be set");
        assert!(
            within(new_last, last + chrono::Duration::seconds(90), 1),
            "expected last_heal_at within 1s of last+90s, got delta {}",
            (new_last - (last + chrono::Duration::seconds(90))).num_seconds()
        );
    }

    #[test]
    fn caps_at_max_accumulated_seconds() {
        let mut state = make_state();
        state.character.max_hp = 200;
        state.character.hp = 100;
        let now = chrono::Utc::now();
        state.last_heal_at = Some(now - chrono::Duration::seconds(60 * 60));

        handle_passive_healer(&mut state, "/tmp/anywhere", now);

        assert_eq!(state.character.hp, 160);
        let new_last = state.last_heal_at.expect("timer must be set");
        assert!(
            (now - new_last).num_seconds().abs() <= 30 * 60,
            "expected last_heal_at within 30min of now"
        );
    }

    #[test]
    fn caps_at_max_hp() {
        let mut state = healer_state(3);
        let now = chrono::Utc::now();
        state.last_heal_at = Some(now - chrono::Duration::seconds(10 * 60));
        let max_hp = state.character.max_hp;

        handle_passive_healer(&mut state, "/tmp/anywhere", now);

        assert_eq!(state.character.hp, max_hp);
        let new_last = state.last_heal_at.expect("timer must be set");
        assert!(
            within(new_last, now, 1),
            "expected last_heal_at snapped to ~now, got delta {}",
            (new_last - now).num_seconds()
        );
    }

    #[test]
    fn no_heal_during_boss() {
        let mut state = healer_state(5);
        let starting_hp = state.character.hp;
        let now = chrono::Utc::now();
        let original = now - chrono::Duration::seconds(5 * 60);
        state.last_heal_at = Some(original);
        state.active_boss = Some(minimal_boss());

        handle_passive_healer(&mut state, "/tmp/anywhere", now);

        assert_eq!(state.character.hp, starting_hp);
        assert_eq!(state.last_heal_at, Some(original));
    }

    #[test]
    fn heal_below_interval_no_change() {
        let mut state = healer_state(5);
        let starting_hp = state.character.hp;
        let now = chrono::Utc::now();
        let original = now - chrono::Duration::seconds(10);
        state.last_heal_at = Some(original);

        handle_passive_healer(&mut state, "/tmp/anywhere", now);

        assert_eq!(state.character.hp, starting_hp);
        assert_eq!(state.last_heal_at, Some(original));
    }

    #[test]
    fn emits_journal_entry_at_home() {
        let home = match dirs::home_dir() {
            Some(p) => p.to_string_lossy().to_string(),
            None => return,
        };
        if home.is_empty() {
            return;
        }
        let mut state = healer_state(5);
        let now = chrono::Utc::now();
        state.last_heal_at = Some(now - chrono::Duration::seconds(60));
        let before = state.journal.len();

        handle_passive_healer(&mut state, &home, now);

        assert_eq!(state.journal.len(), before + 1);
        let entry = state.journal.last().expect("journal entry expected");
        assert!(matches!(entry.event_type, EventType::Discovery));
        assert!(
            entry.message.contains("+2 HP"),
            "expected '+2 HP' substring in journal message, got: {}",
            entry.message
        );
    }

    #[test]
    fn silent_outside_home() {
        let mut state = healer_state(5);
        let now = chrono::Utc::now();
        state.last_heal_at = Some(now - chrono::Duration::seconds(60));
        let expected_hp = state.character.hp + 2;
        let before = state.journal.len();

        handle_passive_healer(&mut state, "/tmp/somewhere_not_home", now);

        assert_eq!(state.character.hp, expected_hp);
        assert_eq!(state.journal.len(), before);
    }

    #[test]
    fn at_max_hp_resyncs_timer() {
        let home = match dirs::home_dir() {
            Some(p) => p.to_string_lossy().to_string(),
            None => return,
        };
        if home.is_empty() {
            return;
        }
        let mut state = make_state();
        let max_hp = state.character.max_hp;
        state.character.hp = max_hp;
        let now = chrono::Utc::now();
        state.last_heal_at = Some(now - chrono::Duration::seconds(5 * 60));
        let before = state.journal.len();

        handle_passive_healer(&mut state, &home, now);

        assert_eq!(state.character.hp, max_hp);
        let new_last = state.last_heal_at.expect("timer must be set");
        assert!(
            within(new_last, now, 1),
            "expected last_heal_at snapped to ~now"
        );
        assert_eq!(state.journal.len(), before);
    }

    #[test]
    fn future_timestamp_clock_skew_resets() {
        let mut state = healer_state(5);
        let starting_hp = state.character.hp;
        let now = chrono::Utc::now();
        state.last_heal_at = Some(now + chrono::Duration::seconds(60));

        handle_passive_healer(&mut state, "/tmp/anywhere", now);

        let new_last = state.last_heal_at.expect("timer must be set");
        assert!(
            new_last <= now,
            "expected future timestamp to be reset to ≤ now, got delta {}",
            (new_last - now).num_seconds()
        );
        assert_eq!(state.character.hp, starting_hp);
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

fn add_to_inventory(
    state: &mut GameState,
    item: crate::character::Item,
    quiet_rejection: bool,
) -> bool {
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
        if !quiet_rejection {
            let msg = full_inventory_message(&item);
            eprintln!("{} {}", "📦".bold(), msg.yellow().italic());
        }
        false
    }
}

pub(crate) fn add_to_inventory_pub(state: &mut GameState, item: crate::character::Item) -> bool {
    add_to_inventory(state, item, false)
}

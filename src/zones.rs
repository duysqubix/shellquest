use rand::Rng;

#[derive(Debug, Clone)]
pub struct Zone {
    pub name: &'static str,
    pub description: &'static str,
    pub danger_level: u32,
    pub color: ZoneColor,
}

#[derive(Debug, Clone)]
pub enum ZoneColor {
    Green,
    Yellow,
    Red,
    Blue,
    Magenta,
    Cyan,
}

const VOID_SEGMENT: &str = "the_void";
pub const VOID_ZONE_NAME: &str = "The Void";

fn has_segment(path: &str, seg: &str) -> bool {
    path.split('/').any(|s| s.eq_ignore_ascii_case(seg))
}

fn has_exact_segment(path: &str, seg: &str) -> bool {
    path.split('/').any(|s| s == seg)
}

pub fn is_void_zone(zone: &Zone) -> bool {
    zone.name == VOID_ZONE_NAME
}

pub fn void_depth(path: &str) -> Option<u32> {
    let mut found_void_root = false;
    let mut depth = 0;

    for component in std::path::Path::new(path).components() {
        let std::path::Component::Normal(segment) = component else {
            continue;
        };

        if found_void_root {
            depth += 1;
            continue;
        }

        if segment.to_string_lossy().eq_ignore_ascii_case(VOID_SEGMENT) {
            found_void_root = true;
        }
    }

    found_void_root.then_some(depth)
}

pub fn zone_from_path(path: &str) -> Zone {
    if has_segment(path, VOID_SEGMENT) {
        Zone {
            name: VOID_ZONE_NAME,
            description: "A hollow maze where commands echo back with teeth...",
            danger_level: 5,
            color: ZoneColor::Magenta,
        }
    } else if has_segment(path, ".ssh") {
        Zone {
            name: "The Keyring Crypt",
            description: "Ancient keys sleep under lock and curse...",
            danger_level: 5,
            color: ZoneColor::Magenta,
        }
    } else if has_segment(path, "secrets") || has_segment(path, "private") {
        Zone {
            name: "The Shadow Vault",
            description: "Private names are sealed behind hungry dark...",
            danger_level: 5,
            color: ZoneColor::Red,
        }
    } else if has_segment(path, "sys") {
        Zone {
            name: "The Kernel Sanctum",
            description: "Sacred kernel runes pulse with forbidden authority...",
            danger_level: 5,
            color: ZoneColor::Red,
        }
    } else if has_segment(path, "root") {
        Zone {
            name: "The Forbidden Throne",
            description: "The root crown waits in a chamber that forgives nothing...",
            danger_level: 5,
            color: ZoneColor::Red,
        }
    } else if has_segment(path, "node_modules") {
        Zone {
            name: "The Abyss of node_modules",
            description: "An infinite void of dependencies...",
            danger_level: 5,
            color: ZoneColor::Red,
        }
    } else if has_segment(path, "dev") {
        Zone {
            name: "The Device Caverns",
            description: "Strange devices hum with raw power...",
            danger_level: 4,
            color: ZoneColor::Magenta,
        }
    } else if has_segment(path, "proc") {
        Zone {
            name: "The Process Spires",
            description: "Towering process trees scrape the static sky...",
            danger_level: 4,
            color: ZoneColor::Magenta,
        }
    } else if has_segment(path, "boot") {
        Zone {
            name: "The Ignition Vault",
            description: "Boot sigils smolder beneath cold iron doors...",
            danger_level: 4,
            color: ZoneColor::Magenta,
        }
    } else if has_segment(path, "vendor") {
        Zone {
            name: "The Vendor Wastes",
            description: "Third-party caravans vanish among dependency dunes...",
            danger_level: 4,
            color: ZoneColor::Red,
        }
    } else if has_segment(path, "tmp") {
        Zone {
            name: "The Wasteland of /tmp",
            description: "A desolate land where files come to die...",
            danger_level: 3,
            color: ZoneColor::Red,
        }
    } else if has_segment(path, "etc") {
        Zone {
            name: "The Config Archives",
            description: "Ancient scrolls of configuration line the walls...",
            danger_level: 2,
            color: ZoneColor::Cyan,
        }
    } else if has_segment(path, "var") {
        Zone {
            name: "The Variable Marshes",
            description: "Shifting logs and pools of data...",
            danger_level: 3,
            color: ZoneColor::Yellow,
        }
    } else if has_segment(path, ".aws")
        || has_segment(path, ".gnupg")
        || has_segment(path, ".config")
    {
        Zone {
            name: "The Sigil Vault",
            description: "Cloud sigils and machine charms glow behind warded glass...",
            danger_level: 3,
            color: ZoneColor::Cyan,
        }
    } else if has_segment(path, ".cache") {
        Zone {
            name: "The Forgotten Cache",
            description: "Dusty caches rustle with half-remembered things...",
            danger_level: 2,
            color: ZoneColor::Yellow,
        }
    } else if has_segment(path, "target") || has_segment(path, "build") {
        Zone {
            name: "The Forge",
            description: "The heat of compilation fills the air...",
            danger_level: 2,
            color: ZoneColor::Yellow,
        }
    } else if has_segment(path, ".git") {
        Zone {
            name: "The Time Vaults",
            description: "Echoes of past commits whisper around you...",
            danger_level: 3,
            color: ZoneColor::Magenta,
        }
    } else if has_segment(path, "src") || has_segment(path, "lib") {
        Zone {
            name: "The Source Sanctum",
            description: "Lines of power flow through structured halls...",
            danger_level: 2,
            color: ZoneColor::Blue,
        }
    } else if has_segment(path, "test") || has_segment(path, "tests") {
        Zone {
            name: "The Proving Grounds",
            description: "Assertions echo through the arena...",
            danger_level: 2,
            color: ZoneColor::Green,
        }
    } else if has_segment(path, "bin") || has_segment(path, "sbin") || has_segment(path, "usr") {
        Zone {
            name: "The Binary Bastion",
            description: "Executable armories line the battlements...",
            danger_level: 3,
            color: ZoneColor::Blue,
        }
    } else if has_segment(path, "dist") {
        Zone {
            name: "The Distribution Expanse",
            description: "Bundled artifacts stretch to the horizon...",
            danger_level: 2,
            color: ZoneColor::Yellow,
        }
    } else if has_segment(path, ".cargo") || has_segment(path, ".rustup") {
        Zone {
            name: "The Crate Caverns",
            description: "Rusty crates glitter in compile-lit tunnels...",
            danger_level: 2,
            color: ZoneColor::Yellow,
        }
    } else if has_segment(path, "__pycache__")
        || has_segment(path, ".venv")
        || has_segment(path, "venv")
    {
        Zone {
            name: "The Bytecode Bog",
            description: "Stale bytecode bubbles under a thin virtual mist...",
            danger_level: 3,
            color: ZoneColor::Cyan,
        }
    } else if has_segment(path, ".gradle") || has_segment(path, ".m2") {
        Zone {
            name: "The Artifact Depths",
            description: "Build relics sink through layered dependency stone...",
            danger_level: 3,
            color: ZoneColor::Magenta,
        }
    } else if has_segment(path, "log") || has_segment(path, "logs") {
        Zone {
            name: "The Logfile Mire",
            description: "Rotating logs churn in a swamp of timestamps...",
            danger_level: 3,
            color: ZoneColor::Yellow,
        }
    } else if has_segment(path, "backup") || has_segment(path, "backups") {
        Zone {
            name: "The Vault of Echoes",
            description: "Old backups whisper what the present forgot...",
            danger_level: 2,
            color: ZoneColor::Blue,
        }
    } else if has_segment(path, "data") {
        Zone {
            name: "The Data Wells",
            description: "Deep wells of structured memory glimmer below...",
            danger_level: 2,
            color: ZoneColor::Cyan,
        }
    } else if has_exact_segment(path, "Downloads") {
        Zone {
            name: "The Drift",
            description: "Downloaded curios wash up on a restless shore...",
            danger_level: 2,
            color: ZoneColor::Cyan,
        }
    } else if has_exact_segment(path, "Desktop") {
        Zone {
            name: "The Surface",
            description: "A calm desktop plain under a bright cursor sun...",
            danger_level: 1,
            color: ZoneColor::Green,
        }
    } else if has_exact_segment(path, "Documents") {
        Zone {
            name: "The Archives",
            description: "Personal scrolls rest in orderly, quiet stacks...",
            danger_level: 1,
            color: ZoneColor::Green,
        }
    } else if has_exact_segment(path, "Pictures")
        || has_exact_segment(path, "Music")
        || has_exact_segment(path, "Videos")
    {
        Zone {
            name: "The Gallery",
            description: "Painted echoes and old songs shimmer on the walls...",
            danger_level: 1,
            color: ZoneColor::Blue,
        }
    } else if has_segment(path, "trash") || has_segment(path, ".Trash") {
        Zone {
            name: "The Refuse Pits",
            description: "Discarded files skitter beneath cracked lids...",
            danger_level: 3,
            color: ZoneColor::Yellow,
        }
    } else if path
        == dirs::home_dir()
            .map(|d| d.to_string_lossy().to_string())
            .unwrap_or_default()
    {
        Zone {
            name: "Home Village",
            description: "The safety of your home directory...",
            danger_level: 1,
            color: ZoneColor::Green,
        }
    } else {
        Zone {
            name: "The Wilds",
            description: "Unknown territory stretches before you...",
            danger_level: 2,
            color: ZoneColor::Yellow,
        }
    }
}

pub fn travel_message(zone: &Zone) -> String {
    let mut rng = rand::thread_rng();
    let messages = [
        format!("You enter {}... {}", zone.name, zone.description),
        format!("You venture into {}. {}", zone.name, zone.description),
        format!("The path leads to {}. {}", zone.name, zone.description),
        format!("You find yourself in {}. {}", zone.name, zone.description),
    ];
    messages[rng.gen_range(0..messages.len())].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tmp_maps_to_wasteland() {
        let zone = zone_from_path("/tmp/foo");
        assert_eq!(zone.name, "The Wasteland of /tmp");
        assert_eq!(zone.danger_level, 3);
    }

    #[test]
    fn dev_maps_to_device_caverns() {
        let zone = zone_from_path("/dev/null");
        assert_eq!(zone.name, "The Device Caverns");
        assert_eq!(zone.danger_level, 4);
    }

    #[test]
    fn etc_maps_to_config_archives() {
        let zone = zone_from_path("/etc/hosts");
        assert_eq!(zone.name, "The Config Archives");
        assert_eq!(zone.danger_level, 2);
    }

    #[test]
    fn var_maps_to_variable_marshes() {
        let zone = zone_from_path("/var/log/syslog");
        assert_eq!(zone.name, "The Variable Marshes");
        assert_eq!(zone.danger_level, 3);
    }

    #[test]
    fn node_modules_maps_to_abyss() {
        let zone = zone_from_path("/home/user/project/node_modules/lodash");
        assert_eq!(zone.name, "The Abyss of node_modules");
        assert_eq!(zone.danger_level, 5);
    }

    #[test]
    fn void_path_maps_to_void_zone() {
        let zone = zone_from_path("/home/user/.shellquest/the_void/a/b");
        assert_eq!(zone.name, VOID_ZONE_NAME);
        assert!(zone.danger_level >= 5);
        assert!(matches!(zone.color, ZoneColor::Magenta));
    }

    #[test]
    fn void_depth_counts_components_after_void_root() {
        assert_eq!(void_depth("/home/user/.shellquest/the_void"), Some(0));
        assert_eq!(void_depth("/home/user/.shellquest/the_void/a"), Some(1));
        assert_eq!(void_depth("/home/user/.shellquest/the_void/a/b"), Some(2));
        assert_eq!(void_depth("/tmp/not_void/a/b"), None);
    }

    #[test]
    fn target_maps_to_forge() {
        let zone = zone_from_path("/home/user/project/target/debug");
        assert_eq!(zone.name, "The Forge");
        assert_eq!(zone.danger_level, 2);
    }

    #[test]
    fn build_maps_to_forge() {
        let zone = zone_from_path("/home/user/project/build/release");
        assert_eq!(zone.name, "The Forge");
    }

    #[test]
    fn git_maps_to_time_vaults() {
        let zone = zone_from_path("/home/user/project/.git/objects");
        assert_eq!(zone.name, "The Time Vaults");
        assert_eq!(zone.danger_level, 3);
    }

    #[test]
    fn src_maps_to_source_sanctum() {
        let zone = zone_from_path("/home/user/project/src/main.rs");
        assert_eq!(zone.name, "The Source Sanctum");
        assert_eq!(zone.danger_level, 2);
    }

    #[test]
    fn lib_maps_to_source_sanctum() {
        let zone = zone_from_path("/usr/lib/libssl.so");
        assert_eq!(zone.name, "The Source Sanctum");
    }

    #[test]
    fn tests_dir_maps_to_proving_grounds() {
        let zone = zone_from_path("/home/user/project/tests/integration.rs");
        assert_eq!(zone.name, "The Proving Grounds");
        assert_eq!(zone.danger_level, 2);
    }

    #[test]
    fn proc_maps_to_process_spires() {
        let zone = zone_from_path("/proc/123/status");
        assert_eq!(zone.name, "The Process Spires");
        assert_eq!(zone.danger_level, 4);
    }

    #[test]
    fn sys_maps_to_kernel_sanctum() {
        let zone = zone_from_path("/sys/kernel/debug");
        assert_eq!(zone.name, "The Kernel Sanctum");
        assert_eq!(zone.danger_level, 5);
    }

    #[test]
    fn root_maps_to_forbidden_throne() {
        let zone = zone_from_path("/root/.profile");
        assert_eq!(zone.name, "The Forbidden Throne");
        assert_eq!(zone.danger_level, 5);
    }

    #[test]
    fn boot_maps_to_ignition_vault() {
        let zone = zone_from_path("/boot/loader");
        assert_eq!(zone.name, "The Ignition Vault");
        assert_eq!(zone.danger_level, 4);
    }

    #[test]
    fn usr_bin_maps_to_binary_bastion() {
        let zone = zone_from_path("/usr/local/bin/sq");
        assert_eq!(zone.name, "The Binary Bastion");
        assert_eq!(zone.danger_level, 3);
    }

    #[test]
    fn ssh_maps_to_keyring_crypt() {
        let zone = zone_from_path("/home/user/.ssh/id_ed25519");
        assert_eq!(zone.name, "The Keyring Crypt");
        assert_eq!(zone.danger_level, 5);
    }

    #[test]
    fn config_maps_to_sigil_vault() {
        let zone = zone_from_path("/home/user/.config/sq/config.json");
        assert_eq!(zone.name, "The Sigil Vault");
        assert_eq!(zone.danger_level, 3);
    }

    #[test]
    fn cache_maps_to_forgotten_cache() {
        let zone = zone_from_path("/home/user/.cache/shellquest");
        assert_eq!(zone.name, "The Forgotten Cache");
        assert_eq!(zone.danger_level, 2);
    }

    #[test]
    fn vendor_maps_to_vendor_wastes() {
        let zone = zone_from_path("/home/user/project/vendor/bundle");
        assert_eq!(zone.name, "The Vendor Wastes");
        assert_eq!(zone.danger_level, 4);
    }

    #[test]
    fn dist_maps_to_distribution_expanse() {
        let zone = zone_from_path("/home/user/project/dist/app.js");
        assert_eq!(zone.name, "The Distribution Expanse");
        assert_eq!(zone.danger_level, 2);
    }

    #[test]
    fn cargo_maps_to_crate_caverns() {
        let zone = zone_from_path("/home/user/.cargo/registry");
        assert_eq!(zone.name, "The Crate Caverns");
        assert_eq!(zone.danger_level, 2);
    }

    #[test]
    fn pycache_maps_to_bytecode_bog() {
        let zone = zone_from_path("/home/user/project/__pycache__/main.pyc");
        assert_eq!(zone.name, "The Bytecode Bog");
        assert_eq!(zone.danger_level, 3);
    }

    #[test]
    fn gradle_maps_to_artifact_depths() {
        let zone = zone_from_path("/home/user/.gradle/caches");
        assert_eq!(zone.name, "The Artifact Depths");
        assert_eq!(zone.danger_level, 3);
    }

    #[test]
    fn downloads_maps_to_drift() {
        let zone = zone_from_path("/home/user/Downloads/archive.zip");
        assert_eq!(zone.name, "The Drift");
        assert_eq!(zone.danger_level, 2);
    }

    #[test]
    fn desktop_maps_to_surface() {
        let zone = zone_from_path("/home/user/Desktop/note.txt");
        assert_eq!(zone.name, "The Surface");
        assert_eq!(zone.danger_level, 1);
    }

    #[test]
    fn documents_maps_to_archives() {
        let zone = zone_from_path("/home/user/Documents/readme.txt");
        assert_eq!(zone.name, "The Archives");
        assert_eq!(zone.danger_level, 1);
    }

    #[test]
    fn pictures_maps_to_gallery() {
        let zone = zone_from_path("/home/user/Pictures/photo.png");
        assert_eq!(zone.name, "The Gallery");
        assert_eq!(zone.danger_level, 1);
    }

    #[test]
    fn log_maps_to_logfile_mire() {
        let zone = zone_from_path("/home/user/log/app.log");
        assert_eq!(zone.name, "The Logfile Mire");
        assert_eq!(zone.danger_level, 3);
    }

    #[test]
    fn backup_maps_to_vault_of_echoes() {
        let zone = zone_from_path("/home/user/backup/save.json");
        assert_eq!(zone.name, "The Vault of Echoes");
        assert_eq!(zone.danger_level, 2);
    }

    #[test]
    fn data_maps_to_data_wells() {
        let zone = zone_from_path("/home/user/project/data/store.db");
        assert_eq!(zone.name, "The Data Wells");
        assert_eq!(zone.danger_level, 2);
    }

    #[test]
    fn secrets_maps_to_shadow_vault() {
        let zone = zone_from_path("/home/user/project/secrets/prod.env");
        assert_eq!(zone.name, "The Shadow Vault");
        assert_eq!(zone.danger_level, 5);
    }

    #[test]
    fn trash_maps_to_refuse_pits() {
        let zone = zone_from_path("/home/user/.Trash/old.tmp");
        assert_eq!(zone.name, "The Refuse Pits");
        assert_eq!(zone.danger_level, 3);
    }

    #[test]
    fn unknown_path_falls_through_to_wilds() {
        let zone = zone_from_path("/home/user/documents/readme.txt");
        assert_eq!(zone.name, "The Wilds");
    }

    #[test]
    fn travel_message_includes_zone_name() {
        let zone = zone_from_path("/tmp/x");
        let msg = travel_message(&zone);
        assert!(
            msg.contains(zone.name),
            "travel_message '{}' should contain zone name '{}'",
            msg,
            zone.name
        );
    }
}

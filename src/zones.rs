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

fn has_segment(path: &str, seg: &str) -> bool {
    path.split('/').any(|s| s.eq_ignore_ascii_case(seg))
}

pub fn zone_from_path(path: &str) -> Zone {
    if has_segment(path, "tmp") {
        Zone {
            name: "The Wasteland of /tmp",
            description: "A desolate land where files come to die...",
            danger_level: 3,
            color: ZoneColor::Red,
        }
    } else if has_segment(path, "dev") {
        Zone {
            name: "The Device Caverns",
            description: "Strange devices hum with raw power...",
            danger_level: 4,
            color: ZoneColor::Magenta,
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
    } else if has_segment(path, "node_modules") {
        Zone {
            name: "The Abyss of node_modules",
            description: "An infinite void of dependencies...",
            danger_level: 5,
            color: ZoneColor::Red,
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
    } else if path == dirs::home_dir().map(|d| d.to_string_lossy().to_string()).unwrap_or_default() {
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

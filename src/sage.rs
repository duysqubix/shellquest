use crate::state::GameState;
use chrono::Utc;
use colored::*;
use rand::Rng;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Check crates.io for the latest version (cached for 24 hours).
/// Returns true if a newer version is available.
fn check_for_update(state: &mut GameState) -> bool {
    let now = Utc::now();

    // Only check crates.io once every 24 hours
    if let Some(last_check) = state.last_version_check {
        if (now - last_check).num_hours() < 24 {
            // Use cached result
            return state
                .latest_version
                .as_ref()
                .map_or(false, |v| v.as_str() != CURRENT_VERSION);
        }
    }

    // Try to fetch latest version (with a short timeout so tick stays fast)
    let latest = fetch_latest_version();
    state.last_version_check = Some(now);

    if let Some(ref ver) = latest {
        let outdated = ver.as_str() != CURRENT_VERSION;
        state.latest_version = latest;
        outdated
    } else {
        false
    }
}

fn fetch_latest_version() -> Option<String> {
    let resp = ureq::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .get("https://crates.io/api/v1/crates/shellquest")
        .call()
        .ok()?;

    let body: String = resp.into_string().ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&body).ok()?;
    parsed["crate"]["newest_version"]
        .as_str()
        .map(|s: &str| s.to_string())
}

/// Force the sage to appear (for testing with --test-sage).
pub fn force_show_sage(state: &mut GameState) {
    let mut rng = rand::thread_rng();

    // Try a real version check, but show regardless
    let _ = check_for_update(state);

    let latest = state
        .latest_version
        .as_deref()
        .unwrap_or("?.?.?");

    let messages = [
        format!(
            "Psst... I sense a disturbance in the codebase. Version {} awaits, young one.",
            latest
        ),
        format!(
            "The ancient scrolls speak of v{}... a power upgrade most worthy.",
            latest
        ),
        format!(
            "A new artifact has been forged: shellquest v{}. Seek it with 'sq update'!",
            latest
        ),
        format!(
            "The stars align! Version {} has been released into the wild. Type 'sq update' to claim it.",
            latest
        ),
        format!(
            "I've traveled far to bring you news... v{} brings new enchantments. Run 'sq update'!",
            latest
        ),
        format!(
            "Your v{} is showing its age, adventurer. Version {} calls to you!",
            CURRENT_VERSION, latest
        ),
        format!(
            "By my beard! v{} dropped and you're still on v{}? Run 'sq update', quick!",
            latest, CURRENT_VERSION
        ),
    ];

    let msg = &messages[rng.gen_range(0..messages.len())];
    print_sage(msg);
}

/// Maybe show the sage during a tick. Very rare, max 3 times per day.
pub fn maybe_show_sage(state: &mut GameState) {
    let mut rng = rand::thread_rng();

    // 1 in 50 chance per tick
    if !rng.gen_ratio(1, 50) {
        return;
    }

    // Max 3 times per day (~8 hours apart)
    if let Some(last_shown) = state.last_sage_shown {
        if (Utc::now() - last_shown).num_hours() < 8 {
            return;
        }
    }

    // Check if update is actually available
    if !check_for_update(state) {
        return;
    }

    let latest = state
        .latest_version
        .as_deref()
        .unwrap_or("???");

    state.last_sage_shown = Some(Utc::now());

    let messages = [
        format!(
            "Psst... I sense a disturbance in the codebase. Version {} awaits, young one.",
            latest
        ),
        format!(
            "The ancient scrolls speak of v{}... a power upgrade most worthy.",
            latest
        ),
        format!(
            "A new artifact has been forged: shellquest v{}. Seek it with 'sq update'!",
            latest
        ),
        format!(
            "The stars align! Version {} has been released into the wild. Type 'sq update' to claim it.",
            latest
        ),
        format!(
            "I've traveled far to bring you news... v{} brings new enchantments. Run 'sq update'!",
            latest
        ),
        format!(
            "Your v{} is showing its age, adventurer. Version {} calls to you!",
            CURRENT_VERSION, latest
        ),
        format!(
            "By my beard! v{} dropped and you're still on v{}? Run 'sq update', quick!",
            latest, CURRENT_VERSION
        ),
    ];

    let msg = &messages[rng.gen_range(0..messages.len())];
    print_sage(msg);
}

fn print_sage(message: &str) {
    // Word-wrap the message to fit inside the speech bubble (max ~46 chars)
    let wrapped = word_wrap(message, 46);
    let max_width = wrapped.iter().map(|l| l.len()).max().unwrap_or(0);
    let border = "─".repeat(max_width + 2);

    eprintln!();
    eprintln!("  {}", format!("┌{}┐", border).cyan().dimmed());
    for line in &wrapped {
        let padding = " ".repeat(max_width - line.len());
        eprintln!(
            "  {} {} {}",
            "│".cyan().dimmed(),
            line.yellow(),
            format!("{}│", padding).cyan().dimmed()
        );
    }
    eprintln!("  {}", format!("└{}┘", border).cyan().dimmed());
    eprintln!("  {}",  "  \\".cyan().dimmed());
    eprintln!("   {}", r"          __/\__".magenta());
    eprintln!("   {}{}{}",  r"      .".magenta(), "_".yellow().bold(), r"  \\''//".magenta());
    eprintln!("   {}{}{}",  r"     -(".magenta(), " ".yellow().bold(), r")-/_||_\".magenta());
    eprintln!("   {}",  r"      .'. \_()_/".magenta());
    eprintln!("   {}",  r"       |   | . \".magenta());
    eprintln!("   {}",  r"       |   | .  \".magenta());
    eprintln!("   {}",  r"      .'. ,\_____'.".magenta());
    eprintln!("   {}", "    ~ The New Age Sage ~".cyan().italic().dimmed());
    eprintln!();
}

fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() > max_width {
            lines.push(current_line);
            current_line = word.to_string();
        } else {
            current_line.push(' ');
            current_line.push_str(word);
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

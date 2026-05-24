//! In-game help registry for `sq`.

#![allow(dead_code)]

use std::collections::HashMap;
use strsim::levenshtein;
use colored::*;

#[derive(Debug)]
pub struct HelpTopic {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub summary: &'static str,
    pub usage: &'static str,
    pub details: &'static str,
    pub examples: &'static [&'static str],
    pub related: &'static [&'static str],
}

pub const CANONICAL_TOPIC_ORDER: &[&str] = &[
    "help",
    "init",
    "status",
    "inventory",
    "identify",
    "journal",
    "tick",
    "hook",
    "equip",
    "wield",
    "remove",
    "drop",
    "shop",
    "buy",
    "sell",
    "enchant",
    "drink",
    "prestige",
    "reset",
    "update",
    "arena",
    "tournament",
];

const TOPICS: &[HelpTopic] = &[
    HelpTopic {
        name: "help",
        aliases: &[],
        summary: "Browse the in-game manual for sq commands.",
        usage: "sq help [topic]",
        details: "There are three distinct help surfaces. `sq help` (no argument) prints the topic index — every authored command in canonical order with a one-line summary. `sq help <topic>` prints the full guide for one topic with summary, aliases, usage, examples, notes, and related links. `sq --help` (and `sq <subcommand> --help`) prints Clap's auto-generated argument usage instead and is unrelated to the authored manual. Topic lookup accepts canonical names (status, journal, arena, …) and aliases (stat → status, inv → inventory, wear → equip, unequip → remove). Typos within edit-distance 2 surface up to three close-match suggestions; unrecognised input falls back to the same suggestion list with no match found.",
        examples: &[
            "sq help",
            "sq help arena",
            "sq help stat",
            "sq --help",
        ],
        related: &["init", "status"],
    },
    HelpTopic {
        name: "init",
        aliases: &[],
        summary: "Create a new character.",
        usage: "sq init",
        details: "Interactive setup that walks through four prompts: name, class (Wizard, Warrior, Rogue, Ranger, Necromancer — each with its own affinity commands and message flavor), race (Human, Elf, Dwarf, Orc, Goblin — each with a stat spread), and whether to enable permadeath. The character is written to ~/.shellquest/save.json. If a character already exists, init asks for [y/N] confirmation before overwriting; declining cancels without touching the existing save. Standard mode penalizes death with a 15% gold loss and a current-level XP reset. Permadeath mode wipes the save on death — there is no recovery. After init, run `sq hook --shell <bash|zsh|fish>` so passive ticks fire after every shell command.",
        examples: &["sq init"],
        related: &["status", "hook", "reset"],
    },
    HelpTopic {
        name: "status",
        aliases: &["stat"],
        summary: "View your character sheet.",
        usage: "sq status",
        details: "Prints your name, class, race, level and XP bar, current/max HP, gold, prestige tier, equipped weapon/armor/ring, and the zone derived from your current directory, followed by the same inventory listing `sq inventory` would produce — a single-screen snapshot of your character before equipping or selling. The alias `sq stat` is identical to `sq status`. To drill into a single equipped or inventory item, use `sq identify <name>`.",
        examples: &[
            "sq status",
            "sq stat",
        ],
        related: &["inventory", "identify", "journal"],
    },
    HelpTopic {
        name: "inventory",
        aliases: &["inv"],
        summary: "Check your unequipped items.",
        usage: "sq inventory",
        details: "Lists every unequipped item you carry — weapons, armor, rings, and potions — each with name, slot, power, and rarity. Inventory is hard-capped at 20 slots: when new loot would overflow, the weakest item is dropped automatically to make room. To act on items, use `sq wield <name>` for weapons, `sq equip <name>` for armor or rings, `sq drink <name>` for potions, `sq drop <name>` to discard permanently, or `sq sell <number|name>` from your home directory to convert an item into gold at half its shop price. For a deep-dive on any single item — base power, enchant level, effective power, buy price, sell breakdown, and where it lives — use `sq identify <name>`.",
        examples: &[
            "sq inventory",
            "sq inv",
        ],
        related: &["status", "identify", "drop", "equip", "wield", "drink", "enchant"],
    },
    HelpTopic {
        name: "identify",
        aliases: &["id"],
        summary: "Show full stats and modifiers for an inventory or equipped item.",
        usage: "sq identify <item name>",
        details: "Prints a card-style detail block for one item: name, slot, rarity, base power, enchant tag and effective power (when enchanted), buy price, sell-value breakdown (base + enchant bonus = total), and where the item lives (Equipped in a specific slot, or `Inventory slot N of M`). Read-only — no state is mutated and no save is written, so unlike `sq enchant`, `sq buy`, or `sq sell` there is no home-directory restriction; identify works from anywhere. Name matching mirrors `sq equip`, `sq drop`, and `sq sell`: an exact name always matches, case-insensitive substrings match (e.g. `scythe` finds `Scythe of Segfault`), and multi-word queries match when every typed token appears anywhere in the item name (e.g. `ring fortune` finds `Ring of Fortune`). The lookup searches BOTH equipped slots (weapon, armor, ring, in that order) and inventory (in inventory order); equipped matches come first in the candidate list, so when the same name appears in both places the equipped copy is shown by default. Append `.1`, `.2`, … to disambiguate (e.g. `sq identify scythe.2` shows the inventory copy when one is also equipped). Potions show a `Heals:` line instead of base/effective power and skip the enchant block entirely.",
        examples: &[
            "sq identify scythe",
            "sq id ring fortune",
            "sq identify pike.2",
            "sq identify common potion",
        ],
        related: &["inventory", "status", "enchant", "sell"],
    },
    HelpTopic {
        name: "journal",
        aliases: &[],
        summary: "View your adventure log.",
        usage: "sq journal",
        details: "Prints the 20 most recent journal entries, newest first. Each tick can record events from any of the journal categories: combat, loot, level-ups, deaths, quests, crafting, zone travel, and discoveries. The save keeps the last 100 entries — older lines are pruned automatically as new ones land. Entries are written by `sq tick` even when nothing visible printed to the terminal, so the journal is the canonical record of what your character has been up to.",
        examples: &["sq journal"],
        related: &["status", "tick"],
    },
    HelpTopic {
        name: "tick",
        aliases: &[],
        summary: "Process one shell command (run by the shell hook; rarely invoked by hand).",
        usage: "sq tick --cmd <STR> --cwd <PATH> --exit-code <N>",
        details: "Internal entry point that the shell hook installed via `sq hook` calls synchronously after every command — players almost never type it manually except to test event paths. Each invocation processes exactly one passive game event (XP gain, loot roll, combat, trap, zone travel, boss hit, or sage notification). Three required flags describe the command that just ran: `--cmd <STR>` is the literal command line (e.g. `git commit` or `cargo build`) used for affinity matching and event routing, `--cwd <PATH>` is the working directory used to derive the zone and its danger multiplier, and `--exit-code <N>` is the command's exit status (zero for success, non-zero unlocks trap and failure-event rolls). All game output is written to stderr so piped stdout is never polluted, and tick returns silently when no save file exists at `~/.shellquest/save.json` so a freshly-installed hook on a machine without `sq init` does not spam errors. There is also a hidden `--test-sage` flag intended for verifying the update-notifier path: it forces the sage to appear regardless of the version cache and is omitted from `sq tick --help`. It is for testing only — players should leave it alone.",
        examples: &[
            "sq tick --cmd \"git commit\" --cwd \".\" --exit-code 0",
            "sq tick --cmd \"bad\" --cwd \"/tmp\" --exit-code 1",
        ],
        related: &["hook", "journal"],
    },
    HelpTopic {
        name: "hook",
        aliases: &[],
        summary: "Print or install the shell hook.",
        usage: "sq hook [--shell bash|zsh|fish] [--install] [--file <PATH>]",
        details: "Generates the shell-specific code that calls `sq tick` after every command, which is what turns ordinary terminal use into game ticks. `--shell` picks the target shell and defaults to `zsh`; the supported values are `bash` (uses PROMPT_COMMAND), `zsh` (uses precmd_functions), and `fish` (uses fish_postexec). Without `--install` and without `--file`, the hook is printed to stdout so you can pipe or paste it into your rc file yourself. With `--install`, the hook is appended to the shell's default rc file: ~/.bashrc, ~/.zshrc, or ~/.config/fish/config.fish. With `--file <PATH>`, the hook is appended to that path instead — supplying `--file` implies `--install`, so the explicit flag is optional. Install is idempotent: an existing `__sq_hook` marker in the target file is detected and the install is skipped, so re-running on an already-hooked shell will not duplicate the snippet.",
        examples: &[
            "sq hook --shell zsh",
            "sq hook --shell bash --install",
            "sq hook --shell fish --install",
            "sq hook --shell zsh --file ~/.config/zsh/.zshrc",
        ],
        related: &["init", "tick"],
    },
    HelpTopic {
        name: "equip",
        aliases: &["wear"],
        summary: "Equip armor or a ring from your inventory.",
        usage: "sq equip <item name>",
        details: "Equips one inventory item into the matching slot. Armor and rings only — weapons are rejected with a hint to use `sq wield`, and potions are rejected as non-equippable. Name matching is forgiving: an exact item name always matches, case-insensitive substrings match (e.g. `leather` finds `Leather Cuirass`), and multi-word queries match when every typed token appears anywhere in the item name (e.g. `ring of fortune` finds `Ring of Fortune`). When multiple inventory items match the same query, the first match in inventory order wins; append `.1`, `.2`, … to pick a specific match (e.g. `sq equip ring.2` selects the second matching ring). If the target slot already holds a piece, the previously equipped item is automatically returned to inventory. The `wear` alias resolves to this same flow — there is no separate `wear` topic.",
        examples: &[
            "sq equip leather",
            "sq wear ring of fortune",
            "sq equip ring.2",
        ],
        related: &["wield", "remove", "inventory"],
    },
    HelpTopic {
        name: "wield",
        aliases: &[],
        summary: "Wield a weapon from your inventory.",
        usage: "sq wield <weapon name>",
        details: "Equips an inventory weapon into the weapon slot. Weapon-only — armor and rings are rejected with a hint to use `sq equip`, and potions are rejected with a hint to use `sq drink`. Name matching mirrors `sq equip` and `sq drop`: an exact weapon name always matches, case-insensitive substrings match (e.g. `vorpal` finds `Vorpal Pointer`), and multi-word queries match when every typed token appears anywhere in the weapon's name (e.g. `rusty sword` finds `Rusty Sword`). When several weapons match the same query, the first inventory match wins by default; append `.1`, `.2`, … to pick a specific one (e.g. `sq wield sword.2` selects the second matching weapon). If a weapon is already wielded, it is sheathed back into inventory automatically when the new one takes the slot.",
        examples: &[
            "sq wield vorpal",
            "sq wield rusty sword",
            "sq wield sword.2",
        ],
        related: &["equip", "remove", "inventory"],
    },
    HelpTopic {
        name: "remove",
        aliases: &["unequip"],
        summary: "Unequip a weapon, armor, or ring back to your inventory.",
        usage: "sq remove <slot|item name>",
        details: "Operates on equipped gear (the weapon, armor, and ring slots), not loose inventory. Two ways to pick what to unequip: a slot keyword — `weapon`, `armor` (`armour` is also accepted), or `ring` — or a fuzzy name match against the currently equipped piece in any slot (case-insensitive substring or multi-word token match against the equipped item's name). Because each slot holds at most one item, the `.1`/`.2` selector used for inventory commands is unnecessary here — only one item per slot can ever match. Fails when your inventory is already at the 20-item cap; drop or sell something first to make room. The `unequip` alias resolves to this same flow — there is no separate `unequip` topic.",
        examples: &[
            "sq remove ring",
            "sq unequip armor",
            "sq remove rusty",
        ],
        related: &["equip", "wield", "drop"],
    },
    HelpTopic {
        name: "drop",
        aliases: &[],
        summary: "Drop an item from inventory permanently.",
        usage: "sq drop <item name>",
        details: "Permanently destroys an inventory item — there is no recovery, no trash bin, and no gold returned. Useful when the inventory is at its 20-item cap and you want to free a slot without walking home to sell. Name matching is the standard inventory matcher: an exact name always matches, case-insensitive substrings match (e.g. `rusty` finds `Rusty Pipe`), and multi-word queries match when every typed token appears in the item name (e.g. `common potion` finds `Common Potion`). When multiple items match the same query, the first inventory match is dropped by default; append `.1`, `.2`, … to drop the N-th match (e.g. `sq drop common.2` discards the second match). If you would rather convert the item to gold, use `sq sell` from your home directory instead.",
        examples: &[
            "sq drop rusty",
            "sq drop common potion",
            "sq drop common.2",
        ],
        related: &["sell", "inventory"],
    },
    HelpTopic {
        name: "shop",
        aliases: &[],
        summary: "Browse the shop (home directory only).",
        usage: "sq shop",
        details: "The Terminal Bazaar lists exactly six numbered items with rarity, slot, power, and price, plus your current gold for quick affordability checks. Stock is generated once per UTC day and persists across reads until midnight UTC, so the same six items remain available for repeated browsing within a day. Only accessible while your shell's working directory is exactly your home directory — running `sq shop` from anywhere else prints a `cd ~` hint and exits without spending a tick. Pair it with `sq buy <number>` to purchase from the displayed list (numbers are 1-indexed) and `sq sell <number|name>` to convert inventory items back into gold at half price.",
        examples: &["cd ~ && sq shop"],
        related: &["buy", "sell", "inventory"],
    },
    HelpTopic {
        name: "buy",
        aliases: &[],
        summary: "Buy an item from the shop by number.",
        usage: "sq buy <number>",
        details: "Numbers are 1-indexed and refer to the most recent `sq shop` listing — `1` is the first row, `6` is the last. There is no name-based purchase path: type the literal slot number, not the item name. Requires being in your home directory and having at least the listed price in gold; out-of-range numbers, insufficient gold, and being away from home all bail out without spending the gold. The purchase removes the item from the shop until the next daily refresh and pushes it into your inventory, which is hard-capped at 20 slots — manage space with `sq drop` or `sq sell` first if you are full.",
        examples: &[
            "sq buy 1",
            "sq buy 4",
        ],
        related: &["shop", "sell"],
    },
    HelpTopic {
        name: "sell",
        aliases: &[],
        summary: "Sell an inventory item at the shop by number, name, or sweep all junk.",
        usage: "sq sell <number|name|junk>",
        details: "Three selection modes. (1) A 1-indexed slot from your `sq inventory` listing (so `sq sell 3` removes the third inventory item). (2) A partial item name — same matcher as `sq equip`, `sq drop`, and `sq drink`: exact name always matches, case-insensitive substrings match (e.g. `rusty` finds `Rusty Pipe`), and multi-word queries match when every typed token appears anywhere in the item name (e.g. `big sword` finds `Big Sword of Awesome`); when several inventory items match the same name, the first inventory match is sold by default — append `.1`, `.2`, … to pick the N-th match (e.g. `sq sell potion.2` sells the second matching potion). (3) The literal word `junk` — sweeps every Common and Uncommon item out of your inventory in one shot, leaving Rare, Epic, and Legendary items untouched; a quick way to clean up after a long grind. All three modes sell for half of the item's listed buy price. Only available from your home directory; called from elsewhere it prints a home-only hint and exits without removing the item.",
        examples: &[
            "cd ~ && sq sell 3",
            "cd ~ && sq sell rusty",
            "cd ~ && sq sell potion.2",
            "cd ~ && sq sell junk",
        ],
        related: &["shop", "buy", "inventory"],
    },
    HelpTopic {
        name: "enchant",
        aliases: &[],
        summary: "Spend gold to add +1 power to an equipped item (max +5 per item).",
        usage: "sq enchant <item name>",
        details: "Targets only EQUIPPED items (weapon, armor, or ring). Each enchant adds +1 to the item's effective power and costs `item_price × (current_enchant_level + 1)` gold — so the first enchant costs the item's full buy price, the second costs 2x, the third 3x, and so on. Max enchant level per item is +5; total cost to fully enchant an item is therefore 15× its buy price (e.g. 225 gold for a Common, 2,250 for a Rare, 30,000 for a Legendary). Potions cannot be enchanted. Enchanted items show an `[Enchanted +N]` tag in escalating colors — green at +1, cyan at +2, blue at +3, magenta at +4, and rainbow per-character at the max +5. Sell-price recovers a small fraction of the enchant investment so enchant-then-sell is always a net loss. Access rule: the **Wizard** class can enchant from anywhere (arcane workbench mastery); all other classes must be standing in their home directory.",
        examples: &[
            "cd ~ && sq enchant pike",
            "cd ~ && sq enchant iron sword",
            "sq enchant blade   # (Wizard only, anywhere)",
        ],
        related: &["inventory", "status", "wield", "equip"],
    },
    HelpTopic {
        name: "drink",
        aliases: &[],
        summary: "Drink a potion from inventory to restore HP.",
        usage: "sq drink <potion name>",
        details: "Consumes an inventory potion and restores its listed power as HP, capped at your maximum HP — overheal is not possible and any excess is discarded. Non-potion items are rejected with a not-drinkable hint. Name matching is the standard inventory matcher: an exact potion name always matches, case-insensitive substrings match (e.g. `minor` finds `Minor Potion`), and multi-word queries match when every typed token appears anywhere in the item name. When multiple potions match the same query, the first inventory match is consumed by default; append `.1`, `.2`, … to drink a specific one (e.g. `sq drink potion.2` drinks the second matching potion).",
        examples: &[
            "sq drink minor",
            "sq drink elixir",
            "sq drink potion.2",
        ],
        related: &["inventory", "status"],
    },
    HelpTopic {
        name: "prestige",
        aliases: &[],
        summary: "Reset level and XP at the cap; keep everything else and gain permanent bonuses.",
        usage: "sq prestige",
        details: "Only available at the level cap of 150 — `sq prestige` aborts with a level-required hint at any lower level. The reset is narrow: level and XP drop back to 1, but every long-term piece of progress carries over untouched — gold, equipped gear, full inventory, journal entries, kill count, shop state, and any prestige tiers earned previously. Each prestige tier grants a permanent +2 to STR, DEX, and INT and +10 max HP, and unlocks one of fifteen class-specific subclasses (three per class) chosen interactively at the prompt. The chosen subclass layers further stat bonuses on top and is reflected in your title from then on. Tiers stack on every successful prestige, so tier 3 is +6 stats / +30 HP over baseline and tier 5 is +10 stats / +50 HP — the loop rewards endurance. If you want to wipe gold, gear, and inventory and start fresh instead, use `sq reset`; prestige is the keep-everything path.",
        examples: &["sq prestige"],
        related: &["status", "reset"],
    },
    HelpTopic {
        name: "reset",
        aliases: &[],
        summary: "Permanently delete your character and save file.",
        usage: "sq reset",
        details: "Destructive and irreversible. Asks for [y/N] confirmation, then deletes ~/.shellquest/save.json outright. Your character, journal, gold, gear, inventory, kills, prestige tier, shop state, and any version-check cache are erased. There is no undo, no automatic backup, and no trash bin — the file is gone. After reset, run `sq init` to start a new character. If you want to keep gold, gear, kills, and inventory while restarting your level, use `sq prestige` at level 150 instead.",
        examples: &["sq reset"],
        related: &["init", "prestige"],
    },
    HelpTopic {
        name: "update",
        aliases: &[],
        summary: "Update sq to the latest release.",
        usage: "sq update",
        details: "Runs `cargo install shellquest --force` under the hood to fetch and install the latest release from crates.io. Requires `cargo` on your PATH; if cargo is missing, the update aborts and points you at https://rustup.rs. Your save file at ~/.shellquest/save.json is left untouched — only the `sq` binary is replaced. Restart your shell (or open a new one) after the install so the hook picks up the new binary. Note: this only updates the binary; if a release also requires regenerating the shell hook, re-run `sq hook --shell <name> --install` afterwards.",
        examples: &["sq update"],
        related: &["hook"],
    },
    HelpTopic {
        name: "arena",
        aliases: &[],
        summary: "Enter the interactive combat gauntlet — risk gold for XP, loot, and gold.",
        usage: "sq arena",
        details: "An interactive combat gauntlet that requires a real TTY on both stdin and stdout. Piped or redirected input (e.g. `echo y | sq arena`) exits with status 1 and an `Arena requires an interactive terminal.` notice — there is no scriptable path. After picking a tier, you must confirm the entry fee in gold (deducted up front); fights then resolve one round at a time, and after each round the prompt asks whether to continue deeper for richer rewards or cash out with current winnings. Cashing out commits the run atomically: gold and XP land in one shot, the journal records `Arena cleared {tier} after N rounds`, and any chest loot that overflows the 20-item inventory cap converts to gold at half its sell value. A knockout costs the entry fee, sets HP to 25% of the max HP recorded at entry, prints `Knocked out`, and writes `Arena KO in {tier} after N rounds. Fee: N gold.` to the journal. Runs are not resumable, so any hard interruption (Ctrl+C, terminal close, crash, save failure) rolls the entire session back to the pre-arena state — entry fee included — and nothing user-visible is committed. Higher tiers (Gauntlet, Colosseum, Abyssal Arena, Godslayer's Court) require minimum level and/or prestige to unlock; locked tiers print a 🔒 hint and exit without taking gold.",
        examples: &["sq arena"],
        related: &["tournament", "status", "prestige"],
    },
    HelpTopic {
        name: "tournament",
        aliases: &[],
        summary: "Deprecated — use `sq arena` instead.",
        usage: "sq tournament",
        details: "Deprecated alias kept only for backwards compatibility with older muscle memory and scripts. Running `sq tournament` prints a yellow `⚠️  The `tournament` command is deprecated. Use `sq arena` instead.` notice and then routes through the exact same flow as `sq arena`: same TTY requirement on stdin and stdout, same tier selection, same entry-fee confirmation prompt, same per-round continue-or-cash-out loop, and the same atomic-rollback semantics on interruption. There is no separate tournament feature, no different reward table, and no plan to keep this command around indefinitely — treat it strictly as a redirect and prefer `sq arena` directly. See `sq help arena` for the full behavioral contract.",
        examples: &["sq tournament"],
        related: &["arena"],
    },
];

pub fn all_topics() -> &'static [HelpTopic] {
    TOPICS
}

#[derive(Debug)]
pub enum LookupResult {
    Found(&'static HelpTopic),
    Suggestions(Vec<&'static HelpTopic>),
    NoMatch,
}

const SUGGESTION_LIMIT: usize = 3;
const SUGGESTION_MAX_DISTANCE: usize = 2;

pub fn lookup_topic(query: &str) -> LookupResult {
    let normalized = query.trim().to_lowercase();

    for topic in all_topics() {
        if topic.name == normalized {
            return LookupResult::Found(topic);
        }
    }

    for topic in all_topics() {
        for alias in topic.aliases {
            if *alias == normalized {
                return LookupResult::Found(topic);
            }
        }
    }

    let suggestions = compute_suggestions(&normalized);
    if suggestions.is_empty() {
        LookupResult::NoMatch
    } else {
        LookupResult::Suggestions(suggestions)
    }
}

fn compute_suggestions(normalized: &str) -> Vec<&'static HelpTopic> {
    let mut best: HashMap<&'static str, (usize, &'static HelpTopic)> = HashMap::new();

    for topic in all_topics() {
        let candidates = std::iter::once(topic.name).chain(topic.aliases.iter().copied());
        let mut topic_best: Option<usize> = None;
        for candidate in candidates {
            let dist = levenshtein(candidate, normalized);
            if dist <= SUGGESTION_MAX_DISTANCE || candidate.starts_with(normalized) {
                topic_best = Some(match topic_best {
                    Some(d) => d.min(dist),
                    None => dist,
                });
            }
        }
        if let Some(d) = topic_best {
            best.insert(topic.name, (d, topic));
        }
    }

    let mut sorted: Vec<(&'static str, usize, &'static HelpTopic)> = best
        .into_iter()
        .map(|(name, (d, t))| (name, d, t))
        .collect();
    sorted.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(b.0)));
    sorted.truncate(SUGGESTION_LIMIT);
    sorted.into_iter().map(|(_, _, t)| t).collect()
}

pub const INDEX_FOOTER: &str = "Run sq help <topic> for the full guide.";

const SEPARATOR_WIDTH: usize = 50;

pub fn render_index() -> String {
    let mut out = String::new();
    out.push_str(&format!("{}\n", "sq Manual".bold().cyan()));
    out.push_str(&format!("{}\n", "─".repeat(SEPARATOR_WIDTH).dimmed()));

    let max_name_len = CANONICAL_TOPIC_ORDER
        .iter()
        .map(|n| n.len())
        .max()
        .unwrap_or(0);

    for name in CANONICAL_TOPIC_ORDER {
        if let Some(topic) = all_topics().iter().find(|t| t.name == *name) {
            let padded = format!("{:width$}", name, width = max_name_len);
            out.push_str(&format!(
                "  {}  {}\n",
                padded.cyan().bold(),
                topic.summary
            ));
        }
    }

    out.push_str(&format!("{}\n", "─".repeat(SEPARATOR_WIDTH).dimmed()));
    out.push_str(INDEX_FOOTER);
    out.push('\n');
    out
}

pub fn render_topic(topic: &HelpTopic) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "{}\n",
        format!("sq {}", topic.name).bold().cyan()
    ));
    out.push_str(&format!("{}\n", "─".repeat(SEPARATOR_WIDTH).dimmed()));

    out.push_str(&format!("  {}\n", topic.summary));
    out.push('\n');

    if !topic.aliases.is_empty() {
        let alias_list: Vec<String> = topic
            .aliases
            .iter()
            .map(|a| (*a).cyan().to_string())
            .collect();
        out.push_str(&format!(
            "  {} {}\n",
            "Aliases:".yellow().bold(),
            alias_list.join(", ")
        ));
        out.push('\n');
    }

    out.push_str(&format!("  {}\n", "Usage:".yellow().bold()));
    out.push_str(&format!("    {}\n", topic.usage.cyan()));
    out.push('\n');

    out.push_str(&format!("  {}\n", "Examples:".yellow().bold()));
    for ex in topic.examples {
        out.push_str(&format!("    {}\n", ex.cyan()));
    }
    out.push('\n');

    out.push_str(&format!("  {}\n", "Notes:".yellow().bold()));
    out.push_str(&format!("    {}\n", topic.details));

    if !topic.related.is_empty() {
        out.push('\n');
        let related_list: Vec<String> = topic
            .related
            .iter()
            .map(|r| (*r).cyan().to_string())
            .collect();
        out.push_str(&format!(
            "  {} {}\n",
            "See also:".yellow().bold(),
            related_list.join(", ")
        ));
    }

    out
}

pub fn render_no_match(query: &str, suggestions: &[&'static HelpTopic]) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "{} no topic matches \"{}\".\n",
        "sq help:".cyan().bold(),
        query
    ));
    out.push('\n');

    if suggestions.is_empty() {
        out.push_str("No close matches found.\n");
    } else {
        out.push_str(&format!("{}\n", "Did you mean:".yellow().bold()));
        for s in suggestions {
            out.push_str(&format!(
                "  {} — {}\n",
                s.name.cyan().bold(),
                s.summary
            ));
        }
    }

    out.push('\n');
    out.push_str("Run sq help to see all topics.\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn registry_topic_names_are_unique() {
        let mut seen: HashSet<&str> = HashSet::new();
        for topic in all_topics() {
            assert!(
                seen.insert(topic.name),
                "duplicate canonical topic name: '{}'",
                topic.name
            );
        }
    }

    #[test]
    fn registry_aliases_are_unique_and_disjoint_from_names() {
        let canonical: HashSet<&str> = CANONICAL_TOPIC_ORDER.iter().copied().collect();
        let mut seen_aliases: HashSet<&str> = HashSet::new();
        for topic in all_topics() {
            for alias in topic.aliases {
                assert!(
                    !canonical.contains(*alias),
                    "alias '{}' on topic '{}' collides with a canonical topic name",
                    alias,
                    topic.name
                );
                assert!(
                    seen_aliases.insert(*alias),
                    "duplicate alias across registry: '{}' on topic '{}'",
                    alias,
                    topic.name
                );
            }
        }
    }

    #[test]
    fn registry_topics_have_non_empty_authored_copy() {
        for topic in all_topics() {
            assert!(!topic.name.is_empty(), "topic name must not be empty");
            assert!(
                !topic.summary.is_empty(),
                "summary must not be empty for topic '{}'",
                topic.name
            );
            assert!(
                !topic.usage.is_empty(),
                "usage must not be empty for topic '{}'",
                topic.name
            );
            assert!(
                !topic.details.is_empty(),
                "details must not be empty for topic '{}'",
                topic.name
            );
        }
    }

    fn assert_found(query: &str, expected_canonical: &str) {
        match lookup_topic(query) {
            LookupResult::Found(t) => assert_eq!(
                t.name, expected_canonical,
                "lookup({:?}) resolved to '{}', expected '{}'",
                query, t.name, expected_canonical
            ),
            other => panic!(
                "lookup({:?}) returned {:?}, expected Found({})",
                query, other, expected_canonical
            ),
        }
    }

    #[test]
    fn lookup_canonical_name_returns_found_topic() {
        assert_found("status", "status");
        assert_found("journal", "journal");
        assert_found("arena", "arena");
    }

    #[test]
    fn lookup_alias_returns_canonical_topic() {
        assert_found("stat", "status");
        assert_found("inv", "inventory");
        assert_found("wear", "equip");
        assert_found("unequip", "remove");
    }

    #[test]
    fn lookup_normalizes_trim_and_lowercase() {
        assert_found("  STATUS  ", "status");
        assert_found("Stat", "status");
        assert_found("\tjournal\n", "journal");
    }

    #[test]
    fn lookup_typo_within_distance_two_returns_suggestion() {
        match lookup_topic("jounral") {
            LookupResult::Suggestions(s) => {
                assert!(!s.is_empty(), "expected at least one suggestion for 'jounral'");
                assert_eq!(
                    s[0].name, "journal",
                    "first suggestion for 'jounral' must be 'journal'"
                );
            }
            other => panic!("expected Suggestions for 'jounral', got {:?}", other),
        }
    }

    #[test]
    fn lookup_gibberish_returns_no_match() {
        match lookup_topic("xyzzy") {
            LookupResult::NoMatch => {}
            other => panic!("expected NoMatch for 'xyzzy', got {:?}", other),
        }
    }

    #[test]
    fn lookup_does_not_duplicate_canonical_when_alias_also_matches() {
        match lookup_topic("sta") {
            LookupResult::Suggestions(s) => {
                let count = s.iter().filter(|t| t.name == "status").count();
                assert_eq!(
                    count, 1,
                    "'status' must appear at most once when canonical and alias both qualify"
                );
            }
            other => panic!("expected Suggestions for 'sta', got {:?}", other),
        }
    }

    fn best_match_distance(topic: &HelpTopic, query: &str) -> Option<usize> {
        let candidates = std::iter::once(topic.name).chain(topic.aliases.iter().copied());
        let mut best: Option<usize> = None;
        for candidate in candidates {
            let dist = levenshtein(candidate, query);
            if dist <= SUGGESTION_MAX_DISTANCE || candidate.starts_with(query) {
                best = Some(match best {
                    Some(d) => d.min(dist),
                    None => dist,
                });
            }
        }
        best
    }

    fn assert_suggestion_ordering_invariant(query: &str) {
        let suggestions = match lookup_topic(query) {
            LookupResult::Suggestions(s) => s,
            other => panic!("expected Suggestions for {:?}, got {:?}", query, other),
        };
        assert!(!suggestions.is_empty(), "no suggestions for {:?}", query);
        for pair in suggestions.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let da = best_match_distance(a, query).expect("returned suggestion must have a distance");
            let db = best_match_distance(b, query).expect("returned suggestion must have a distance");
            let ordered = da < db || (da == db && a.name <= b.name);
            assert!(
                ordered,
                "ordering invariant violated for query {:?}: {} (d={}) precedes {} (d={})",
                query, a.name, da, b.name, db
            );
        }
    }

    #[test]
    fn lookup_suggestions_obey_ordering_invariant_across_queries() {
        for query in ["re", "h", "i", "s", "in", "st", "j"] {
            if matches!(lookup_topic(query), LookupResult::Suggestions(_)) {
                assert_suggestion_ordering_invariant(query);
            }
        }
    }

    #[test]
    fn lookup_returns_at_most_three_suggestions() {
        if let LookupResult::Suggestions(s) = lookup_topic("") {
            assert!(s.len() <= SUGGESTION_LIMIT, "got {} suggestions", s.len());
        }
    }

    #[test]
    fn render_topic_emits_sections_in_required_order() {
        colored::control::set_override(false);
        let topic = match lookup_topic("status") {
            LookupResult::Found(t) => t,
            other => panic!("expected Found(status), got {:?}", other),
        };

        let out = render_topic(topic);

        let summary_pos = out
            .find(topic.summary)
            .expect("summary text must appear in topic output");
        let aliases_pos = out
            .find("Aliases:")
            .expect("status has an alias and must render an Aliases section");
        let usage_pos = out.find("Usage:").expect("Usage section must appear");
        let examples_pos = out
            .find("Examples:")
            .expect("Examples section must appear");
        let notes_pos = out.find("Notes:").expect("Notes section must appear");
        let see_also_pos = out
            .find("See also:")
            .expect("status has related topics; See also must appear");

        assert!(summary_pos < aliases_pos, "summary must precede Aliases");
        assert!(aliases_pos < usage_pos, "Aliases must precede Usage");
        assert!(usage_pos < examples_pos, "Usage must precede Examples");
        assert!(examples_pos < notes_pos, "Examples must precede Notes");
        assert!(notes_pos < see_also_pos, "Notes must precede See also");
    }

    #[test]
    fn render_topic_skips_aliases_when_topic_has_none() {
        colored::control::set_override(false);
        let topic = match lookup_topic("init") {
            LookupResult::Found(t) => t,
            other => panic!("expected Found(init), got {:?}", other),
        };
        assert!(
            topic.aliases.is_empty(),
            "fixture broken: 'init' is supposed to have no aliases"
        );

        let out = render_topic(topic);
        assert!(
            !out.contains("Aliases:"),
            "Aliases section must be omitted when a topic has none:\n{}",
            out
        );
    }

    #[test]
    fn render_index_includes_canonical_footer_string() {
        colored::control::set_override(false);
        let out = render_index();
        assert!(
            out.contains(INDEX_FOOTER),
            "index must end with the exact footer line; got:\n{}",
            out
        );
    }
}

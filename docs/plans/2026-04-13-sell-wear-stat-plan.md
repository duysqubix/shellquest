# sell/wear/stat/status --full Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add `sq sell`, `sq wear` alias, `sq stat` alias, and `sq status --full` to the shellquest CLI.

**Architecture:** All changes are in `src/main.rs` only. No new files. No changes to display.rs, character.rs, loot.rs, state.rs, or events.rs. Features reuse existing helpers: `loot::item_price()`, `display::print_inventory()`, `display::print_status()`.

**Tech Stack:** Rust, clap 4.x (derive macros), colored 2.x, dirs 5.x

---

## Reference: Key Existing Patterns

Before writing any code, understand these patterns (lines in `src/main.rs`):

- **Home directory guard** (copy from `cmd_buy`, lines 702-717): compare `current_dir()` vs `dirs::home_dir()`
- **Load game state** (pattern from every cmd): `match state::load() { Ok(g) => g, Err(e) => { eprintln!(...); return; } }`
- **Inventory indexed access**: `game.character.inventory` is `Vec<Item>`, 0-indexed in code, 1-indexed for users
- **Sell price**: `loot::item_price(&item) / 2` — integer division of existing formula
- **Save**: `if let Err(e) = state::save(&game) { eprintln!(...); }`
- **clap alias**: `#[clap(alias = "wear")]` on a variant

---

### Task 1: Add `sq wear` alias and `sq stat` alias

**Files:**
- Modify: `src/main.rs:28-29` (Status variant), `src/main.rs:62-66` (Equip variant)

**Step 1: Add `#[clap(alias = "stat")]` to the `Status` variant**

In `src/main.rs`, find (around line 28):
```rust
    /// View your character sheet
    Status,
```

Change to:
```rust
    /// View your character sheet
    #[clap(alias = "stat")]
    Status,
```

**Step 2: Add `#[clap(alias = "wear")]` to the `Equip` variant**

In `src/main.rs`, find (around line 62):
```rust
    /// Equip armor or ring from inventory
    Equip {
        /// Item name (or partial match)
        name: Vec<String>,
    },
```

Change to:
```rust
    /// Equip armor or ring from inventory
    #[clap(alias = "wear")]
    Equip {
        /// Item name (or partial match)
        name: Vec<String>,
    },
```

**Step 3: Build to verify**

```bash
cargo build
```
Expected: Compiles clean. Zero errors.

**Step 4: Manual smoke test**

```bash
sq stat          # should be identical to sq status output
sq wear          # should show same usage error as sq equip (missing name)
```

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "feat: add sq stat alias for status, sq wear alias for equip"
```

---

### Task 2: Add `sq status --full` flag

**Files:**
- Modify: `src/main.rs` — `Status` variant, its match arm, and `cmd_status()` function

**Step 1: Add `full` flag to the `Status` variant**

Find the `Status` variant (now with the `#[clap(alias = "stat")]` from Task 1):
```rust
    /// View your character sheet
    #[clap(alias = "stat")]
    Status,
```

Change to:
```rust
    /// View your character sheet
    #[clap(alias = "stat")]
    Status {
        /// Show full inventory list below the character sheet
        #[clap(long)]
        full: bool,
    },
```

**Step 2: Update the match arm for `Status`**

Find (around line 102):
```rust
        Commands::Status => cmd_status(),
```

Change to:
```rust
        Commands::Status { full } => cmd_status(full),
```

**Step 3: Update `cmd_status()` to accept `full: bool`**

Find (around line 308):
```rust
fn cmd_status() {
    match state::load() {
        Ok(game) => display::print_status(&game.character, game.permadeath),
        Err(e) => eprintln!("{} {}", "❌".bold(), e.red()),
    }
}
```

Change to:
```rust
fn cmd_status(full: bool) {
    match state::load() {
        Ok(game) => {
            display::print_status(&game.character, game.permadeath);
            if full {
                display::print_inventory(&game.character);
            }
        }
        Err(e) => eprintln!("{} {}", "❌".bold(), e.red()),
    }
}
```

**Step 4: Build to verify**

```bash
cargo build
```
Expected: Compiles clean. Zero errors.

**Step 5: Manual smoke test**

```bash
sq status           # unchanged — no inventory
sq status --full    # character sheet + full inventory list below
sq stat --full      # same output as above (alias still works)
```

**Step 6: Commit**

```bash
git add src/main.rs
git commit -m "feat: add --full flag to sq status / sq stat to show full inventory"
```

---

### Task 3: Add `sq sell <number>` command

**Files:**
- Modify: `src/main.rs` — `Commands` enum, match arm in `main()`, new `cmd_sell()` function

**Step 1: Add `Sell` variant to the `Commands` enum**

Find the `Buy` variant (around line 79):
```rust
    /// Buy an item from the shop by number (see `sq shop` for numbered list)
    Buy {
        /// Item number from the shop list
        number: usize,
    },
```

Add the `Sell` variant directly after `Buy`:
```rust
    /// Sell an inventory item at the shop by number (see `sq inventory` for numbered list)
    Sell {
        /// Inventory slot number (1-indexed, from `sq inventory`)
        number: usize,
    },
```

**Step 2: Add the match arm for `Sell` in `main()`**

Find (around line 113):
```rust
        Commands::Buy { number } => cmd_buy(number),
```

Add directly after it:
```rust
        Commands::Sell { number } => cmd_sell(number),
```

**Step 3: Write `cmd_sell()`**

Add the new function directly after `cmd_buy()` (around line 771). Write the complete function:

```rust
fn cmd_sell(number: usize) {
    if number == 0 {
        eprintln!(
            "{} Usage: {} (see {} for numbered list)",
            "❌".bold(),
            "sq sell <number>".cyan(),
            "sq inventory".cyan()
        );
        return;
    }

    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let home = dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    if cwd != home {
        println!(
            "{} The shop is only accessible from your {}.",
            "🏠".bold(),
            "home directory".cyan().bold()
        );
        return;
    }

    let mut game = match state::load() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{} {}", "❌".bold(), e.red());
            return;
        }
    };

    if game.character.inventory.is_empty() {
        println!(
            "{} Nothing to sell. Check {}.",
            "⚠️".yellow(),
            "sq inventory".cyan()
        );
        return;
    }

    let idx = number - 1;
    if idx >= game.character.inventory.len() {
        println!(
            "{} No item at slot {}. You have {} item{}. Check {}.",
            "⚠️".yellow(),
            format!("{}", number).white().bold(),
            format!("{}", game.character.inventory.len()).white().bold(),
            if game.character.inventory.len() == 1 { "" } else { "s" },
            "sq inventory".cyan()
        );
        return;
    }

    let sell_price = loot::item_price(&game.character.inventory[idx]) / 2;
    let item = game.character.inventory.remove(idx);
    let old_gold = game.character.gold;
    game.character.gold += sell_price;

    println!(
        "{} Sold {} (+{} {}) [{}] for {} gold.",
        "💰".bold(),
        item.name.white().bold(),
        item.power,
        format!("{}", item.slot).dimmed(),
        format!("{}", item.rarity).dimmed(),
        format!("{}", sell_price).yellow().bold(),
    );
    println!(
        "   Gold: {} → {}",
        format!("{}", old_gold).dimmed(),
        format!("{}", game.character.gold).yellow().bold(),
    );

    if let Err(e) = state::save(&game) {
        eprintln!("{} Failed to save: {}", "❌".bold(), e.red());
    }
}
```

**Step 4: Build to verify**

```bash
cargo build
```
Expected: Compiles clean. Zero errors.

**Step 5: Manual smoke test**

```bash
# Setup: need items in inventory
sq inventory       # note item numbers and their expected sell prices

cd ~
sq sell 1          # sells slot 1, shows item name + sell price + gold before→after
sq sell 0          # usage error (number == 0)
sq sell 999        # "No item at slot 999. You have N items."

# Empty inventory edge case
# (manually drain inventory via multiple sells, then try)
sq sell 1          # "Nothing to sell."

# Location guard
cd /tmp
sq sell 1          # "The shop is only accessible from your home directory."
cd ~
```

**Verify sell price is correct**: If an item has power 10 and rarity Common (multiplier 5), buy price = (10×5)+5 = 55, sell price = 55/2 = 27. Check gold delta matches.

**Step 6: Commit**

```bash
git add src/main.rs
git commit -m "feat: add sq sell command to sell inventory items at the shop"
```

---

### Task 4: Update README.md commands table

**Files:**
- Modify: `README.md` — commands table section

**Step 1: Update the commands table**

Find the commands table in `README.md`. Add `sq sell`, and note the aliases for `sq wear` and `sq stat`. The table currently has:

```markdown
| `sq equip <name>` | Equip armor or ring from inventory |
| `sq wield <name>` | Wield a weapon from inventory |
```

Add `sq sell` after `sq buy`, add `sq wear` as alias row, add `sq stat` as alias row:

```markdown
| `sq sell <n>` | Sell inventory item at the shop (home directory only) |
| `sq equip <name>` / `sq wear <name>` | Equip armor or ring from inventory |
| `sq wield <name>` | Wield a weapon from inventory |
```

And update the `sq status` row to mention `--full` and `sq stat`:

```markdown
| `sq status` / `sq stat` | View your character sheet (`--full` to include inventory) |
```

**Step 2: Build to verify nothing broke**

```bash
cargo build
```

**Step 3: Commit**

```bash
git add README.md
git commit -m "docs: update commands table for sell, wear alias, stat alias, status --full"
```

---

### Task 5: Final integration check

**Step 1: Run all manual tests end-to-end**

```bash
cargo build                  # must be clean
sq stat                      # identical to sq status
sq stat --full               # status + inventory
sq wear sword                # (if sword in inventory) identical to sq equip sword
cd ~ && sq sell 1            # sells item, shows gold delta
cd /tmp && sq sell 1         # location error
sq sell 0                    # usage error
sq sell 999                  # invalid slot error
```

**Step 2: Verify no regressions on existing commands**

```bash
sq status                    # unchanged
sq equip                     # unchanged (wear is additive alias)
sq inventory                 # unchanged
sq shop                      # unchanged
sq buy 1                     # unchanged
```

**Step 3: Final commit and push (if all green)**

```bash
git push origin master
```

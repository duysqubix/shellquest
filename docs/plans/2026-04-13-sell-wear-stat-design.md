# Design: sq sell, sq wear, sq stat, sq status --full

**Date**: 2026-04-13  
**Status**: Approved

---

## Overview

Four small, cohesive additions to the shop/inventory/status UX surface:

1. **`sq sell <n>`** — Sell an inventory item at the shop for 50% of its buy price
2. **`sq wear <name>`** — Alias for `sq equip` (clap alias, same handler)
3. **`sq stat`** — Alias for `sq status` (clap alias, same handler)
4. **`sq status --full`** — Print full character sheet + complete inventory list

---

## Feature 1: `sq sell <number>`

### Behavior

- **Location guard**: Home directory only. Same guard used by `sq shop` and `sq buy` (compare `current_dir()` vs `dirs::home_dir()`). Non-home CWD → print error + return.
- **Item selection**: 1-indexed from the character's inventory Vec — same numbering shown by `sq inventory`. `sq sell 1` sells the first item in inventory.
- **Sell price**: `loot::item_price(&item) / 2` (integer division, existing function, no new pricing logic).
- **Scope**: Inventory items only. Equipped gear (weapon/armor/ring) cannot be sold — user must unequip first (no `.N` suffix support needed; sell-by-number is unambiguous).
- **All item slots** (Weapon, Armor, Ring, Potion) are sellable.

### Display Output

```
💰 Sold ★ Vorpal Pointer ★ (+18 Weapon) [Epic] for 364 gold.
   Gold: 1,847 → 2,211
```

### Error Cases

| Condition | Message |
|-----------|---------|
| `number == 0` | Usage error: show usage hint |
| Not in home directory | "🏠 The shop is only accessible from your home directory." |
| Inventory empty | "⚠️  Nothing to sell. Check `sq inventory`." |
| `number > inventory.len()` | "⚠️  No item at slot N. You have M items. Check `sq inventory`." |

### Implementation Sketch

```rust
// In Commands enum:
/// Sell an inventory item at the shop (must be in home directory)
Sell {
    /// Inventory slot number (see `sq inventory` for numbered list)
    number: usize,
},

// Match arm:
Commands::Sell { number } => cmd_sell(number),

// New function:
fn cmd_sell(number: usize) {
    // 1. Guard: number == 0 → usage error
    // 2. Guard: cwd != home → shop location error (copy from cmd_buy)
    // 3. Load game state
    // 4. Guard: inventory empty → "Nothing to sell"
    // 5. idx = number - 1; guard idx >= inventory.len() → "No item at slot N"
    // 6. let sell_price = loot::item_price(&game.character.inventory[idx]) / 2;
    // 7. let item = game.character.inventory.remove(idx);
    // 8. game.character.gold += sell_price;
    // 9. Print confirmation with item name, sell price, old→new gold
    // 10. state::save(&game)
}
```

---

## Feature 2: `sq wear` alias for `sq equip`

### Behavior

`sq wear <name>` routes to the exact same `cmd_equip()` handler. No new function. No logic duplication.

### Implementation

```rust
// In Commands enum, change:
/// Equip armor or ring from inventory
Equip {
    name: Vec<String>,
},

// To:
/// Equip armor or ring from inventory
#[clap(alias = "wear")]
Equip {
    name: Vec<String>,
},
```

One line added. Done.

---

## Feature 3: `sq stat` alias for `sq status`

### Behavior

`sq stat` and `sq stat --full` route to the exact same `cmd_status()` handler.

### Implementation

```rust
// In Commands enum, change:
/// View your character sheet
Status,

// To:
/// View your character sheet
#[clap(alias = "stat")]
Status {
    /// Show full inventory in addition to equipment
    #[clap(long)]
    full: bool,
},
```

And update the match arm and handler signature accordingly (see Feature 4).

---

## Feature 4: `sq status --full` (full inventory in status output)

### Behavior

- **Default (`sq status`)**: Unchanged. Calls `display::print_status()` as today.
- **`sq status --full`**: Calls `display::print_status()`, then immediately calls `display::print_inventory()` below it.

No new display functions needed — `print_inventory()` already exists and is used by `sq inventory`.

### Implementation

```rust
// Commands enum: Status variant becomes a struct (see Feature 3 above)

// Match arm update:
Commands::Status { full } => cmd_status(full),

// Handler update:
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

---

## Files to Modify

| File | Change |
|------|--------|
| `src/main.rs` | Add `#[clap(alias = "wear")]` to `Equip` variant |
| `src/main.rs` | Add `#[clap(alias = "stat")]` + `full: bool` field to `Status` variant |
| `src/main.rs` | Update `Status` match arm to pass `full` |
| `src/main.rs` | Update `cmd_status()` signature + body to accept `full: bool` |
| `src/main.rs` | Add `Sell { number: usize }` variant to `Commands` enum |
| `src/main.rs` | Add `Commands::Sell { number } => cmd_sell(number)` match arm |
| `src/main.rs` | Add `fn cmd_sell(number: usize)` function |
| `README.md` | Add `sq sell`, `sq wear`, `sq stat` to commands table |

**No new files. No changes to `display.rs`, `character.rs`, `loot.rs`, `state.rs`, or `events.rs`.**

---

## Non-Goals

- No sell confirmation prompt (keep it snappy like `sq buy`)
- No selling equipped items (inventory only)
- No partial sell / sell quantity > 1
- No sell-by-name (number is unambiguous and consistent with `sq buy`)
- No changes to shop refresh or pricing formula

---

## Testing

Manual testing steps (from `AGENTS.md` pattern):

```bash
# sq wear
sq init        # create char, get a weapon in inventory first via tick loot
sq inv         # note item name
sq wear <name> # should equip same as sq equip

# sq stat alias
sq stat        # should be identical output to sq status
sq stat --full # should show character sheet + full inventory

# sq sell
cd ~
sq inventory   # note item numbers
sq sell 1      # sell first item, verify gold increases by ~50% of buy price
sq sell 99     # invalid slot → error message
sq sell 0      # usage error
cd /tmp && sq sell 1  # location error

# cargo build
cargo build    # must compile clean
```

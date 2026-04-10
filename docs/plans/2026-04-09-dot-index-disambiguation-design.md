# Dot-Index Disambiguation Design

**Date:** 2026-04-09  
**Status:** Approved — ready for implementation

---

## Overview

Extend the existing fuzzy item-matching system with a `.n` suffix syntax so players can unambiguously select the nth match when multiple inventory items match the same query.

---

## Syntax

```
<query>[.<n>]
```

- `n` is a **1-based** positive integer
- Omitting `.n` is equivalent to `.1` (first match)
- If the suffix after the last `.` is not a positive integer, the entire string is treated as the query

### Examples

| Input | Query | n | Result |
|---|---|---|---|
| `potion` | `"potion"` | 1 | First matching potion |
| `potion.1` | `"potion"` | 1 | First matching potion (explicit) |
| `potion.2` | `"potion"` | 2 | Second matching potion |
| `potion.10` | `"potion"` | 10 | Tenth matching potion |
| `Potion.of.Awesome` | `"Potion.of.Awesome"` | 1 | Suffix "Awesome" is non-numeric; full string is query |
| `ring.0` | `"ring"` | 0 | Error: index must be ≥ 1 |
| `sword.99` (only 2 match) | `"sword"` | 99 | Error: only N found |

---

## API Changes

### New: `find_inventory_items`

```rust
/// Returns ALL inventory indices matching `query` (fuzzy, 3-tier cascade), in order.
fn find_inventory_items(game: &state::GameState, query: &str) -> Vec<usize>
```

Implements the same 3-tier cascade as the old `find_inventory_item`:
1. Exact match (case-insensitive)
2. Substring match (`.contains`)
3. Fuzzy token match (`fuzzy_match_name`)

Returns all matches in **inventory order** (stable).

### Changed: `find_inventory_item`

```rust
/// Parses `name` for an optional `.n` suffix, finds all fuzzy matches,
/// returns the nth one (1-indexed).
///
/// Returns:
///   Ok(Some(idx))  — found the nth match at inventory index `idx`
///   Ok(None)       — no items match the query at all
///   Err(msg)       — matches exist but n is out of range; msg is ready-to-print
fn find_inventory_item(game: &state::GameState, name: &str) -> Result<Option<usize>, String>
```

### Parsing logic

```
Split `name` on the last '.'
If suffix is a valid integer:
  - If suffix == 0 → return Err("Item index must be 1 or higher (e.g. potion.1)")
  - Otherwise: query = prefix, n = suffix
Else:
  - query = name (whole string), n = 1
```

---

## Error Messages

| Situation | Output |
|---|---|
| `.0` or negative | `⚠️  Item index must be 1 or higher (e.g. potion.1)` |
| n > count of matches | `⚠️  Only {count} '{query}' item(s) found — use {query}.1 … {query}.{count}` |
| No matches at all | *(handled by existing caller logic: "No item matching X")* |

---

## Caller Changes

`cmd_equip`, `cmd_wield`, `cmd_drink`, `cmd_drop` each gain a third match arm:

```rust
match find_inventory_item(&game, name) {
    Ok(Some(idx)) => { /* proceed as before */ }
    Ok(None)      => { /* "No item matching X" — existing behavior */ }
    Err(msg)      => { println!("{} {}", "⚠️".yellow(), msg); return; }
}
```

---

## Test Plan

### `find_inventory_items` unit tests

| Test | Assertion |
|---|---|
| Empty inventory | returns `[]` |
| Single word matches item containing it | returns `[idx]` |
| Multi-token `"big sword"` | matches items with both "big" AND "sword" |
| Multiple matches returned in inventory order | indices are stable & ascending |
| Case-insensitive | `"POTION"` matches `"Potion of Coffee"` |
| No match | returns `[]` |
| Exact match takes priority over substring | exact appears first |

### `find_inventory_item` (selector parsing) unit tests

| Test | Input | Expected |
|---|---|---|
| No suffix | `"potion"` | `Ok(Some(first_idx))` |
| `.1` explicit | `"potion.1"` | `Ok(Some(first_idx))` |
| `.2` selector | `"potion.2"` | `Ok(Some(second_idx))` |
| Non-numeric suffix treated as query | `"Potion.of"` | matches "Potion of Coffee" at idx |
| `.0` invalid | `"potion.0"` | `Err(contains "1 or higher")` |
| n > match count | `"potion.5"` (2 exist) | `Err(contains "Only 2")` |
| No matches + no suffix | `"xyz"` | `Ok(None)` |
| No matches + valid suffix | `"xyz.2"` | `Ok(None)` |

### Caller integration

- `cmd_drink` with out-of-range index prints `⚠️` to stdout
- `cmd_wield` with bad index does NOT modify save file
- `cmd_equip` with `.2` selects second matching item

---

## Out of Scope

- Shop buy-by-name (`sq buy potion` — still number-based)
- Showing a list when multiple matches exist on a plain query (user confirmed: silently pick #1)
- Negative indices or non-integer suffixes being interpreted as selectors

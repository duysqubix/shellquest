# Dot-Index Disambiguation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add `.n` suffix syntax to all item commands so `sq drink potion.2` selects the second inventory item that fuzzy-matches "potion", with polite errors for out-of-range indices.

**Architecture:** Everything lives in `src/main.rs`. Split `find_inventory_item` into two functions: `find_inventory_items` (returns all matches as a `Vec<usize>`) and a rewritten `find_inventory_item` (parses the `.n` suffix, returns `Result<Option<usize>, String>`). All four callers (`cmd_equip`, `cmd_wield`, `cmd_drink`, `cmd_drop`) gain a third `Err(msg)` arm. TDD throughout — write every test before the implementation.

**Tech Stack:** Rust, `cargo test` (no clippy in this toolchain)

**Design doc:** `docs/plans/2026-04-09-dot-index-disambiguation-design.md`

---

## Key Reference: Existing Code

Before starting, read:
- `src/main.rs` lines 745–771 — `fuzzy_match_name()` and current `find_inventory_item()`
- `src/main.rs` lines 883–961 — `cmd_equip()` (caller pattern to replicate)
- `src/main.rs` lines 963–1027 — `cmd_wield()` (same caller pattern)
- `src/main.rs` lines 1029–1086 — `cmd_drink()` (same caller pattern)
- `src/main.rs` lines 1088–1130 — `cmd_drop_item()` (same caller pattern)
- `src/main.rs` lines 773–881 — existing `mod tests` block (append new tests here)

Existing `find_inventory_item` signature:
```rust
fn find_inventory_item(game: &state::GameState, name: &str) -> Option<usize>
```
This returns on the FIRST match. We are replacing it entirely.

---

## Task 1: Add `find_inventory_items` with failing tests

**Files:**
- Modify: `src/main.rs` (tests block + new function)

**Step 1: Add failing tests to the existing `#[cfg(test)] mod tests` block**

Append these tests inside the existing `mod tests { ... }` in `src/main.rs`:

```rust
// ── find_inventory_items ──────────────────────────────────────

#[test]
fn find_all_empty_inventory_returns_empty() {
    let state = make_state_with_items(vec![]);
    assert_eq!(find_inventory_items(&state, "potion"), Vec::<usize>::new());
}

#[test]
fn find_all_single_word_matches_substring() {
    let state = make_state_with_items(vec![
        item("Potion of Coffee"),
        item("Rusty Pipe"),
        item("Potion of Sorrow"),
    ]);
    assert_eq!(find_inventory_items(&state, "potion"), vec![0, 2]);
}

#[test]
fn find_all_multi_token_requires_all_tokens() {
    let state = make_state_with_items(vec![
        item("Big Sword of Awesome"),
        item("Small Dagger"),
        item("Big Shield"),
    ]);
    // "big sword" requires both tokens → only index 0 matches
    assert_eq!(find_inventory_items(&state, "big sword"), vec![0]);
}

#[test]
fn find_all_case_insensitive() {
    let state = make_state_with_items(vec![
        item("Potion of Coffee"),
        item("Rusty Pipe"),
    ]);
    assert_eq!(find_inventory_items(&state, "POTION"), vec![0]);
}

#[test]
fn find_all_no_match_returns_empty() {
    let state = make_state_with_items(vec![item("Rusty Pipe")]);
    assert_eq!(find_inventory_items(&state, "xyz"), Vec::<usize>::new());
}

#[test]
fn find_all_exact_and_partial_both_included_in_order() {
    // "pipe" matches both exactly and as substring of another item
    let state = make_state_with_items(vec![
        item("Rusty Pipe"),
        item("Pipewright Gauntlets"),
        item("Sword"),
    ]);
    let result = find_inventory_items(&state, "pipe");
    assert_eq!(result, vec![0, 1]);
}

#[test]
fn find_all_returns_stable_inventory_order() {
    let state = make_state_with_items(vec![
        item("Potion of Sorrow"),
        item("Potion of Coffee"),
        item("Rusty Pipe"),
    ]);
    let result = find_inventory_items(&state, "potion");
    assert_eq!(result, vec![0, 1]); // in inventory order, not alphabetical
}
```

**Step 2: Run to confirm compile error**

```bash
cargo test find_all 2>&1 | head -15
```
Expected: compile error — `find_inventory_items` not found.

**Step 3: Implement `find_inventory_items`**

Add this function **between** `fuzzy_match_name` and `find_inventory_item` (around line 752 in `src/main.rs`):

```rust
fn find_inventory_items(game: &state::GameState, query: &str) -> Vec<usize> {
    let query_lower = query.to_lowercase();
    let inv = &game.character.inventory;

    // Collect all indices that match via any of the three tiers.
    // A single pass: check exact first, then contains, then fuzzy —
    // but we want ALL matches, not just the first of each tier.
    // Strategy: union of all three passes, deduplicated, in inventory order.
    let mut matched: Vec<usize> = (0..inv.len())
        .filter(|&i| {
            let name_lower = inv[i].name.to_lowercase();
            name_lower == query_lower
                || name_lower.contains(&query_lower)
                || fuzzy_match_name(&inv[i].name, query)
        })
        .collect();

    // Stable: already in ascending index order from the range iterator.
    matched.dedup();
    matched
}
```

**Step 4: Run tests**

```bash
cargo test find_all 2>&1 | tail -5
```
Expected: 7 passed.

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "Add find_inventory_items returning all fuzzy matches"
```

---

## Task 2: Rewrite `find_inventory_item` with dot-index parsing and failing tests

**Files:**
- Modify: `src/main.rs` (tests block + rewrite function)

**Step 1: Add failing tests for the new `find_inventory_item` signature**

Append to `mod tests` in `src/main.rs`:

```rust
// ── find_inventory_item (dot-index selector) ──────────────────

#[test]
fn selector_no_suffix_returns_first_match() {
    let state = make_state_with_items(vec![
        item("Potion of Coffee"),
        item("Potion of Sorrow"),
    ]);
    assert_eq!(find_inventory_item(&state, "potion"), Ok(Some(0)));
}

#[test]
fn selector_explicit_dot_one_returns_first_match() {
    let state = make_state_with_items(vec![
        item("Potion of Coffee"),
        item("Potion of Sorrow"),
    ]);
    assert_eq!(find_inventory_item(&state, "potion.1"), Ok(Some(0)));
}

#[test]
fn selector_dot_two_returns_second_match() {
    let state = make_state_with_items(vec![
        item("Potion of Coffee"),
        item("Potion of Sorrow"),
    ]);
    assert_eq!(find_inventory_item(&state, "potion.2"), Ok(Some(1)));
}

#[test]
fn selector_dot_zero_returns_err() {
    let state = make_state_with_items(vec![item("Potion of Coffee")]);
    let result = find_inventory_item(&state, "potion.0");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("1 or higher"));
}

#[test]
fn selector_n_exceeds_match_count_returns_err() {
    let state = make_state_with_items(vec![
        item("Potion of Coffee"),
        item("Potion of Sorrow"),
    ]);
    let result = find_inventory_item(&state, "potion.5");
    assert!(result.is_err());
    let msg = result.unwrap_err();
    assert!(msg.contains("Only 2"), "expected 'Only 2' in: {msg}");
    assert!(msg.contains("potion"), "expected query name in: {msg}");
}

#[test]
fn selector_non_numeric_suffix_treated_as_query() {
    // "Potion.of.Coffee" — last suffix is "Coffee", not a number → whole string is query
    let state = make_state_with_items(vec![item("Potion of Coffee")]);
    // The query "Potion.of.Coffee" won't match "Potion of Coffee" via contains or fuzzy
    // because the dot isn't a space. So result is Ok(None).
    assert_eq!(find_inventory_item(&state, "Potion.of.Coffee"), Ok(None));
}

#[test]
fn selector_no_match_no_suffix_returns_ok_none() {
    let state = make_state_with_items(vec![item("Rusty Pipe")]);
    assert_eq!(find_inventory_item(&state, "xyz"), Ok(None));
}

#[test]
fn selector_no_match_with_valid_suffix_returns_ok_none() {
    // No items match "xyz", so even .2 returns Ok(None) (no matches to index into)
    let state = make_state_with_items(vec![item("Rusty Pipe")]);
    assert_eq!(find_inventory_item(&state, "xyz.2"), Ok(None));
}

#[test]
fn selector_dot_n_on_exact_match_works() {
    let state = make_state_with_items(vec![
        item("Rusty Pipe"),
        item("Rusty Sword"),
    ]);
    // "rusty" matches both; .2 should give index 1
    assert_eq!(find_inventory_item(&state, "rusty.2"), Ok(Some(1)));
}
```

**Step 2: Run to confirm compile error**

```bash
cargo test selector_ 2>&1 | head -15
```
Expected: compile error — return type mismatch (`Option` vs `Result`).

**Step 3: Rewrite `find_inventory_item`**

Replace the existing `fn find_inventory_item` (lines ~753–771) with:

```rust
/// Parses an optional `.n` suffix from `name`, finds all fuzzy matches,
/// and returns the nth one (1-indexed).
///
/// Returns:
///   Ok(Some(idx))  — nth match exists at inventory index `idx`
///   Ok(None)       — no items match the query at all
///   Err(msg)       — matches exist but n is out-of-range; msg is ready to print
fn find_inventory_item(game: &state::GameState, name: &str) -> Result<Option<usize>, String> {
    // Parse optional ".n" suffix from the last '.'
    let (query, n) = if let Some(dot_pos) = name.rfind('.') {
        let suffix = &name[dot_pos + 1..];
        match suffix.parse::<usize>() {
            Ok(0) => {
                return Err("Item index must be 1 or higher (e.g. potion.1)".to_string());
            }
            Ok(n) => (&name[..dot_pos], n),
            Err(_) => (name, 1usize), // suffix is not a number → whole string is query
        }
    } else {
        (name, 1usize)
    };

    let matches = find_inventory_items(game, query);

    if matches.is_empty() {
        return Ok(None);
    }

    match matches.get(n - 1) {
        Some(&idx) => Ok(Some(idx)),
        None => Err(format!(
            "Only {} '{}' item(s) found — use {}.1 … {}.{}",
            matches.len(),
            query,
            query,
            query,
            matches.len()
        )),
    }
}
```

**Step 4: Run tests**

```bash
cargo test selector_ 2>&1 | tail -5
```
Expected: 9 passed.

**Step 5: Run full test suite to catch regressions from changed return type**

```bash
cargo test 2>&1 | tail -5
```
Expected: compile errors in callers (`cmd_equip`, `cmd_wield`, `cmd_drink`, `cmd_drop`) — they still treat the return as `Option`. That's expected; fix in Task 3.

**Step 6: Commit (even with broken callers — compile errors are fine at this stage)**

```bash
git add src/main.rs
git commit -m "Rewrite find_inventory_item with dot-index selector (Result return)"
```

---

## Task 3: Update all four callers to handle `Result`

**Files:**
- Modify: `src/main.rs` — `cmd_equip`, `cmd_wield`, `cmd_drink`, `cmd_drop_item`

All four callers have the same pattern:

**OLD (to replace in each function):**
```rust
let idx = match find_inventory_item(&game, name) {
    Some(i) => i,
    None => {
        println!(
            "{} No item matching {} in your inventory.",
            "⚠️".yellow(),
            format!("\"{}\"", name).white().bold()
        );
        return;
    }
};
```

**NEW (replace with in each function):**
```rust
let idx = match find_inventory_item(&game, name) {
    Ok(Some(i)) => i,
    Ok(None) => {
        println!(
            "{} No item matching {} in your inventory.",
            "⚠️".yellow(),
            format!("\"{}\"", name).white().bold()
        );
        return;
    }
    Err(msg) => {
        println!("{} {}", "⚠️".yellow(), msg);
        return;
    }
};
```

Apply this pattern to all four functions: `cmd_equip`, `cmd_wield`, `cmd_drink`, `cmd_drop_item`.

**Step 1: Update all four callers (one edit per function)**

Make the replacement in each of the four functions.

**Step 2: Build clean**

```bash
cargo build 2>&1 | tail -3
```
Expected: no errors.

**Step 3: Run full test suite**

```bash
cargo test 2>&1 | tail -5
```
Expected: all tests pass (106+ tests).

**Step 4: Manual smoke test**

```bash
# Setup: inject two potions into save
# Then test:
cargo run -- drink "potion"     2>&1  # should drink first potion
cargo run -- drink "potion.2"   2>&1  # should drink second potion  
cargo run -- drink "potion.5"   2>&1  # should print "Only N 'potion' items..."
cargo run -- drink "potion.0"   2>&1  # should print "1 or higher"
```

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "Update item command callers to handle Result from find_inventory_item"
```

---

## Task 4: Final verification

**Step 1: Full build and test**

```bash
cargo build 2>&1 | tail -3
cargo test 2>&1 | tail -5
```
Expected: build clean, all tests pass.

**Step 2: End-to-end smoke test**

Inject two potions and two swords into the save file, then verify:

```bash
# sq drink potion       → drinks first potion (Potion of Coffee)
# sq drink potion.2     → drinks second potion (Potion of Sorrow)
# sq drink potion.3     → "Only 2 'potion' item(s) found — use potion.1 … potion.2"
# sq drink potion.0     → "Item index must be 1 or higher (e.g. potion.1)"
# sq wield sword.2      → wields second sword
# sq equip "ring.of"    → matches item named "Ring of Tab Completion" (non-numeric suffix = query)
```

**Step 3: Commit if any fixes needed, then push**

```bash
git log --oneline -5  # review the 3 new commits
git push
```

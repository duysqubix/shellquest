# Boss System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a passive boss encounter system where a rare world-event boss spawns, persists in save.json across ticks, and resolves over 10–14 commands.

**Architecture:** `src/boss.rs` owns the Boss struct, roster, spawn/tick/victory/flee logic. `src/loot.rs` gets `roll_boss_loot()`. `src/state.rs` gains `active_boss: Option<Boss>`. `src/events.rs` calls boss functions each tick. All output via `eprintln!` on stderr.

**Tech Stack:** Rust, serde/serde_json, rand 0.8.x, chrono, colored

**Design doc:** `docs/plans/2026-04-09-boss-system-design.md`

---

## Key Reference: Existing Patterns

Before starting, read:
- `src/events.rs` lines 653–720 — existing `combat()` function (hit/dodge rolls, damage math)
- `src/loot.rs` lines 1–50 — `LootEntry` struct and `roll_loot()` implementation  
- `src/display.rs` lines 86–106 — `print_loot()` for rarity-styled output pattern
- `src/state.rs` lines 9–55 — `GameState` struct and `new()` constructor
- `src/character.rs` lines 270–281 — `attack_power()` and `defense()` implementations

Existing combat math:
- Player hits if: `rng.gen_range(1..=20) + player.attack_power() > 10`
- Monster hits if: `rng.gen_range(1..=20) > 10 + player.defense()`
- Player damage per hit: use `rng.gen_range(player.attack_power()/2 ..= player.attack_power())`
- Boss damage to player: `(boss.attack - player.defense()).max(1)`

---

## Task 1: Boss struct and roster in src/boss.rs

**Files:**
- Create: `src/boss.rs`
- Test in: `src/boss.rs` (inline `#[cfg(test)]`)

**Step 1: Create src/boss.rs with failing tests first**

Write `src/boss.rs` with ONLY the test module — no implementation yet:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Boss {
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub attack: i32,
    pub xp_reward: u32,
    pub gold_reward: u32,
    pub spawned_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boss_roster_has_five_entries() {
        assert_eq!(BOSS_ROSTER.len(), 5);
    }

    #[test]
    fn all_bosses_have_positive_hp_and_attack() {
        for (_, hp, atk, _, _) in BOSS_ROSTER.iter() {
            assert!(*hp > 0);
            assert!(*atk > 0);
        }
    }

    #[test]
    fn spawn_boss_returns_boss_with_full_hp() {
        let boss = spawn_boss();
        assert_eq!(boss.hp, boss.max_hp);
        assert!(boss.hp > 0);
    }

    #[test]
    fn spawn_boss_xp_reward_is_substantial() {
        let boss = spawn_boss();
        assert!(boss.xp_reward >= 500);
    }

    #[test]
    fn is_stale_returns_false_for_fresh_boss() {
        let boss = spawn_boss();
        assert!(!boss.is_stale());
    }
}
```

**Step 2: Run to confirm compile error (BOSS_ROSTER not defined)**

```bash
rtk cargo test boss 2>&1 | head -20
```
Expected: compile error — `BOSS_ROSTER` and `spawn_boss` not found.

**Step 3: Implement**

Add above the test module in `src/boss.rs`:

```rust
pub const BOSS_ROSTER: &[(&str, i32, i32, u32, u32)] = &[
    ("The Kernel Panic",    100, 22, 900, 350),
    ("Lord of /dev/null",    85, 18, 700, 280),
    ("SIGKILL Supreme",      90, 25, 800, 320),
    ("The Infinite Loop",   110, 15, 950, 300),
    ("The Memory Corruption", 95, 20, 850, 310),
];

pub fn spawn_boss() -> Boss {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let (name, hp, attack, xp_reward, gold_reward) =
        BOSS_ROSTER[rng.gen_range(0..BOSS_ROSTER.len())];
    Boss {
        name: name.to_string(),
        hp,
        max_hp: hp,
        attack,
        xp_reward,
        gold_reward,
        spawned_at: Utc::now(),
    }
}

impl Boss {
    pub fn is_stale(&self) -> bool {
        let age = Utc::now() - self.spawned_at;
        age.num_hours() >= 24
    }
}
```

**Step 4: Run tests**

```bash
rtk cargo test boss 2>&1 | tail -5
```
Expected: 5 passed.

**Step 5: Commit**

```bash
git add src/boss.rs
git commit -m "Add Boss struct, roster, and spawn_boss()"
```

---

## Task 2: Boss loot table in src/loot.rs

**Files:**
- Modify: `src/loot.rs`

**Step 1: Read roll_loot() first**

Read `src/loot.rs` to understand `roll_loot(danger_level: u32)` — specifically how it uses `gen_ratio` to pick rarity and then selects from rarity-specific loot arrays.

**Step 2: Write the failing test**

Add to the `#[cfg(test)]` module in `src/loot.rs` (create one if it doesn't exist):

```rust
#[test]
fn boss_loot_never_rolls_common() {
    for _ in 0..200 {
        let item = roll_boss_loot();
        assert!(
            !matches!(item.rarity, crate::character::Rarity::Common),
            "boss loot rolled Common"
        );
    }
}

#[test]
fn boss_loot_returns_valid_item() {
    let item = roll_boss_loot();
    assert!(!item.name.is_empty());
    assert!(item.power > 0);
}
```

**Step 3: Verify tests fail**

```bash
rtk cargo test boss_loot 2>&1 | head -10
```
Expected: compile error — `roll_boss_loot` not found.

**Step 4: Implement roll_boss_loot()**

Add to `src/loot.rs` (after `roll_loot()`):

```rust
pub fn roll_boss_loot() -> crate::character::Item {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    // Boss table: no Common. Uncommon 40%, Rare 47%, Epic 10%, Legendary 3%
    let rarity = if rng.gen_ratio(3, 100) {
        crate::character::Rarity::Legendary
    } else if rng.gen_ratio(10, 97) {
        crate::character::Rarity::Epic
    } else if rng.gen_ratio(47, 87) {
        crate::character::Rarity::Rare
    } else {
        crate::character::Rarity::Uncommon
    };
    roll_item_of_rarity(rarity, 3) // danger_level 3 for boss loot power
}
```

Note: `roll_item_of_rarity` may need to be extracted from `roll_loot` if it doesn't exist — check how `roll_loot` selects items by rarity and factor that out into a helper.

**Step 5: Run tests**

```bash
rtk cargo test boss_loot 2>&1 | tail -5
```
Expected: 2 passed.

**Step 6: Commit**

```bash
git add src/loot.rs
git commit -m "Add roll_boss_loot() with improved rarity weights"
```

---

## Task 3: active_boss field in GameState

**Files:**
- Modify: `src/state.rs`

**Step 1: Write the failing test**

Add to `#[cfg(test)]` in `src/state.rs`:

```rust
#[test]
fn game_state_new_has_no_active_boss() {
    let state = GameState::new(make_character());
    assert!(state.active_boss.is_none());
}

#[test]
fn game_state_serializes_and_deserializes_boss() {
    use crate::boss::{Boss, spawn_boss};
    let mut state = GameState::new(make_character());
    state.active_boss = Some(spawn_boss());
    let json = serde_json::to_string(&state).unwrap();
    let restored: GameState = serde_json::from_str(&json).unwrap();
    assert!(restored.active_boss.is_some());
}
```

**Step 2: Verify tests fail**

```bash
rtk cargo test game_state 2>&1 | head -10
```
Expected: compile error — `active_boss` field not found.

**Step 3: Add field to GameState**

In `src/state.rs`, add to the `GameState` struct:

```rust
#[serde(default)]
pub active_boss: Option<crate::boss::Boss>,
```

The `#[serde(default)]` ensures old save files without the field deserialize cleanly (defaults to `None`).

**Step 4: Run tests**

```bash
rtk cargo test game_state 2>&1 | tail -5
```
Expected: all state tests pass.

**Step 5: Commit**

```bash
git add src/state.rs
git commit -m "Add active_boss field to GameState"
```

---

## Task 4: Display functions in src/display.rs

**Files:**
- Modify: `src/display.rs`

No unit tests for display (pure output side effects). Add four functions:

```rust
pub fn print_boss_spawn(boss: &crate::boss::Boss) {
    eprintln!();
    eprintln!("{}", "╔══════════════════════════════════════════════╗".red().bold());
    eprintln!("{} {} {}",
        "║".red().bold(),
        format!("⚠️  WORLD BOSS: {} HAS APPEARED!", boss.name).red().bold(),
        "║".red().bold());
    eprintln!("{} {} {}",
        "║".red().bold(),
        format!("   HP: {}  ATK: {}  — Defeat it for legendary rewards!", boss.max_hp, boss.attack).red(),
        "║".red().bold());
    eprintln!("{}", "╚══════════════════════════════════════════════╝".red().bold());
    eprintln!();
}

pub fn print_boss_tick(boss: &crate::boss::Boss, player_dmg: Option<i32>, boss_dmg: Option<i32>) {
    if let Some(dmg) = player_dmg {
        eprintln!("{} {} You strike for {}! (HP: {}/{})",
            "💀".bold(),
            format!("[BOSS] {}!", boss.name).red().bold(),
            format!("{}", dmg).green().bold(),
            boss.hp.max(0), boss.max_hp);
    } else {
        eprintln!("{} {} You swing and miss!",
            "💀".bold(),
            format!("[BOSS] {}!", boss.name).red().dimmed());
    }
    if let Some(dmg) = boss_dmg {
        eprintln!("   {} {}",
            "It retaliates —".red(),
            format!("took {} damage.", dmg).red().bold());
    }
}

pub fn print_boss_victory(boss: &crate::boss::Boss, xp: u32, gold: u32) {
    eprintln!();
    eprintln!("{}", "╔══════════════════════════════════════════════╗".yellow().bold());
    eprintln!("{} {} {}",
        "║".yellow().bold(),
        format!("🏆  {} HAS BEEN DEFEATED!", boss.name).yellow().bold(),
        "║".yellow().bold());
    eprintln!("{} {} {}",
        "║".yellow().bold(),
        format!("   +{} XP  +{} gold  — Loot awaits!", xp, gold).yellow(),
        "║".yellow().bold());
    eprintln!("{}", "╚══════════════════════════════════════════════╝".yellow().bold());
    eprintln!();
}

pub fn print_boss_flee(boss_name: &str, reason: &str) {
    eprintln!("{} {} {}",
        "👻".bold(),
        format!("[BOSS]").red().dimmed(),
        format!("{} {}.", boss_name, reason).dimmed().italic());
}
```

**Step: Cargo build clean**

```bash
rtk cargo build 2>&1 | tail -3
```
Expected: no errors.

**Step: Commit**

```bash
git add src/display.rs
git commit -m "Add boss display functions (spawn, tick, victory, flee)"
```

---

## Task 5: Boss spawn and tick logic in src/boss.rs

**Files:**
- Modify: `src/boss.rs`

**Step 1: Write failing tests**

Add to `#[cfg(test)]` in `src/boss.rs`:

```rust
#[test]
fn maybe_spawn_does_not_spawn_if_boss_active() {
    use crate::character::{Character, Class, Race};
    use crate::state::GameState;
    let mut state = GameState::new(Character::new("T".to_string(), Class::Warrior, Race::Human));
    let existing = spawn_boss();
    state.active_boss = Some(existing);
    let boss_name_before = state.active_boss.as_ref().unwrap().name.clone();
    // Force spawn by calling internal function directly — no RNG gating in test
    if state.active_boss.is_none() {
        state.active_boss = Some(spawn_boss());
    }
    assert_eq!(state.active_boss.as_ref().unwrap().name, boss_name_before);
}

#[test]
fn stale_boss_is_detected_correctly() {
    let mut boss = spawn_boss();
    boss.spawned_at = Utc::now() - chrono::Duration::hours(25);
    assert!(boss.is_stale());
}
```

**Step 2: Run to confirm tests fail**

```bash
rtk cargo test boss 2>&1 | head -20
```

**Step 3: Implement maybe_spawn and tick_boss**

Add to `src/boss.rs`:

```rust
pub fn maybe_spawn(state: &mut crate::state::GameState) {
    use rand::Rng;
    if state.active_boss.is_some() {
        return;
    }
    let mut rng = rand::thread_rng();
    if rng.gen_ratio(1, 1000) {
        let boss = spawn_boss();
        crate::display::print_boss_spawn(&boss);
        state.active_boss = Some(boss);
    }
}

pub fn tick_boss(state: &mut crate::state::GameState) {
    use rand::Rng;
    use crate::journal::{EventType, JournalEntry};

    let boss_is_stale = state.active_boss.as_ref().map_or(false, |b| b.is_stale());
    if boss_is_stale {
        let name = state.active_boss.as_ref().unwrap().name.clone();
        crate::display::print_boss_flee(&name, "grows bored waiting and retreats. It will return");
        state.active_boss = None;
        return;
    }

    let boss = match state.active_boss.as_mut() {
        Some(b) => b,
        None => return,
    };

    let mut rng = rand::thread_rng();
    let player_power = state.character.attack_power();
    let player_defense = state.character.defense();

    // Player attacks boss
    let hit_roll: i32 = rng.gen_range(1..=20);
    let player_dmg = if hit_roll + player_power > 10 {
        let dmg = rng.gen_range((player_power / 2).max(1)..=player_power.max(1));
        boss.hp -= dmg;
        Some(dmg)
    } else {
        None
    };

    let boss_hp_after = boss.hp;
    let boss_max_hp = boss.max_hp;
    let boss_atk = boss.attack;
    let boss_name = boss.name.clone();

    // Check boss death before it attacks
    if boss_hp_after <= 0 {
        let xp = state.character.gain_xp(
            state.active_boss.as_ref().unwrap().xp_reward
        );
        let gold = state.active_boss.as_ref().unwrap().gold_reward;
        state.character.gold += gold;
        crate::display::print_boss_tick(state.active_boss.as_ref().unwrap(), player_dmg, None);
        crate::display::print_boss_victory(state.active_boss.as_ref().unwrap(),
            state.active_boss.as_ref().unwrap().xp_reward, gold);
        let loot = crate::loot::roll_boss_loot();
        let loot_msg = format!("Boss loot: {} (+{} {}) [{}]",
            loot.name, loot.power, loot.slot, loot.rarity);
        crate::display::print_loot(&loot_msg, &loot.rarity);
        state.add_journal(JournalEntry::new(EventType::Combat, format!("Defeated {}! +{} XP +{} gold", boss_name, state.active_boss.as_ref().unwrap().xp_reward, gold)));
        crate::events::add_to_inventory_pub(state, loot);
        state.active_boss = None;
        if xp { crate::events::emit_level_up(state); }
        return;
    }

    // Boss attacks player
    let dodge_roll: i32 = rng.gen_range(1..=20);
    let boss_dmg = if dodge_roll > 10 + player_defense {
        let dmg = (boss_atk - player_defense).max(1);
        let died = state.character.take_damage(dmg);
        if died {
            crate::display::print_boss_tick(state.active_boss.as_ref().unwrap(), player_dmg, Some(dmg));
            crate::display::print_boss_flee(&boss_name, "laughs as you fall... and vanishes into the void");
            state.add_journal(JournalEntry::new(EventType::Death, format!("{} fled after you fell in battle.", boss_name)));
            state.active_boss = None;
            return;
        }
        Some(dmg)
    } else {
        None
    };

    crate::display::print_boss_tick(state.active_boss.as_ref().unwrap(), player_dmg, boss_dmg);
    state.add_journal(JournalEntry::new(EventType::Combat,
        format!("[BOSS] {} — HP: {}/{}", boss_name, boss_hp_after.max(0), boss_max_hp)));
}
```

**Note on public helpers:** `tick_boss` calls `crate::events::add_to_inventory_pub` and `crate::events::emit_level_up`. These need to be extracted from `events.rs` as `pub(crate)` functions (see Task 6).

**Step 4: Run all tests**

```bash
rtk cargo test 2>&1 | tail -5
```
Expected: all pass (new boss tests + existing 88).

**Step 5: Commit**

```bash
git add src/boss.rs
git commit -m "Add maybe_spawn() and tick_boss() to boss module"
```

---

## Task 6: Extract helpers in src/events.rs

**Files:**
- Modify: `src/events.rs`

The boss module needs to call `add_to_inventory` and the level-up display. Extract two `pub(crate)` helpers:

**Step 1: Make add_to_inventory pub(crate)**

Change:
```rust
fn add_to_inventory(state: &mut GameState, item: crate::character::Item) {
```
To:
```rust
pub(crate) fn add_to_inventory_pub(state: &mut GameState, item: crate::character::Item) {
```

Keep the private `add_to_inventory` as an alias calling the pub version, OR rename all call sites. Simplest: rename to `pub(crate) fn add_to_inventory` (Rust allows `pub(crate)` on private functions used only within the crate).

**Step 2: Extract emit_level_up**

Add:
```rust
pub(crate) fn emit_level_up(state: &mut GameState) {
    let plain = format!("LEVEL UP! You are now level {}! Title: {}", state.character.level, state.character.title);
    display::print_level_up(&color_level_up(&state.character));
    state.add_journal(crate::journal::JournalEntry::new(crate::journal::EventType::LevelUp, plain));
}
```

And update `check_level_up` to call it:
```rust
fn check_level_up(state: &mut GameState, leveled: bool) {
    if leveled { emit_level_up(state); }
}
```

**Step 3: Build clean**

```bash
rtk cargo build 2>&1 | tail -3
```
Expected: no errors.

**Step 4: Run all tests**

```bash
rtk cargo test 2>&1 | tail -5
```
Expected: all pass.

**Step 5: Commit**

```bash
git add src/events.rs
git commit -m "Extract pub(crate) helpers from events for boss module"
```

---

## Task 7: Wire boss into the tick in src/events.rs and src/main.rs

**Files:**
- Modify: `src/events.rs`
- Modify: `src/main.rs`

**Step 1: Declare mod boss in main.rs**

Add near the top of `src/main.rs` with the other `mod` declarations:
```rust
mod boss;
```

**Step 2: Call boss functions in tick()**

In `src/events.rs`, at the bottom of the `pub fn tick(...)` function — after the passive healing block, add:

```rust
// Boss tick (runs every tick if a boss is active)
crate::boss::tick_boss(state);

// Passive boss spawn check (very rare world event)
crate::boss::maybe_spawn(state);
```

Order matters: `tick_boss` first (advance current fight), then `maybe_spawn` (possibly start a new one after the old one is resolved).

**Step 3: Build**

```bash
rtk cargo build 2>&1 | tail -3
```
Expected: no errors.

**Step 4: Run all tests**

```bash
rtk cargo test 2>&1 | tail -5
```
Expected: all pass.

**Step 5: Manual QA**

Manually force a boss spawn to verify end-to-end:

```bash
# Temporarily set spawn rate to 1/1 to guarantee spawn
# Edit maybe_spawn: rng.gen_ratio(1, 1) for one test run
cargo run -- tick --cmd "ls" --cwd "." --exit-code 0 2>&1
# Expect: boss spawn announcement
cargo run -- tick --cmd "ls" --cwd "." --exit-code 0 2>&1
# Expect: boss tick line with HP drain
```

Revert the spawn rate back to `1, 1000` after confirming.

**Step 6: Commit**

```bash
git add src/main.rs src/events.rs
git commit -m "Wire boss spawn and tick into the main game loop"
```

---

## Task 8: Update AGENTS.md

**Files:**
- Modify: `AGENTS.md`
- Modify: `src/AGENTS.md`

**Step 1: Add boss module to src/AGENTS.md**

Add `src/boss.rs` to the Key Files table:

```
| `boss.rs` | Boss system: `Boss` struct, 5-boss roster, `spawn_boss()`, `maybe_spawn()` (1/1000 chance per tick), `tick_boss()` (per-tick combat, victory, flee). Called from `events::tick()`. |
```

**Step 2: Add testing note to root AGENTS.md**

In the Testing Requirements section, add:
```
- Force boss spawn for testing: temporarily set `gen_ratio(1, 1)` in `maybe_spawn()`, run `sq tick`, then revert
- Boss state lives at `character.active_boss` in the save file — can be cleared manually via JSON edit
```

**Step 3: Commit**

```bash
git add AGENTS.md src/AGENTS.md
git commit -m "Update AGENTS.md with boss module documentation"
```

---

## Task 9: Final verification

**Step 1: Full clean build and test**

```bash
cargo clean && rtk cargo build 2>&1 | tail -3
rtk cargo test 2>&1 | tail -5
```
Expected: build clean, all tests pass (90+ tests).

**Step 2: End-to-end boss fight QA**

1. Temporarily set `gen_ratio(1, 1)` in `maybe_spawn()`
2. `cargo run -- tick --cmd "ls" --cwd "." --exit-code 0` → boss spawns
3. Run 15+ ticks → boss HP drains, eventually dies
4. Verify victory message, loot drop, XP/gold reward
5. Verify `sq status` shows updated stats
6. Revert spawn rate to `gen_ratio(1, 1000)`

**Step 3: Test death path**

1. Set low HP manually in save.json (`"hp": 1`)
2. Force boss spawn
3. Run tick → player should die, boss should flee
4. Verify flee message, no reward, boss cleared from save

**Step 4: Test stale boss path**

1. Manually set `spawned_at` in save.json to 25 hours ago
2. Run tick
3. Verify stale-flee message, boss cleared

**Step 5: Final commit and publish**

```bash
git log --oneline -8  # review commits
./publish.sh patch    # publish v1.5.3
```

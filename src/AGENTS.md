<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-04-09 | Updated: 2026-04-09 -->

# src

## Purpose
All Rust source code for the `sq` binary. Organized as a flat module structure — `main.rs` declares modules and implements CLI commands, while each module owns a distinct game domain (character data, game events, loot tables, zone mapping, display rendering, journal logging, save/load persistence).

## Key Files

| File | Description |
|------|-------------|
| `main.rs` | Entry point: CLI definition (clap derive), subcommand handlers (`init`, `status`, `inventory`, `journal`, `tick`, `hook`, `shop`, `buy`, `equip`, `wield`, `drink`, `drop`, `prestige`, `reset`, `update`), shell hook code generation and installation |
| `boss.rs` | Boss system: `Boss` struct, 5-boss roster, `spawn_boss()`, `maybe_spawn()` (1/1000 chance per tick), `tick_boss()` (per-tick combat, victory, flee). Called from `events::tick()`. |
| `character.rs` | Core data model: `Character` struct, `Class` (5), `Race` (5), `Subclass` (15), `Item`, `ItemSlot`, `Rarity` enums, stat calculations, XP/leveling curve (max 150), prestige system, equip/damage/heal logic |
| `messages.rs` | Class-aware message module: 20 event functions returning `(plain, colored)` tuples. One function per event type, 5 class variants each (Wizard/Warrior/Rogue/Ranger/Necromancer). Called from `events.rs` handlers. |
| `events.rs` | Game engine: `tick()` dispatches command-specific events (30+ handlers), combat system with d20-style rolls, probability-gated encounters via `gen_ratio()`, auto-equip and inventory management. Uses `scaled_xp(base, danger_level)` for zone-danger XP multiplier (1.0×–2.0×) and `affinity_multiplier(class, cmd)` for class affinity (+50% XP for class-matching commands). All event messages route through `messages::*` functions. |
| `display.rs` | Terminal rendering: colored output helpers, rarity-tiered loot formatting (Common through Legendary with box-drawing), status sheet with HP/XP bars, inventory list, journal display. All game output uses `eprintln!` |
| `loot.rs` | Loot system: 150+ items across 5 rarity tiers with weighted drop rates (Common 70%, Uncommon 25%, Rare 4%, Epic 0.99%, Legendary 0.01%), organized by slot (Weapon, Armor, Ring, Potion) with power ranges |
| `zones.rs` | Zone mapping: converts filesystem paths to themed zones (e.g., `/tmp` = "The Wasteland", `node_modules` = "The Abyss") with danger levels and colors, used for scaling combat and loot |
| `sage.rs` | Update notifier: checks crates.io every 24h, guarantees sage appears once on the first tick a new version is detected, then falls back to 1/50 random (max 3×/day). Tracks `last_announced_version` in `GameState` to prevent repeat first-time announcements. |
| `state.rs` | Persistence: `GameState` struct wrapping character + journal + timestamps + `permadeath: bool` + version-check cache (`latest_version`, `last_announced_version`), atomic save via temp-file-then-rename to `~/.shellquest/save.json` |
| `journal.rs` | Journal system: `JournalEntry` with timestamp + `EventType` enum (Combat, Loot, Travel, Discovery, LevelUp, Death, Quest, Craft), capped at 100 entries |

## For AI Agents

### Working In This Directory
- All modules are declared in `main.rs` with `mod` — no `lib.rs` or nested modules
- `events.rs` is the largest file (~756 lines) and the core game loop — changes here affect gameplay balance
- Display uses a two-pass pattern: build a `plain` string for journal storage, then a `colored` string for terminal output — both must stay in sync
- Loot tables in `loot.rs` use `const` arrays of `LootEntry` structs — add new items by appending to the appropriate rarity tier array
- The `tick()` function in `events.rs` matches on the base command name (first word, lowercased) and routes to specific handlers
- Combat uses a d20-style system: `hit_roll + attack_power > 10` for player hits, `dodge_roll > 10 + defense` for monster hits
- XP curve scales by level brackets in `character.rs:level_up()` — early levels are fast, late levels are slow
- `MAX_LEVEL` is 150, after which the player must prestige to continue gaining XP

### Testing Requirements
- `cargo build` — must compile without errors (`cargo clippy` is not installed in the current toolchain — skip it)
- Manually test with: `sq tick --cmd "git commit" --cwd "/tmp" --exit-code 0` (triggers craft event)
- Test failed commands: `sq tick --cmd "bad" --cwd "." --exit-code 1` — triggers trap at 25% chance (run several times to confirm ~1/4 trigger rate)
- `cd ~ && sq shop` → shop only works from home directory; shows numbered item list
- `cd ~ && sq buy 1` → buy item by **number** (1-indexed), not by name
- Verify loot balance: ensure new items have power ranges consistent with their rarity tier

### Common Patterns
- **Event handler signature**: `fn handle_*(state: &mut GameState, rng: &mut impl Rng)` — some take `cwd: &str` for zone-aware events
- **Probability gates**: `rng.gen_ratio(numerator, denominator)` — e.g., `gen_ratio(1, 3)` = 33% chance
- **Level-up check**: call `check_level_up(state, leveled)` after any `gain_xp()` call
- **Auto-equip flow**: `roll_loot()` -> `auto_equip()` which compares power, equips if better, otherwise `add_to_inventory()` (max 20 items, drops weakest)
- **Color helpers**: `display::color_damage()`, `color_xp()`, `color_gold()`, `color_hp()`, `color_monster()`, `color_item_inline()`, `color_zone()` for consistent inline formatting
- **Class-aware messages**: `crate::messages::FUNCTION(&state.character.class, ...)` returns `(plain, colored)` — use plain for journal, colored for display
- **Zone-scaled XP**: `final_xp(base, zone.danger_level, &state.character.class, cmd)` applies both zone and affinity bonuses

## Dependencies

### Internal
- All modules depend on `character.rs` types (`Character`, `Item`, `Rarity`, `Class`, etc.)
- `events.rs` depends on `display`, `journal`, `loot`, `state`, and `zones`
- `display.rs` depends on `character` and `journal` types, plus `zones::Zone`/`ZoneColor`
- `main.rs` depends on all modules

### External
- `clap` — CLI parsing (only in `main.rs`)
- `colored` — used in `main.rs`, `display.rs`, `events.rs`
- `rand` — used in `events.rs`, `loot.rs`, `zones.rs`
- `serde` / `serde_json` — used in `character.rs`, `journal.rs`, `state.rs`
- `chrono` — used in `journal.rs`, `state.rs`, `main.rs`
- `dirs` — used in `state.rs`, `zones.rs`

<!-- MANUAL: Any manually added notes below this line are preserved on regeneration -->

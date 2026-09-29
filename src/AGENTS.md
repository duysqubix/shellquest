<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-04-09 | Updated: 2026-09-29 -->

# src

## Purpose
All Rust source code for the `sq` binary. Flat module structure — `main.rs` declares every module (no `lib.rs`) and implements CLI commands; each module owns a distinct game domain. ~20.6k lines: `main.rs` plus 15 modules. Overworld combat lives in `events.rs`; the arena is a separate, larger combat system in `arena.rs`; `telemetry.rs` carries the `SQ_DEBUG` diagnostics channel.

## Key Files

| File | ~Lines (v1.28) | Description |
|------|------:|-------------|
| `arena.rs` | 3.3k | Arena system: 5 tiers, `ArenaCombatTuning`, wave escalation (`arena_wave`), DEX-driven miss math (`compute_player_hit`/`compute_enemy_hit_at_round`), 1.5s pacing (`SQ_NO_PACING` opt-out), transactional commit + chest overflow. Owns its own combat math, separate from `events.rs`. |
| `main.rs` | 3.2k | Entry point: clap `Commands` enum (**25** subcommands incl. read-only `Items {--json}` / `Bestiary {--json}` catalog dumps), handlers, the `sq hook --install` rc-file upgrade (`cmd_hook`; the hook code lives in `hook.rs`). Home-dir guards for `shop`/`buy`/`sell`/`enchant`/`quest`. `cmd_tick` is silent if no save (logs + exits non-zero under `SQ_DEBUG`). |
| `events.rs` | 3.2k | Overworld engine: `tick()` routes **~39 command arms / 37 handlers** (first word, lowercased) over ~140 recognized commands; `affinity_multiplier()` per-class +50% XP; HP-pool **multi-turn** combat (≤30 turns/tick); `MonsterTier` + 40-monster `MONSTER_POOL`; `tiers_for_danger()` gating; `monster_bestiary()`/`tier_danger()`/`elite_modifiers()` feed `sq bestiary`. Emits `SQ_ENCOUNTER` per fight under `SQ_DEBUG`. Boss hooks via `boss::maybe_spawn`/`tick_boss`. Void zone: 18-mob roster (`VOID_MOB_POOL`) with depth+level scaling via `random_void_mob()`/`void_enemy_dex_mod()`. |
| `messages.rs` | 2.3k | Class-flavored message templates. `type Msg = (String, String)` — every fn returns `(plain, colored)`. 5 class variants per event. |
| `loot.rs` | 1.5k | Loot tables: **160 items** (5 rarities × 32; 8 per slot). Drop weights 70/25/4/0.99/0.01. Power mults 5/8/13/22/35. `item_price=(power+1)×mult`. `catalog()`/`CatalogEntry`/`rarity_weights()`/`rarity_multiplier()` feed `sq items --json`. |
| `display.rs` | 1.4k | Terminal rendering (all `eprintln!`): color helpers, rarity-tiered loot boxes, status sheet with ATK/DEF gear breakdown, inventory grouped by slot, journal re-coloring by `EventType`. |
| `character.rs` | 1.3k | Core data model: `Character`, `Class`(5), `Race`(5), `Subclass`(15), `Item`(+`enchant_level`), `ItemSlot`, `Rarity`. Stat/attack/defense math, XP curve (`MAX_LEVEL` 150), prestige, `signature_bonus` (class passives). |
| `hook.rs` | 450 | Shell hook code per shell (`code`), the rc-file loader `--install` writes (`loader`), and `plan_rc`, which finds the exact hooks every past version printed (plus `# >>> shellquest hook (<shell>) >>>` blocks) so they can be upgraded in place. `VERSION` is passed as `sq tick --hook=N`. End-to-end tests in `tests/shell_hooks/` drive real zsh/bash through a pty. |
| `help.rs` | 760 | In-game manual: topic registry, fuzzy topic lookup, rendered `sq help [topic]` pages. |
| `zones.rs` | 590 | Path→zone mapping (**34 themed zones**, `$HOME`=Home Village .. `node_modules`=Abyss, `the_void`=The Void) with danger levels + colors. `has_segment` (case-insensitive) vs `has_exact_segment` (case-sensitive, for capitalized user dirs like `Documents`). `zone_from_path()` is the entry point. `is_void_zone()`/`void_depth()` support Void-specific encounter scaling. |
| `boss.rs` | 800 | 5-boss roster (`BOSS_ROSTER`, HP **440–540**, ATK 36–54 at L25), spawning only from `BOSS_MIN_LEVEL` 25 and scaled by `level_scale()` = (L+`BOSS_SCALE_OFFSET`)/30 (HP, ATK, XP, gold); `maybe_spawn()` (**1/500**/tick = `BOSS_SPAWN_RATE`); `flee()` backs `sq flee` (10% gold); a boss never kills a permadeath character (1 HP, boss leaves); an ignored `duel_harness` test measures win rates by level; `tick_boss()` HP-pool combat (one exchange/tick; `dmg_dealt_total`/`dmg_taken_total` accumulate across ticks, emit `SQ_ENCOUNTER` at win/loss/flee). `BossInfo`/`boss_roster()` feed `sq bestiary`. |
| `sage.rs` | 230 | Update notifier: crates.io check every 24h (via `ureq`), guaranteed once on first new-version tick, then 1/50 random (max 3×/day). |
| `state.rs` | 280 | `GameState` (character + journal + caches + `active_boss` + `permadeath`). Atomic save: temp-file + rename to `~/.shellquest/save.json`. Quest save fields (all `#[serde(default)]`): `quest_refreshed` (last UTC-day the quest was generated), `quest_phrase` (the secret phrase hidden in the maze), `quest_scroll_path` (path to the scroll file, cleared on answer), `quest_completed_today` (prevents double-claim). |
| `telemetry.rs` | 100 | **`SQ_DEBUG` diagnostics channel** (declared `mod telemetry` in `main.rs`). `sq_debug_enabled()`, `emit_encounter()` (prints one `SQ_ENCOUNTER` line per resolved fight to stderr, only when `SQ_DEBUG` set), hex enemy-name encoder. Consumed by the balance sim. |
| `journal.rs` | 70 | `JournalEntry` + `EventType` enum (Combat/Loot/Travel/Discovery/LevelUp/Death/Quest/Craft/Tournament), capped at 100. |
| `void.rs` | 560 | Procedural maze generator: `generate_void(rng)` / `clear_void()` / `hide_file_in_void(root, contents, rng)`. Bounded depth 7 (`MAX_DEPTH`), acyclic symlink rifts (`create_rifts`), strictly contained under save dir (`ensure_void_contained`). `void_root()` returns `~/.shellquest/the_void`. Daily reshuffle calls `clear_void_in` before regenerating — orphan prevention is structural. |

## For AI Agents

### Working In This Directory
- All modules declared in `main.rs` with `mod` — no `lib.rs` or nested modules
- Hotspots (by size/complexity): `arena.rs` (~3.3k lines), `main.rs` (~3.2k), `events.rs` (~3.2k), `messages.rs` (~2.3k), `loot.rs` (~1.5k)
- Display uses a two-pass pattern: a `plain` string for journal storage, a `colored` string for terminal output — both must stay in sync or journal history and screen diverge
- Loot tables in `loot.rs` are `const` arrays — add items by appending to the right rarity tier (keep 8-per-slot symmetry)
- `tick()` in `events.rs` matches on the base command name (first word, lowercased) and routes to handlers
- **Combat is HP-pool multi-turn** (not single-roll): every monster has HP; the fight loops up to 30 turns/tick (`COMBAT_MAX_TURNS`), draw if neither dies. d20 to-hit + d20-dodge; crits per-turn at `max(13, 20−INT/8)` for 2×; class signatures fire every turn. `arena.rs` has its own parallel combat resolver (wave-scaled).
- Monsters spawn by zone danger via `tiers_for_danger()`: 1=Vermin · 2=Vermin+Bruiser · 3=Bruiser+Hunter · 4=Hunter+Horror · 5+=Horror+Boss-adjacent
- XP curve scales by level brackets in `character.rs` `level_up_core()`; `MAX_LEVEL` is 150
- **Arena Transactions**: built as an `ArenaCommit`, applied in one atomic save. Runs are not resumable; `Ctrl+C` mid-run = full rollback (including entry fee).

### Key Public Types (know these before editing; grep the name, line numbers drift)
- `MonsterTier` + `MONSTER_POOL` + `tiers_for_danger` (`events.rs`) — Vermin/Bruiser/Hunter/Horror/BossAdjacent
- `ArenaTier` + `ArenaCombatTuning` (`arena.rs`) — tier table + all arena combat constants
- `Class`/`Race`/`Subclass`/`Rarity`/`ItemSlot`/`Item`/`Character` (`character.rs`) — shared data model; `MAX_LEVEL` = 150
- `GameState` (`state.rs`) — the persisted save root

### Testing Requirements
- Unit tests are **in-file** `#[cfg(test)] mod tests`. Integration tests in `tests/` run the real binary with a temporary HOME; `tests/concurrent_saves.rs` runs 8 simultaneous ticks per round.
- **~457 `#[test]` total** (456 run + 1 `#[ignore]`d, as of v1.28). Heaviest coverage: arena ~116, main ~72, character ~71, events ~57, zones ~37; `sage.rs` has none and `messages.rs` only 3. Add a guard test when adding a balance constant.
- `cargo build` (or `cargo build --bin sq`) to compile; `cargo test` for the suite; `cargo clippy --all-targets` for lints (installed; pre-existing warnings are not yet fatal — don't add new ones). `just check` runs every gate.
- Cargo cache quirk lives in the parent `../AGENTS.md`; manual QA runs only through `dev-tools/sq-sandbox` (never the real HOME) with the scenario recipes in `.claude/skills/sq-qa/SKILL.md` — don't duplicate here. Arena tests rely on seeded `StdRng`; combat fns take `&mut impl Rng` so they're deterministic in tests.
- When adding a balance constant, add or update its guard test in `arena.rs` `mod tests` (the wave/tuning invariants are pinned there).

### Common Patterns
- **Event handler signature**: `fn handle_*(state: &mut GameState, rng: &mut impl Rng)` — some take `cwd: &str` for zone-aware events
- **Probability gates**: `rng.gen_ratio(numerator, denominator)` — e.g., `gen_ratio(1, 3)` = 33% chance
- **Level-up check**: call `check_level_up(state, leveled)` after any `gain_xp()` call
- **Loot flow**: `roll_loot*()` → `add_to_inventory()` (events.rs). There is no auto-equip. The cap is 20, and it is also hard-coded in arena.rs and main.rs. When full, it drops the weakest Common–Rare item by raw power and never an Epic/Legendary; callers must check its `bool` result
- **Color helpers**: `display::color_damage()`, `color_xp()`, `color_gold()`, `color_hp()`, `color_monster()`, `color_item_inline()`, `color_zone()` for consistent inline formatting
- **Class-aware messages**: `crate::messages::FUNCTION(&state.character.class, ...)` returns `(plain, colored)` — use plain for journal, colored for display
- **Zone-scaled XP**: `final_xp(base, zone.danger_level, &state.character.class, cmd)` applies both zone and affinity bonuses

## Invariants & Anti-Patterns (source-level)
- **Arena math never reads live `Character` mid-run** (`ArenaEntrySnapshot`). Use the `ArenaEntrySnapshot` frozen at entry — otherwise low-level chars power-level inside a run.
- **Never store `ArenaRun` in `GameState`** (`ArenaRun`). It's transient; only the `ArenaCommit` touches the save.
- **Deferred arena output renders only AFTER `state::save()` succeeds** (`ArenaDeferredOutput`, `render_arena_deferred_output`) — never inline during `apply_arena_commit`.
- **MAX_LEVEL entrants get XP suppressed at the commit boundary** (`build_commit`, the `run.entry.level >= MAX_LEVEL` branch) — don't "fix" this; it's intentional.
- **Read-modify-write commands hold the save lock from load to save.** Use `load_for_update()` in main.rs; it wraps `state::lock` and `state::load`.
  - Never hold the lock across an interactive prompt or a network call, because every shell's tick waits on it. Tick uses `TICK_LOCK_TIMEOUT` (1 s) and skips on contention, and it fetches the daily crates.io version before locking.
  - Arena and prestige still load before their prompts and save at the end without the lock; that is shellqeuest-x3p.2.
  - Delete a character only with `state::delete_save()`, which also removes the backup; `sq init` uses `state::save_new()`.
- **New save fields MUST be `#[serde(default)]`** (e.g. `Item::enchant_level`, the quest fields on `GameState`) or old `save.json` fails to load.
- **Two-pass messages must stay in sync** — store `plain` in journal, print `colored`; never store ANSI text (it gets re-colored by `EventType` in `display.rs`).
- **`tick` stays fast + silent**: no character → return silently (`cmd_tick` in `main.rs`); save failure only logs to stderr, never panics. **Exception**: when `SQ_DEBUG` is set (dev/sim diagnostics; mirrors `SQ_NO_PACING`), `cmd_tick` logs load/save failures with context and exits non-zero. Default (unset) behavior is unchanged.
- **Combat telemetry (`SQ_DEBUG`)**: `combat()` (`events.rs`) and `tick_boss()` (`boss.rs`) emit one `SQ_ENCOUNTER kind=… enemy=<hex> … outcome=… dmg_dealt=… dmg_taken=… xp=… gold=…` line to **stderr** per resolved fight **only when `SQ_DEBUG` is set** (the balance sim parses these). Default (unset) behavior is unchanged — no extra output. Boss per-fight damage totals accumulate in `Boss.dmg_dealt_total`/`dmg_taken_total` (serde-default) since a boss fight spans many ticks. The shared emit helper + hex name encoder live in `src/telemetry.rs`.
- **All game output is `eprintln!` (stderr)** so piped stdout stays clean.
- `sq tournament` is **deprecated** — a real subcommand that forwards to `arena` with a warning, not a clap alias.

## Dependencies

### Internal
- All modules depend on `character.rs` types (`Character`, `Item`, `Rarity`, `Class`, etc.)
- `events.rs` depends on `display`, `journal`, `loot`, `state`, `zones`, `boss`, `messages`
- `arena.rs` depends on `character`, `messages`, `journal`, `display`, `loot` (owns its own combat math)
- `display.rs` depends on `character` and `journal` types, plus `zones::Zone`/`ZoneColor`
- `main.rs` depends on all modules

### External
- `clap` — CLI parsing (only in `main.rs`)
- `colored` — `main.rs`, `display.rs`, `events.rs`, `arena.rs`, `messages.rs`
- `rand` — `events.rs`, `arena.rs`, `boss.rs`, `loot.rs`, `zones.rs`, `sage.rs`
- `serde` / `serde_json` — `character.rs`, `journal.rs`, `state.rs`
- `chrono` — `journal.rs`, `state.rs`, `main.rs`, `sage.rs`
- `dirs` — `state.rs`, `zones.rs`
- `ureq` — `sage.rs` (crates.io version check)
- `strsim` — `help.rs` only (Levenshtein topic suggestions for `sq help`); item-name lookup in `main.rs` is substring/token matching

<!-- MANUAL: Any manually added notes below this line are preserved on regeneration -->

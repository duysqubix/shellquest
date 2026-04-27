<!-- Generated: 2026-04-09 | Updated: 2026-04-09 -->

# shellquest

## Purpose
A passive RPG that lives in your terminal. Every shell command you run triggers game events — combat encounters, loot drops, zone travel, XP gains, and more. Installed as the `sq` CLI binary, it hooks into your shell's prompt to intercept commands via `sq tick` and progresses your character automatically. Published to crates.io, GitHub releases, and Docker Hub.

## Key Files

| File | Description |
|------|-------------|
| `Cargo.toml` | Package manifest — binary is `sq`, deps: clap, colored, dirs, rand, serde, serde_json, chrono |
| `Dockerfile` | Multi-stage build: rust builder + debian-slim runtime with tini entrypoint |
| `install.sh` | Curl-pipe installer: clones repo, `cargo install`, auto-installs shell hook |
| `publish.sh` | Release script: version bump, git push, `gh release`, `cargo publish` |
| `README.md` | User-facing documentation |
| `LICENSE` | MIT license |

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| `src/` | All Rust source code (see `src/AGENTS.md`) |

## For AI Agents

### Working In This Directory
- Binary name is `sq` (not `shellquest`) — defined in `[[bin]]` in Cargo.toml
- Save data lives at `~/.shellquest/save.json` (atomic write via temp file + rename)
- Shell hook uses `precmd`/`PROMPT_COMMAND`/`fish_postexec` to call `sq tick` synchronously after every command
- All game output goes to **stderr** (`eprintln!`) so it doesn't interfere with piped stdout
- The `tick` subcommand must remain fast and silent on error (no character = silent return)

### Testing Requirements
- `cargo build` to verify compilation (`cargo clippy` is not installed in the current toolchain — skip it)
- `cargo test` to run the unit test suite. Manual testing is also required for CLI flow:
  - `sq init` → create character
  - `sq status` → view sheet
  - `sq tick --cmd "git commit" --cwd "." --exit-code 0` → trigger craft event
  - `sq tick --cmd "bad" --cwd "." --exit-code 1` → trigger trap (25% chance — run several times)
  - `cd ~ && sq shop` → shop only works from home directory; shows numbered item list
  - `cd ~ && sq buy 1` → buy item by **number** (1-indexed), not by name
  - Force boss spawn for testing: temporarily set `gen_ratio(1, 1)` in `maybe_spawn()` in `src/boss.rs`, run `sq tick --cmd "ls" --cwd "." --exit-code 0`, then revert
  - Boss state lives at `active_boss` in the save file — can be cleared manually via JSON edit of `~/.shellquest/save.json`
  - Test permadeath mode: set `"permadeath": true` in save.json, set `"hp": 1`, run `sq tick --cmd "bad" --cwd "." --exit-code 1` — eulogy should print, save file should be deleted
  - Test class messages: run `sq tick --cmd "git commit" --cwd "." --exit-code 0` then `sq journal` — message should reflect your class flavor (Wizard: grimoire/arcane, Warrior: battle-scroll, etc.)
  - Test zone XP scaling: run ticks from `$HOME` (danger 1) vs `/tmp` (danger 3) — XP in journal should be ~1.5× higher in /tmp
  - Test sage update notification: set `"last_announced_version": null` in save.json and `"latest_version": "99.0.0"` — sage should appear on next tick guaranteed (without the 1/50 random gate)
  - **Arena QA**:
    - `sq arena` (interactive) — verify tier selection, combat loop, and cash-out.
    - `echo "y" | sq arena` — verify rejection of non-interactive input (should fail if not a TTY).
    - `sq arena` -> select tier -> cash out at Round 1 — verify gold/XP gain and journal entry.
    - `sq arena` -> get KO'd — verify loss of entry fee, HP set to 25%, and "Knocked out" journal label.
    - Chest overflow: fill inventory (20 items), win arena with loot — verify rejected items convert to half-sell-value gold.
    - Interruption: `Ctrl+C` during a run — verify no state is saved (rollback behavior).

### Common Patterns
- Serde for all data structures (JSON serialization)
- `colored` crate for terminal output with rarity-tiered styling
- `rand::Rng` with `gen_ratio()` for probability-based event triggers
- Two-pass message formatting: plain text for journal storage, colored for terminal display
- Auto-equip logic: new item replaces equipped if higher power, otherwise goes to inventory (capped at 20)
- **Arena Transactions**: Arena results are committed atomically at the end of a session. Runs are not resumable. Hard interruptions result in a rollback to the pre-arena state (including the entry fee).

## Dependencies

### External
- `clap` 4.x — CLI argument parsing with derive macros
- `colored` 2.x — Terminal color/style output
- `dirs` 5.x — Cross-platform home directory resolution
- `rand` 0.8.x — RNG for combat, loot, and event probabilities
- `serde` / `serde_json` 1.x — Save file serialization
- `chrono` 0.4.x — Timestamps for journal entries and last tick tracking

<!-- MANUAL: Any manually added notes below this line are preserved on regeneration -->

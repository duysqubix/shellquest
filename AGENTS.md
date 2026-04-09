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
- `cargo build` to verify compilation
- `cargo clippy` for lint checks
- No test suite exists yet — manual testing via `sq init`, `sq status`, `sq tick --cmd "test" --cwd "." --exit-code 0`

### Common Patterns
- Serde for all data structures (JSON serialization)
- `colored` crate for terminal output with rarity-tiered styling
- `rand::Rng` with `gen_ratio()` for probability-based event triggers
- Two-pass message formatting: plain text for journal storage, colored for terminal display
- Auto-equip logic: new item replaces equipped if higher power, otherwise goes to inventory (capped at 20)

## Dependencies

### External
- `clap` 4.x — CLI argument parsing with derive macros
- `colored` 2.x — Terminal color/style output
- `dirs` 5.x — Cross-platform home directory resolution
- `rand` 0.8.x — RNG for combat, loot, and event probabilities
- `serde` / `serde_json` 1.x — Save file serialization
- `chrono` 0.4.x — Timestamps for journal entries and last tick tracking

<!-- MANUAL: Any manually added notes below this line are preserved on regeneration -->

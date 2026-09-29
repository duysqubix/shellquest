<!-- Generated: 2026-04-09 | Updated: 2026-09-29 -->

# shellquest

## Purpose
A passive RPG that lives in your terminal. Every shell command you run triggers game events — combat encounters, loot drops, zone travel, XP gains, and more. Installed as the `sq` CLI binary, it hooks into your shell's prompt to intercept commands via `sq tick` and progresses your character automatically. Features 34 zones, a daily Void quest (portal opens at `$HOME`, maze reshuffles at UTC midnight), and a 5-tier arena gauntlet. Published to crates.io and GitHub releases (the `Dockerfile` builds a local play/sim image; nothing pushes it to Docker Hub).

## Key Files

| File | Description |
|------|-------------|
| `Cargo.toml` | Package manifest — binary is `sq`, deps: clap, colored, dirs, rand, serde, serde_json, chrono, ureq, strsim |
| `Dockerfile` | Multi-stage build: rust builder + debian-slim runtime with tini entrypoint |
| `install.sh` | Curl-pipe installer: clones repo, `cargo install`, auto-installs shell hook |
| `publish.sh` | Release script: version bump, commit, push, `gh release` (notes from `release-notes/vX.Y.Z.md`), `cargo publish` |
| `justfile` | Task runner: `just check` (all gates), `build`/`test`/`fmt`/`lint`, `sandbox`, `ship`, and the `sim-*` recipes that drive the balance simulator |
| `dev-tools/sq-sandbox` | Runs the dev `sq` against a throwaway HOME for manual QA (stdlib Python); see Testing Requirements |
| `CLAUDE.md`, `.claude/` | Claude Code setup: CLAUDE.md imports this file. `.claude/` holds permission rules, the live-save guard hook (plus its tests), and the skills `sq-qa`, `balance-check`, and `release` |
| `README.md` | User-facing documentation |
| `LICENSE` | MIT license |

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| `src/` | All Rust source code (see `src/AGENTS.md`) |
| `dev-tools/balance-sim/` | Dev-only Python balance simulator (see its `AGENTS.md`) — NOT shipped in the binary |
| `release-notes/` | One `vX.Y.Z.md` per release; canonical source for GitHub release notes (see `release-notes/README.md`) |
| `docs/` | Balance analysis + research notes (`balance-analysis.md`, `docs/research/`) — design rationale, not code |

## For AI Agents

### Working In This Directory
- Binary name is `sq` (not `shellquest`) — defined in `[[bin]]` in Cargo.toml
- Save data lives at `~/.shellquest/save.json`. Writes go to a per-process temp file that is renamed into place, while an exclusive lock on `save.lock` is held. The previous save is kept as `save.json.bak` (hard link), and a save that won't parse is restored from it automatically.
- Shell hook uses `precmd`/`PROMPT_COMMAND`/`fish_postexec` to call `sq tick` synchronously after every command
- All game output goes to **stderr** (`eprintln!`) so it doesn't interfere with piped stdout
- The `tick` subcommand must remain fast and silent on error (no character = silent return) — **unless `SQ_DEBUG` is set** (dev/sim diagnostics), in which case tick logs load/save failures with context and exits non-zero. Default behavior (SQ_DEBUG unset) is unchanged.
- **Read-only catalog commands**: `sq items --json` and `sq bestiary --json` dump the static loot tables / boss roster + monster bestiary as JSON (no save access, any cwd, exit 0). The balance-sim dashboard consumes them for its Items/Bestiary tabs. Source of truth = `loot.rs` / `boss.rs` / `events.rs` const data.
- **Combat telemetry**: under `SQ_DEBUG`, `combat()` and `tick_boss()` emit one `SQ_ENCOUNTER` line per resolved fight to stderr (`src/telemetry.rs` owns the helper). The sim parses these. Silent when `SQ_DEBUG` unset.

### Testing Requirements
- **Gates:** `just check` runs everything: `cargo fmt --check`, `cargo clippy --all-targets`, `cargo test`, and the guard-hook tests.
  - Unit tests are in-file `#[cfg(test)]` modules. Integration tests live in `tests/` and drive the real binary (`CARGO_BIN_EXE_sq`) with a temporary HOME. None of them touch the real `$HOME`.
  - Clippy is installed. Its pre-existing warnings are reported but not yet fatal; don't add new ones.
- **Manual QA never uses the real HOME.** The maintainer plays this game: `~/.shellquest/save.json` is a live character, and their shell hook ticks it before every prompt.
  - Never run `sq`, `target/*/sq`, or `cargo run` against the real HOME, and never edit `~/.shellquest` by hand.
  - Use `dev-tools/sq-sandbox`. It runs the dev build with `HOME=<repo>/.sq-sandbox/home` and `SQ_NO_PACING=1`, and provides `new` (non-interactive character creation), `set` / `show` (edit and inspect the sandbox save, including `@now-1d` timestamps), `tick "<cmd>" -n N`, `boss`, and `tty` (drives the arena through a pseudo-terminal).
  - Scenario recipes live in `.claude/skills/sq-qa/SKILL.md`, plain markdown that any agent can follow. They cover class flavor, traps, zone XP, shop, enchant, identify, junk sweep, bosses, permadeath, sage, the Void quest, and every arena path (cash-out, KO, Ctrl-C rollback, chest overflow, TTY refusal).
  - Claude Code enforces all this with `.claude/hooks/guard-live-save.py`.
- **Known cargo cache quirk:** `cargo build --bin sq` sometimes reports `Finished … (0 crates compiled)` while `target/debug/sq` is stale relative to source. It happens when the test profile was rebuilt but the prod binary's fingerprint went out of sync. If a fresh edit is missing from a manual QA run, run `cargo clean -p shellquest && cargo build --bin sq`. The rebuild takes about 3 s but discards roughly 250 MB of cache. Observed in the v1.18 and v1.20 QA cycles.

### Common Patterns
- Serde for all data structures (JSON serialization)
- `colored` crate for terminal output with rarity-tiered styling
- `rand::Rng` with `gen_ratio()` for probability-based event triggers
- Two-pass message formatting: plain text for journal storage, colored for terminal display
- Loot is never auto-equipped: drops go to the inventory via `add_to_inventory()` (cap 20). When full, it drops the weakest Common–Rare item by raw power (Epic/Legendary are never dropped). Players equip with `sq equip`/`sq wield`
- **Arena Transactions**: Arena results are committed atomically at the end of a session. Runs are not resumable. Hard interruptions result in a rollback to the pre-arena state (including the entry fee).

### Release Cadence (effective post-v1.22.0)
- **Batch related work into larger themed releases** — do not ship after every distinct feature. Accumulate features into release arcs.
- **Threshold for a release**: the release notes file would have 5+ paragraphs of player-visible material covering a coherent theme. Below that bar, commits stay on `master` awaiting their thematic partners.
- **Exception**: critical bug fixes always ship as immediate patch releases.
- **Anti-pattern**: 7 releases in one session (as happened during the v1.18→v1.22 balance overhaul). That cadence felt fragmented; future overhauls should ship as 1-2 atomic releases per arc, not 5-7.
- **Conventional commits remain unchanged** — every commit is still atomic and properly typed (`feat:`, `fix:`, etc). The change is purely about *when* to invoke `publish.sh`, not how to structure individual commits on master.

### Release Notes Workflow
- Write `release-notes/vX.Y.Z.md` **before** running `./publish.sh`. The script auto-detects it via the convention path and passes it to `gh release create --notes-file`; missing file → falls back to `--generate-notes` and warns.
- `release-notes/` is the canonical source. Re-sync GitHub at any time: `gh release edit vX.Y.Z --notes-file release-notes/vX.Y.Z.md`. See `release-notes/README.md` for the voice rules.

### Balance Tuning
- Gameplay numbers are validated empirically by the simulator in `dev-tools/balance-sim/` (Python, dev-only). Each simulated character runs in its own Docker container (`just sim-*` recipes; one container per character for filesystem isolation). Use `just sim-*` to run sweeps before/after a balance change; never tune by feel alone. The baseline-vs-change workflow and the `(sim-validated)` commit convention are written up in `.claude/skills/balance-check/SKILL.md`. **Caveat (2026-09-29):** the simulator has known fidelity defects (beads epic `shellqeuest-dpl`): arena results are misparsed, there is no simulated clock, and it runs almost no overworld fights. Until that epic closes, treat arena, boss and death-rate numbers as unvalidated.

## Dependencies

### External
- `clap` 4.x — CLI argument parsing with derive macros
- `colored` 2.x — Terminal color/style output
- `dirs` 5.x — Cross-platform home directory resolution
- `rand` 0.8.x — RNG for combat, loot, and event probabilities
- `serde` / `serde_json` 1.x — Save file serialization
- `chrono` 0.4.x — Timestamps for journal entries and last tick tracking
- `ureq` 2.x — Blocking HTTP client for the sage's crates.io version check
- `strsim` 0.11.x — Levenshtein suggestions for mistyped `sq help <topic>` names (help.rs only; item lookup is substring/token matching in main.rs)

<!-- MANUAL: Any manually added notes below this line are preserved on regeneration -->

<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:7510c1e2 -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

**Architecture in one line:** issues live in a local Dolt DB; sync uses `refs/dolt/data` on your git remote; `.beads/issues.jsonl` is a passive export. See https://github.com/gastownhall/beads/blob/main/docs/SYNC_CONCEPTS.md for details and anti-patterns.

## Session Completion

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
<!-- END BEADS INTEGRATION -->

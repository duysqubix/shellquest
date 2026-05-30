<!-- Generated: 2026-04-09 | Updated: 2026-05-30 -->

# shellquest

## Purpose
A passive RPG that lives in your terminal. Every shell command you run triggers game events — combat encounters, loot drops, zone travel, XP gains, and more. Installed as the `sq` CLI binary, it hooks into your shell's prompt to intercept commands via `sq tick` and progresses your character automatically. Features 34 zones, a daily Void quest (portal opens at `$HOME`, maze reshuffles at UTC midnight), and a 5-tier arena gauntlet. Published to crates.io, GitHub releases, and Docker Hub.

## Key Files

| File | Description |
|------|-------------|
| `Cargo.toml` | Package manifest — binary is `sq`, deps: clap, colored, dirs, rand, serde, serde_json, chrono, ureq, strsim |
| `Dockerfile` | Multi-stage build: rust builder + debian-slim runtime with tini entrypoint |
| `install.sh` | Curl-pipe installer: clones repo, `cargo install`, auto-installs shell hook |
| `publish.sh` | Release script: version bump, commit, push, `gh release` (notes from `release-notes/vX.Y.Z.md`), `cargo publish` |
| `justfile` | Task runner — `just build`/`test`/`ship` + `just sim-*` recipes that drive the balance simulator |
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
- Save data lives at `~/.shellquest/save.json` (atomic write via temp file + rename)
- Shell hook uses `precmd`/`PROMPT_COMMAND`/`fish_postexec` to call `sq tick` synchronously after every command
- All game output goes to **stderr** (`eprintln!`) so it doesn't interfere with piped stdout
- The `tick` subcommand must remain fast and silent on error (no character = silent return) — **unless `SQ_DEBUG` is set** (dev/sim diagnostics), in which case tick logs load/save failures with context and exits non-zero. Default behavior (SQ_DEBUG unset) is unchanged.
- **Read-only catalog commands**: `sq items --json` and `sq bestiary --json` dump the static loot tables / boss roster + monster bestiary as JSON (no save access, any cwd, exit 0). The balance-sim dashboard consumes them for its Items/Bestiary tabs. Source of truth = `loot.rs` / `boss.rs` / `events.rs` const data.
- **Combat telemetry**: under `SQ_DEBUG`, `combat()` and `tick_boss()` emit one `SQ_ENCOUNTER` line per resolved fight to stderr (`src/telemetry.rs` owns the helper). The sim parses these. Silent when `SQ_DEBUG` unset.

### Testing Requirements
- `cargo build` to verify compilation (`cargo clippy` is not installed in the current toolchain — skip it)
- **Known cargo cache quirk**: `cargo build --bin sq` sometimes reports `Finished … (0 crates compiled)` while the on-disk `target/debug/sq` is stale relative to source — this happens when the test profile was rebuilt but the prod binary fingerprint went out of sync. If a freshly-edited change is missing from manual QA on `target/debug/sq`, run `cargo clean -p shellquest && cargo build --bin sq` to force a full prod rebuild (~3s, eats ~250MB of cache). Observed in v1.18 and v1.20 manual QA cycles.
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
  - Test enchant: `cd ~ && sq enchant <equipped item>` — verify +1 power, gold deducted, `[Enchanted +N]` tag (max +5). Wizards can enchant from any directory; other classes only from `$HOME`.
  - Test identify: `sq id <name>` from any directory — read-only card, no save write, no tick consumed.
  - Test junk sweep: `cd ~ && sq sell junk` — sells all Common + Uncommon, never Rare and up.
  - **Void quest QA**:
    - `cd ~ && sq quest` — should reveal the quest and open the portal (creates `~/.shellquest/the_void/`).
    - Navigate into the maze, find `lost_scroll_NNNN.txt`, read the phrase.
    - `cd ~ && sq quest answer <phrase>` — should claim reward (Rare+ loot, XP, gold) and remove the scroll.
    - `cd ~ && sq quest answer wrongphrase` — should reject with a hint.
    - Run `sq quest` again same day — should show quest already completed.
    - Set `quest_refreshed` to yesterday in save.json, run `sq quest` — should reshuffle the maze.
  - **Arena QA** (combat is paced ~1.5s/line + wave-escalating since v1.24):
    - `sq arena` (interactive) — verify tier selection, paced combat loop, banked-gold preview, and cash-out.
    - `SQ_NO_PACING=1 sq arena` — disables the 1.5s pacing for fast manual QA (combat resolves instantly).
    - `echo "y" | sq arena` — verify rejection of non-interactive input (should fail if not a TTY).
    - `sq arena` -> select tier -> cash out at Round 1 — verify gold/XP gain and journal entry.
    - `sq arena` -> get KO'd — verify loss of entry fee, HP set to 25% of max HP at entry, "Knocked out" CLI summary, and journal entry "Arena KO in {tier} after N rounds. Fee: N gold."
    - Chest overflow: fill inventory (20 items), win arena with loot — verify rejected items convert to half-sell-value gold.
    - Interruption: `Ctrl+C` during a run — verify no state is saved (rollback behavior).

### Common Patterns
- Serde for all data structures (JSON serialization)
- `colored` crate for terminal output with rarity-tiered styling
- `rand::Rng` with `gen_ratio()` for probability-based event triggers
- Two-pass message formatting: plain text for journal storage, colored for terminal display
- Auto-equip logic: new item replaces equipped if higher power, otherwise goes to inventory (capped at 20)
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
- Gameplay numbers are validated empirically by the simulator in `dev-tools/balance-sim/` (Python, dev-only). Each simulated character runs in its own Docker container (`just sim-*` recipes; one container per character for filesystem isolation). Use `just sim-*` to run sweeps before/after a balance change; never tune by feel alone.

## Dependencies

### External
- `clap` 4.x — CLI argument parsing with derive macros
- `colored` 2.x — Terminal color/style output
- `dirs` 5.x — Cross-platform home directory resolution
- `rand` 0.8.x — RNG for combat, loot, and event probabilities
- `serde` / `serde_json` 1.x — Save file serialization
- `chrono` 0.4.x — Timestamps for journal entries and last tick tracking
- `ureq` 2.x — Blocking HTTP client for the sage's crates.io version check
- `strsim` 0.11.x — Fuzzy string matching for item-name lookup (`equip`/`sell`/`identify`)

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

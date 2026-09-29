# CLAUDE.md

`sq` (shellquest) is a passive RPG that lives in the terminal. It is a Rust CLI, and its shell hook runs `sq tick` after every command the player types. It ships as one binary built from flat modules in `src/`, with in-file unit tests. A dev-only Python balance simulator lives in `dev-tools/balance-sim/`.

Knowledge shared by every coding agent lives in AGENTS.md, imported below. `src/CLAUDE.md` and `dev-tools/balance-sim/CLAUDE.md` import their directory's AGENTS.md, so those notes load when you work there.

@AGENTS.md

## Guardrails

- **The developer's live character is off-limits.** `~/.shellquest/save.json` is a real save, and their shell hook ticks it before every prompt.
  - Run every manual `sq` through `dev-tools/sq-sandbox`; the `sq-qa` skill has the workflow and scenario recipes.
  - `.claude/hooks/guard-live-save.py` denies unsandboxed `sq` and `cargo run`, and asks before commands that name `~/.shellquest`.
  - `cargo test` is safe: the tests never touch `$HOME`.
- **Leave the installed binary alone.** `cargo install` and `sq update` replace `~/.cargo/bin/sq`, which the hook runs before every prompt. A broken build breaks the player's shell, so ask first.
- **Publishing is irreversible.** Run `./publish.sh`, `just ship`, `cargo publish`, or `gh release` only when the user asks (the `release` skill). crates.io versions can't be deleted.
- **Balance numbers are sim-validated, never tuned by feel.** Use the `balance-check` skill and commit as `fix(balance): … (sim-validated)` with before/after numbers.
- **Old saves must keep loading.** Every new persisted field gets `#[serde(default)]`. Never rename or remove a persisted field without a migration.

## Commands

| Task | Command |
|---|---|
| Build | `cargo build` (about 3 s incremental) |
| Test | `cargo test` (in-file unit tests, about 2 s) |
| All gates before a commit | `just check`: rustfmt check, clippy, Rust tests, guard-hook tests |
| Format / lint | `just fmt` / `just lint`. Clippy warnings are reported but not yet fatal; don't add new ones |
| Play safely | `dev-tools/sq-sandbox new`, then `dev-tools/sq-sandbox status`, `… tick "git commit" -n 20`, `… tty -i 1 -i y -- arena` |
| Balance sims (Docker) | `just sim-quick`, `just sim-pit <label>`, `just report <label>`; see `just --list` |

## Working here

- **Tasks:** track work in beads; see the Beads section below and in AGENTS.md. Don't use TodoWrite or markdown TODO lists.
- **Commits:** conventional and atomic, straight to `master`, e.g. `feat(arena): …`, `fix(balance): …`, `docs(release-notes): …`, `chore(beads): …`.
- **Releases:** batched into themed arcs (AGENTS.md → Release Cadence) and shipped with the `release` skill.
- **Code invariants** (stderr output, plain/colored message pairs, a fast silent `tick`, arena commits): see AGENTS.md and src/AGENTS.md.
- **Known problems:** the 2026-09-29 sweep filed its findings as beads epics (`bd list --label sweep`). Check the relevant epic before changing an area. The balance simulator is unreliable until `shellqeuest-dpl` closes.

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

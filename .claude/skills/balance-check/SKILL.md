---
name: balance-check
description: Validate a gameplay-number change (XP curve, combat math, loot odds, arena tuning, boss stats, economy) with the Docker balance simulator, comparing baseline against change, before it is committed as "sim-validated". Use when tuning constants, when investigating "X is too strong/weak/fast/slow", or when a diff touches balance constants in character.rs, events.rs, arena.rs, boss.rs, or loot.rs.
---

# Sim-validate a balance change

**Project rule:** gameplay numbers are tuned empirically, never by feel (AGENTS.md → Balance Tuning).

The simulator in `dev-tools/balance-sim/` drives the real `sq` binary through compressed lifetimes. It runs one Docker container per character, and every metric lands in `dev-tools/balance-sim/runs.db`. Internals are in `dev-tools/balance-sim/AGENTS.md`.

## Known simulator defects: read this first

Until beads epic `shellqeuest-dpl` is closed, the simulator misreports important things (found in the 2026-09-29 sweep):

- **Arena results are misparsed** (`shellqeuest-dpl.1`). Every full clear is recorded as a cash-out, one round short, and crits are dropped for 4 of the 5 classes.
- **Play is unrealistic** (`shellqeuest-dpl.2`).
  - There is no simulated clock, so passive healing and the boss flee never happen.
  - Almost no overworld fights happen, because only `pwd` reaches the encounter branch.
  - The command mix is craft-heavy.
  - Recorded attack uses DEX/2, while the game uses DEX/3.
- **Slow runs are dropped silently** (`shellqeuest-dpl.3`). Runs that hit the container time budget are killed and never finalized, which biases averages toward the runs that finished.
- **An enchant lockout can stop all arena entry mid-run** (`shellqeuest-dpl.4`).
- **Potions are destroyed** by a fake "equip" (`shellqeuest-dpl.5`).

What this means for you:

- Don't treat arena, boss, or death-rate numbers as evidence yet.
- Prefer overworld XP and level-pace metrics, and check mechanics by hand in the `sq-qa` sandbox.
- In the commit body, say which defects applied.
- Commits `40f8ad6`, `50df1f7`, `c9d8251` and `90ebf4c` were "sim-validated" under these defects.

## Before you start

- Docker must be running (`docker info`).
- Run `just sim-image` once. It builds the Python image; the sim code is mounted, not baked in.
- Every `sim-*` recipe first runs `just sim-sq-linux`, which builds a Linux `sq` from the **working tree**. The binary under test is whatever is checked out, so uncommitted changes count.

## Loop

1. **State the hypothesis and the metric that settles it.** Example: "Levels 1–20 go too fast. Median ticks to L20 should rise 30–50% while the death rate stays under 8%."
2. **Pick the narrowest recipe that exercises it** (see `just --list`).

   | Recipe | Covers |
   |---|---|
   | `sim-quick` | Smoke test |
   | `sim-pit` | Tier 1 arena |
   | `sim-gauntlet`, `sim-colosseum`, `sim-abyssal` | Tiers 2–4 |
   | `sim-endgame` | Level 150 (slow: runs longer than the ~32-minute container budget are cut off; see Known defects) |
   | `sim-full` | Lifetime to L60 across classes, races, and strategies |
   | `sim-custom`, `sim-custom-seeded`, `sim-custom-seeded-prestige` | Start level and prestige seeding for deep tiers |

3. **Run the baseline on the unmodified tree:**
   `just sim-pit baseline-<topic> 3 4`
   The arguments are the label, the number of runs, and the parallelism. Labels are positional; `label=x` would be passed literally.
4. **Apply the change.** Update each constant together with its guard test in the module's `mod tests`, then run `cargo test`.
5. **Rerun the same recipe with the same arguments:** `just sim-pit change-<topic> 3 4`.
6. **Compare.**
   - Run `just report baseline-<topic>` and `just report change-<topic>`. Always pass the label: with uv installed, bare `just report` fails and says "no runs in db". Markdown reports land in `dev-tools/balance-sim/reports/`.
   - Run `just dashboard change-<topic>` for the A/B dashboard.
   - Check the target metric and the neighbours: other classes, other tiers, death rates, gold flow.
7. **Iterate** until the metric lands without regressions elsewhere.
8. **Commit** as `fix(balance): <what> (sim-validated)`.
   - Put the before and after numbers, with the run labels, in the commit body.
   - Record the same in the beads issue with `bd update <id> --append-notes "…"` (`--notes` replaces existing notes).

## Rules

- **Compare like with like.** Use the same recipe, runs, strategies, and target level for both sides. Noisy metrics, especially arena outcomes, need more runs.
- **Keep `runs.db` for the whole comparison.** `just clean-sims` wipes it and every label in it.
- **Update the driver before trusting runs after an arena text change.** If you edit arena menus or prompts in `src/arena.rs`, update the PTY driver in `dev-tools/balance-sim/simulator/driver.py` first. It pattern-matches prompts, so a stale driver quietly misplays the arena.
- **Run long sweeps in the background.** Follow progress with `just watch`. Bosses spawn at 1/500 ticks, so short sweeps may record no boss fights at all.
- **Sandbox ticks are not balance evidence.** Use the `sq-qa` sandbox to check that a mechanic works, and the simulator to check that its numbers are right.

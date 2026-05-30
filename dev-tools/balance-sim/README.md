# balance-sim — shellquest balance simulator

A "Black Mirror Hang the DJ" style simulator that runs thousands of compressed
shellquest playthroughs in parallel — **each character in its own Docker container**
for true filesystem isolation — and persists every interesting metric to
SQLite. Use it to validate balance changes before shipping them.

**Not part of shellquest core. Dev-only tool.**

## What it does

`runner.py` runs on the host and launches **one Docker container per simulated
character** (`docker run --rm --network=none`, image `shellquest-sim`). The `sq`
binary mounts read-only at `/opt/sq`, the sim code at `/sim`, and a read-write
`/out` dir receives that character's SQLite shard. Host root is never mounted —
that container is the filesystem jail (`sq` is expected to gain real-filesystem
reach, so a tempdir `$HOME` is no longer enough isolation).

Inside each container, `container_main.py` drives one AI lifetime:

1. Initializes a fresh character (class × race × strategy)
2. Drives the AI player through their entire lifetime:
   - Tick the game (vary `cwd` to hit different danger zones, vary `cmd` to
     trigger crafts / fights / traps)
   - Auto-equip dropped loot when it's an upgrade
   - Visit the shop periodically, buy upgrades, enchant
   - Run the highest unlocked arena tier when affordable
3. Snapshots full character state at intervals into the shard
4. Logs every action, every arena attempt, every item event, **every raw `sq`
   invocation** (argv/stdout/stderr/exit_code) for post-mortem diagnosis, and
   **every resolved overworld/boss fight** (`overworld_encounter`: enemy, dmg dealt/taken,
   outcome) parsed from the game's `SQ_ENCOUNTER` diagnostic lines

The host merges all shards into `runs.db`. Then the reporter queries it to produce comparative reports:

- Time-to-level distributions across class × strategy
- Arena win rate per tier per character build
- Boss encounters & outcomes (win/loss/flee), total mobs encountered, and damage dealt/taken split overworld-mob vs boss
- Gear progression heatmaps
- Gold accumulation curves
- "Which tuning_label gives the best player retention?"

## Usage

```bash
# Smoke test: 1 Warrior to L20, in a container (builds image + linux sq first)
# No label = auto-labeled 'sim-quick-<timestamp>'; runs accumulate separately in runs.db.
just sim-quick

# Name/group a run by passing the label POSITIONALLY (not label=NAME):
just sim-quick my-baseline

# Full sweep: many lives per (class × race × strategy) combo, one container each
just sim-full v1.24-soft 5

# Generate markdown report (no arg = most recent label; or pass one positionally)
just report
just report v1.24-soft

# Generate interactive HTML dashboard (host-native; open file:// in browser)
# Lists every label in runs.db in the primary/compare dropdowns.
just dashboard
```

Every `sim-*` recipe is **Docker-only** and depends on `just sim-image` (the python
sim image) plus `just sim-sq-linux` (extracts a Linux `sq` — a macOS `cargo build`
produces a Mach-O binary that can't run in the Linux container). `report` /
`dashboard` / `watch` stay host-native (read-only on `runs.db`).

## Dashboard

`python3 dashboard.py` generates a single self-contained `dashboard.html`
file next to `runs.db`. Open it locally in any browser (`file:///.../dashboard.html`).

The dashboard pulls every `tuning_label` from the database and gives you
seven tabs:

- **Summary** — overview tiles + lifetime stats table (class × strategy)
- **Progression** — line charts of level / attack / defense / gold over tick number
- **Arena** — survival rate, average rounds reached, damage taken, enemy crit
  rate, all bucketed by tier × class
- **Combat** — boss win/loss/flee rate, mob outcomes, and damage dealt/taken
  split overworld-mob vs boss (from the `overworld_encounter` telemetry)
- **Items** — static item-catalog distributions (rarity, power, slot, price)
  sourced from `sq items --json`; renders even with no sim data
- **Bestiary** — static monster bestiary (by tier) + boss roster, tier→danger
  gating and elite modifiers, sourced from `sq bestiary --json`
- **A/B Compare** — pick a second `tuning_label` in the header dropdown to
  diff two tuning configs side-by-side (the "did v1.25 actually improve
  over v1.24?" view)

Charts use Chart.js loaded from CDN — no other JS deps. Refresh the dashboard
after new sims by re-running `just dashboard` (the Items/Bestiary tabs need a
built `sq` binary; `cargo build --bin sq` first).

## Layout

```
dev-tools/balance-sim/
├── README.md                  this file
├── schema.sql                 SQLite DDL
├── simulator/
│   ├── db.py                  SQLite connection + insert/query/merge
│   ├── driver.py              sq CLI command wrappers + invocation capture
│   ├── container_main.py      in-container entrypoint (one lifetime → shard)
│   ├── strategies.py          decision strategies (greedy/balanced/conservative)
│   └── player.py              SimPlayer lifecycle class
├── runner.py                  host orchestrator: docker run per character (CLI)
├── report.py                  markdown report generator (CLI)
├── Dockerfile                 the shellquest-sim image (python-slim + zone dirs)
├── runs.db                    SQLite output (gitignored)
├── .sq-linux/                 extracted Linux sq binary (gitignored)
└── reports/                   generated reports (gitignored)
```

## Dependencies

Python 3.10+, stdlib only. No external packages required. Containers are launched
via stdlib `subprocess` + `docker run` (deliberately not docker-py). Requires Docker.

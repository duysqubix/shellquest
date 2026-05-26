# balance-sim — shellquest balance simulator

A "Black Mirror Hang the DJ" style simulator that runs thousands of compressed
shellquest playthroughs in parallel and persists every interesting metric to
SQLite. Use it to validate balance changes before shipping them.

**Not part of shellquest core. Dev-only tool.**

## What it does

For each simulated character, the harness:

1. Creates an isolated `HOME=/tmp/sq-bench-{run_id}/`
2. Initializes a fresh character (class × race × strategy)
3. Drives an AI player through their entire lifetime:
   - Tick the game (vary `cwd` to hit different danger zones, vary `cmd` to
     trigger crafts / fights / traps)
   - Auto-equip dropped loot when it's an upgrade
   - Visit the shop periodically, buy upgrades, enchant
   - Run the highest unlocked arena tier when affordable
4. Snapshots full character state at intervals into SQLite
5. Logs every action, every arena attempt, every item event

Then the reporter queries SQLite to produce comparative reports:

- Time-to-level distributions across class × strategy
- Arena win rate per tier per character build
- Gear progression heatmaps
- Gold accumulation curves
- "Which tuning_label gives the best player retention?"

## Usage

```bash
# Run a single character lifetime (smoke test)
python3 runner.py --runs 1 --class Warrior --strategy greedy

# Full Black Mirror sweep: 100 lives per (class × strategy) combo
python3 runner.py --runs 100 --tuning-label v1.24-soft

# Generate report from the database
python3 report.py --tuning-label v1.24-soft --output reports/v1.24-soft.md
```

## Layout

```
dev-tools/balance-sim/
├── README.md                  this file
├── schema.sql                 SQLite DDL
├── simulator/
│   ├── db.py                  SQLite connection + insert/query
│   ├── driver.py              sq CLI command wrappers (tick, shop, arena)
│   ├── strategies.py          decision strategies (greedy/balanced/conservative)
│   └── player.py              SimPlayer lifecycle class
├── runner.py                  parallel run orchestrator (CLI)
├── report.py                  markdown report generator (CLI)
├── runs.db                    SQLite output (gitignored)
└── reports/                   generated reports (gitignored)
```

## Dependencies

Python 3.10+, stdlib only. No external packages required.

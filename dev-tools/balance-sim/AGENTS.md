<!-- Parent: ../../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-05-29 -->

# dev-tools/balance-sim

## Purpose
Dev-only "black-mirror" balance simulator. Runs many compressed shellquest lifetimes in parallel, drives the real `sq` binary as a subprocess (tick → equip → enchant → arena), and persists every metric to SQLite for tuning analysis. **Python 3.10+, stdlib only — no external deps.** Not shipped in the `sq` binary.

## Layout

```
balance-sim/
├── runner.py       # CLI orchestrator: builds class×race×strategy job matrix, multiprocessing.Pool.imap_unordered, merges per-worker DBs
├── report.py       # CLI → markdown report (lifetime summary, arena summary, time-to-level histograms)
├── dashboard.py    # CLI → single self-contained Chart.js dashboard.html
├── watch.py        # live `runs.db` tail/monitor (polling)
├── schema.sql      # SQLite schema + indexes (WAL, foreign keys)
└── simulator/
    ├── player.py     # one simulated lifetime: temp HOME, seed save.json, strategy loop, snapshots
    ├── driver.py     # `sq` wrapper: save.json read/write, command families, PTY-driven arena automation + output parsing
    ├── strategies.py # greedy / balanced / conservative decision policies
    └── db.py         # SQLite open/init/insert/merge (ATTACH + run-id remap)
```

## Run It (via root justfile)
```bash
just sim-quick label=smoke          # 1 Warrior to L20 (smoke test)
just sim-pit label=x runs=N parallel=4
just sim-gauntlet / sim-colosseum / sim-abyssal / sim-endgame
just sim-full label=x runs=N        # full class×race×strategy sweep to L60
just watch                          # live DB tail
just report label=x                 # → dev-tools/balance-sim/reports/x.md
just dashboard label=x              # → dashboard.html
just clean-sims                     # wipe runs.db / dashboards / worker DBs
```
The justfile auto-selects `uv run --script` when `uv` is installed (scripts are uv-runnable), else `python3`. Direct: `python3 runner.py …`, `report.py …`, `dashboard.py …`. Release recipe: `just ship version=patch`.

## Data Model (SQLite)
5 tables: `run` (one per lifetime: seed/class/race/strategy/tuning_label + final stats + end reason), `tick_snapshot` (per-tick char state), `action_log` (per-tick action + JSON details), `arena_attempt` (tier/rounds/outcome/damage/crits/swings), `item_event` (equip/enchant). WAL mode + FKs. Parallelism is **DB-per-worker, merged at the end** via `ATTACH` + run-id remap.

## Conventions & Gotchas
- **Hardcoded ABSOLUTE binary path**: `driver.py:13` → `SQ_BIN = "/home/duys/.repos/shellquest/target/debug/sq"`. Run `just build` first; **edit `SQ_BIN` if the repo ever moves** or sims silently fail to find `sq`.
- **Isolated HOME per run**: each lifetime sets `HOME=/tmp/sq-bench-{seed}-…` and reads/writes that copy of `.shellquest/save.json`. Never points at your real `~`.
- **`SQ_NO_PACING=1`** is set for all sims — arena's 1.5s pacing must be off or runs hang.
- **Arena automation needs a PTY** + prompt matching (it's not a plain `subprocess.run`). Touch `driver.py` arena parsing carefully.
- **No shared writable DB during a sweep** — don't expect to query `runs.db` mid-run except via `watch.py` (read-only tail).
- `runs.db`, `dashboard.html`, and `*-worker-*.db` are **disposable artifacts** — `just clean-sims` before a fresh sweep; keep them out of commits.
- Validate balance changes here **before** shipping a tuning release (see parent `AGENTS.md` → Balance Tuning).

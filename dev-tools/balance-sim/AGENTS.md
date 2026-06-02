<!-- Parent: ../../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-05-29 -->

# dev-tools/balance-sim

## Purpose
Dev-only "black-mirror" balance simulator. Runs many compressed shellquest lifetimes in parallel; **each simulated character runs inside its own Docker container** (image `shellquest-sim`) for true OS-level filesystem isolation. Inside each container the harness drives the real `sq` binary as a subprocess (tick → equip → enchant → arena) and persists every metric — plus every raw `sq` invocation — directly into the shared `runs.db` while the sweep is still running. **Python 3.10+, stdlib only — no external deps** (containers are launched via stdlib `subprocess` + `docker run`, deliberately not docker-py). Not shipped in the `sq` binary.

**Why containers?** `sq` is expected to gain real-filesystem reach (events that read/write/traverse arbitrary host paths). A tempdir `$HOME` no longer isolates that — a container is a true filesystem jail. The container **never mounts host root**; that is the whole point.

## Layout

```
balance-sim/
├── runner.py       # CLI orchestrator: builds class×race×strategy job matrix, initializes runs.db, multiprocessing.Pool.imap_unordered
├── report.py       # CLI → markdown report (lifetime, arena, boss/mob/damage combat summaries, time-to-level histograms)
├── dashboard.py    # CLI → single self-contained Chart.js dashboard.html
├── watch.py        # live `runs.db` tail/monitor (polling)
├── schema.sql      # SQLite schema + indexes (WAL, foreign keys)
└── simulator/
    ├── player.py        # one simulated lifetime: temp HOME, seed save.json, strategy loop, snapshots
    ├── driver.py        # `sq` wrapper: save.json read/write, command families, PTY-driven arena automation + output parsing; records every sq invocation
    ├── container_main.py # in-container entrypoint: runs one SimPlayer lifetime, writes shared /db/runs.db, prints one JSON result line to stdout
    ├── strategies.py    # greedy / balanced / conservative decision policies
    └── db.py            # SQLite open/init/insert helpers; merge_dbs remains for tests/legacy imports
```

## Container Model
`runner.py` stays on the **host** (keeps its `multiprocessing.Pool`). Each worker does **one `docker run --rm --network=none` per simulated character** instead of calling `simulate_one` in-process. Before the pool starts, the host opens `runs.db`, runs `db.init_schema()` once so containers do not race to create tables, and starts a tiny stdlib request/response-file writer in the DB directory. Mounts: the `sq` binary read-only at `/opt/sq`, the sim code read-only at `/sim`, and the directory containing `runs.db` read-write at `/db`. The container receives `/db/runs.db` as its DB path plus `SQ_SIM_DB_RPC_DIR=/db/runs.db.rpc`; `simulator.db.open_db()` therefore streams write requests through that mounted RPC directory to the host writer, which is the only SQLite writer. This preserves live shared `runs.db` visibility while avoiding Docker Desktop bind-mount SQLite corruption under parallel container writers. The container `$HOME` is disposable; **host root is never mounted**. `container_main.py` runs the full `SimPlayer` lifetime and prints exactly one JSON result line to stdout (`{"ok":true,run_id,ended_reason,final_state,ticks}` or `{"ok":false,error,...}`); the host parses it for progress only. There is no shard move and no end-of-sweep merge. runner.py honours env overrides `SQ_BIN_HOST` / `SIM_DIR_HOST` / `SIM_IMAGE`.

## Run It (via root justfile)
```bash
just sim-image                     # build the python sim image (shellquest-sim)
just sim-sq-linux                  # extract a LINUX sq binary for the containers (see gotcha below)
just sim-quick                     # 1 Warrior to L20 (smoke test) — auto-labels 'sim-quick-<timestamp>'
just sim-quick my-run              # … or pass a label POSITIONALLY to name/group the run
just sim-pit my-run 3 4            # positional args: label, runs, parallel
just sim-gauntlet / sim-colosseum / sim-abyssal / sim-endgame
just sim-full my-run 5             # full class×race×strategy sweep to L60
just watch                         # live DB tail (host-native, read-only)
just report                        # → report for the most recent label (host-native)
just report my-run                 # → dev-tools/balance-sim/reports/my-run.md
just dashboard                     # → dashboard.html, most recent label (host-native)
just dashboard my-run              # → dashboard focused on a specific label
just clean-sims                     # wipe runs.db / dashboards / worker DBs / .sq-linux
```
Every `sim-*` recipe is **Docker-only**: it declares `sim-image` + `sim-sq-linux` as prerequisites and runs one container per character. `report` / `dashboard` / `watch` stay host-native (they only read `runs.db`). Direct host invocation still works for the orchestrator: `python3 runner.py …`. Release recipe: `just ship version=patch`.

**Labels & persistence**: every `sim-*` run is APPENDED directly into the shared `runs.db` (live writes, never overwrite). With no positional label, each run auto-labels as `<recipe>-<YYYYMMDD-HHMMSS>` so repeated/different sims stay separate and individually selectable in the dashboard dropdown. Pass a positional label (`just sim-quick my-run` — NOT `label=my-run`, which `just` would pass literally) to name or intentionally group runs. `just clean-sims` is the only thing that wipes `runs.db`.

## Data Model (SQLite)
7 tables: `run` (one per lifetime: seed/class/race/strategy/tuning_label + final stats + end reason), `tick_snapshot` (per-tick char state), `action_log` (per-tick action + JSON details), `arena_attempt` (tier/rounds/outcome/damage/crits/swings), `item_event` (equip/enchant), `sq_invocation` (every `sq` subprocess call: argv, cwd, exit_code, stdout, stderr, duration_ms — for post-mortem diagnosis of binary bugs), `overworld_encounter` (one row per resolved overworld/boss fight: kind mob|boss, enemy_name, elite, dmg_dealt, dmg_taken, outcome kill|death|draw|win|loss|flee, xp/gold), and `final_item` (end-state gear/inventory). WAL mode + FKs + autocommit connections keep write-lock windows to one statement and make rows visible immediately. During Docker sweeps, parallel containers write **one shared DB live** through the host-owned `runs.db.rpc/` writer; SQLite serializes `INSERT INTO run`, so `AUTOINCREMENT` run ids are globally unique and child rows use the exact `run_id` returned inside each container. `db.merge_dbs` still exists for legacy/unit-test coverage, but `runner.py` no longer calls it.

## Conventions & Gotchas
- **`SQ_BIN` is env-overridable** (`driver.py`: `os.environ.get("SQ_BIN", "/opt/sq")`). In a container it resolves to the read-only mount at `/opt/sq`; no absolute host path is baked in. (The old hardcoded `/home/duys/...` path is gone.)
- **Linux binary required**: a macOS host `cargo build` produces a Mach-O `sq` that **cannot** run in the Linux sim container. `just sim-sq-linux` extracts a Linux ELF `sq` from the game `Dockerfile` to `.sq-linux/sq`; the `sim-*` recipes mount that via `SQ_BIN_HOST`.
- **Diagnostics capture**: each container sets `RUST_BACKTRACE=full` and `SQ_DEBUG=1`, and `driver.py` records every `sq` invocation (argv/cwd/stdout/stderr/exit_code/duration) into the `sq_invocation` table. When a bug surfaces in `sq` mid-sim, the evidence survives the disposable container in `runs.db`. (`SQ_DEBUG` makes `sq`'s `cmd_tick` log previously-swallowed load/save failures and exit non-zero — see `src/AGENTS.md`.)
- **Container FS isolation per character**: each lifetime runs in its own `docker run --rm --network=none` with a disposable `$HOME`; sim code is read-only at `/sim`, the `sq` binary is read-only at `/opt/sq`, and **host root is never mounted**. The intended minimal relaxation is that the host directory containing `runs.db` is mounted read-write at `/db` so SQLite WAL sidecars can live beside the shared DB. The container cannot touch host paths outside that DB directory. (Inside the container, `SimPlayer` uses a temp `HOME` under `/sim-home/sq-bench-{seed}-…` for the save file — deliberately **not** under `/tmp`, since a `/tmp` segment would mis-map the home to the Wasteland zone.)
- **Zone cwds are container-relative** (`driver.cwd_for_danger(danger, home)`): danger 1=the actual per-character `$HOME` (maps to Home Village only via exact `dirs::home_dir()` match in `src/zones.rs`), 2=`/zones/src`, 3=`/tmp`, 4=`/dev`, 5=`/zones/node_modules` (zone danger is matched by path *segment* in `src/zones.rs`; the `/zones/*` dirs live **outside** `/sim` so the read-only sim-code bind-mount can't hide them). Note: a prior latent bug used `node_modules-fake`, which has no `node_modules` segment and so mapped to The Wilds (danger 2) — meaning danger-5 (the Abyss) was never actually exercised before this fix.
- **Combat telemetry (`overworld_encounter`)**: under `SQ_DEBUG`, the `sq` game emits one machine-parseable line per resolved fight to **stderr**: `SQ_ENCOUNTER kind=<mob|boss> enemy=<lowercase-hex-utf8> elite=<0|1> dmg_dealt=<int> dmg_taken=<int> outcome=<kill|death|draw|win|loss|flee> xp=<int> gold=<int>` (emitted from `combat()` in `src/events.rs` and `tick_boss()` in `src/boss.rs`; enemy name is hex-encoded to dodge quoting). `driver.parse_encounter_lines()` decodes these from each tick's stderr and `player.py` inserts an `overworld_encounter` row. This is how boss encounters/outcomes, total mobs encountered, and damage dealt/taken (split mob-vs-boss) are captured — `report`/`dashboard` surface them as Boss Performance / Mob Encounters / Damage Summary. Bosses spawn 1/500 ticks, so short sweeps may have zero boss rows.
- **`SQ_NO_PACING=1`** is set for all sims — arena's 1.5s pacing must be off or runs hang.
- **Arena automation needs a PTY** + prompt matching (it's not a plain `subprocess.run`). Touch `driver.py` arena parsing carefully.
- **Live shared DB during a sweep** — `runs.db` is queryable mid-run from the host. `watch.py` polls it every 2s and now shows true in-flight runs (`ended_at IS NULL`) plus their latest committed snapshots.
- `runs.db`, its `-wal`/`-shm` sidecars, transient `runs.db.rpc/` / `runs.db.lockdir`, `dashboard.html`, old `runs-w*.db` artifacts, and `.sq-linux/` are **disposable artifacts** — `just clean-sims` before a fresh sweep; keep them out of commits.
- Validate balance changes here **before** shipping a tuning release (see parent `AGENTS.md` → Balance Tuning).

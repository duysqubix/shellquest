#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
from __future__ import annotations

import argparse
import importlib
import json
import multiprocessing as mp
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path
from typing import Any, cast

sys.path.insert(0, str(Path(__file__).resolve().parent))

db = cast(Any, importlib.import_module("simulator.db"))
player = cast(Any, importlib.import_module("simulator.player"))


def _error_result(message: str, *, seed: int, cls: str, race: str,
                  strategy: str, worker_db: Path) -> dict[str, Any]:
    return {"error": message, "seed": seed, "class": cls,
            "race": race, "strategy": strategy,
            "worker_db": str(worker_db)}


def _snippet(text: str, limit: int = 500) -> str:
    text = text.strip()
    if not text:
        return "<empty>"
    if len(text) <= limit:
        return text
    return "..." + text[-limit:]


def parse_container_result(proc_returncode: int, stdout: str, stderr: str, *,
                           seed: int, cls: str, race: str, strategy: str,
                           worker_db: Path) -> dict[str, Any]:
    stdout_lines = [line.strip() for line in stdout.splitlines() if line.strip()]
    parsed = None
    parse_error = None
    if stdout_lines:
        try:
            parsed = json.loads(stdout_lines[-1])
        except json.JSONDecodeError as e:
            parse_error = e

    if isinstance(parsed, dict):
        if parsed.get("ok") is True and proc_returncode == 0:
            required_keys = ("run_id", "ended_reason", "final_state", "ticks")
            missing_keys = [key for key in required_keys if key not in parsed]
            if missing_keys:
                message = ("docker infra failure (parse-error): ok:true JSON "
                           f"missing required keys: {', '.join(missing_keys)}; "
                           f"stderr: {_snippet(stderr)}; stdout: {_snippet(stdout)}")
                return _error_result(message, seed=seed, cls=cls, race=race,
                                     strategy=strategy, worker_db=worker_db)
            return {"run_id": parsed["run_id"],
                    "ended_reason": parsed["ended_reason"],
                    "final_state": parsed["final_state"],
                    "ticks": parsed["ticks"],
                    "worker_db": str(worker_db)}
        if parsed.get("ok") is False:
            return _error_result(str(parsed.get("error", "simulation failed")),
                                 seed=seed, cls=cls, race=race,
                                 strategy=strategy, worker_db=worker_db)

    if proc_returncode == 125:
        reason = "docker infra failure (rc=125 daemon/run error)"
    else:
        reason = f"docker infra failure (rc={proc_returncode})"

    if parse_error is not None:
        reason += f": unparseable JSON result ({parse_error})"
    elif not stdout_lines:
        reason += ": no JSON result on stdout"
    else:
        reason += ": unexpected JSON result"

    reason += f"; stderr: {_snippet(stderr)}; stdout: {_snippet(stdout)}"
    return _error_result(reason, seed=seed, cls=cls, race=race,
                         strategy=strategy, worker_db=worker_db)


def _container_timeout(max_ticks: int) -> int:
    # 120s floor plus 2s per 100 simulated ticks: generous for legitimate
    # long runs, but bounded so a wedged container cannot hang the sweep.
    return max(120, max_ticks * 2 // 100 + 120)


def _unlink_db_artifacts(db_path: Path) -> None:
    db_path.unlink(missing_ok=True)
    Path(str(db_path) + "-wal").unlink(missing_ok=True)
    Path(str(db_path) + "-shm").unlink(missing_ok=True)


def _move_shard_artifacts(shard_path: Path, worker_db: Path) -> bool:
    if not shard_path.exists():
        return False
    _unlink_db_artifacts(worker_db)
    shutil.move(str(shard_path), str(worker_db))
    for suffix in ("-wal", "-shm"):
        sidecar = Path(str(shard_path) + suffix)
        if sidecar.exists():
            shutil.move(str(sidecar), str(worker_db) + suffix)
    return True


def _remove_container(container_name: str) -> None:
    try:
        subprocess.run(
            ["docker", "rm", "-f", container_name],
            capture_output=True, text=True, timeout=30,
        )
    except (OSError, subprocess.TimeoutExpired):
        pass


def _worker(args: tuple[str, str, str, int, str, str, int, int, int, int]) -> dict[str, Any]:
    (cls, race, strategy, seed, tuning_label, db_path,
     target_level, max_ticks, snapshot_every, min_arena_tier_index) = args
    worker_db = Path(db_path).with_name(f"runs-w{seed}.db")
    # Mount only this per-character directory as /out. The container writes its
    # shard there, then the host moves it to worker_db so main()'s merge glob
    # still finds runs-w*.db without giving containers writable access to the
    # balance-sim source directory or sibling shards.
    out_dir = worker_db.parent / "_shards" / str(seed)
    shutil.rmtree(out_dir, ignore_errors=True)
    out_dir.mkdir(parents=True, exist_ok=True)
    _unlink_db_artifacts(worker_db)
    shard_container_path = f"/out/{worker_db.name}"
    shard_host_path = out_dir / worker_db.name
    container_name = f"shellquest-sim-{seed}"
    sq_bin_host = os.environ.get(
        "SQ_BIN_HOST",
        str(Path(db_path).resolve().parents[2] / "target" / "debug" / "sq"),
    )
    sim_dir_host = os.environ.get("SIM_DIR_HOST",
                                  str(Path(__file__).resolve().parent))
    sim_image = os.environ.get("SIM_IMAGE", "shellquest-sim")
    argv = [
        "docker", "run", "--rm", "--network=none",
        "--name", container_name,
        "-v", f"{sq_bin_host}:/opt/sq:ro",
        "-v", f"{sim_dir_host}:/sim:ro",
        "-v", f"{out_dir}:/out",
        sim_image,
        "python3", "/sim/simulator/container_main.py",
        "--class", cls, "--race", race, "--strategy", strategy,
        "--seed", str(seed),
        "--tuning-label", tuning_label,
        "--target-level", str(target_level),
        "--max-ticks", str(max_ticks),
        "--snapshot-every", str(snapshot_every),
        "--min-arena-tier", str(min_arena_tier_index),
        "--shard-out", shard_container_path,
    ]
    if not Path(sq_bin_host).is_file():
        shutil.rmtree(out_dir, ignore_errors=True)
        message = ("docker infra failure (rc=125 daemon/run error): "
                   f"SQ_BIN_HOST mount source is not a file: {sq_bin_host}; "
                   "stderr: host preflight rejected sq binary mount; "
                   "stdout: <empty>")
        return _error_result(message, seed=seed, cls=cls, race=race,
                             strategy=strategy, worker_db=worker_db)
    if not Path(sim_dir_host).is_dir():
        shutil.rmtree(out_dir, ignore_errors=True)
        message = ("docker infra failure (rc=125 daemon/run error): "
                   f"SIM_DIR_HOST mount source is not a directory: {sim_dir_host}; "
                   "stderr: host preflight rejected sim code mount; "
                   "stdout: <empty>")
        return _error_result(message, seed=seed, cls=cls, race=race,
                             strategy=strategy, worker_db=worker_db)
    timeout = _container_timeout(max_ticks)
    try:
        _remove_container(container_name)
        proc = subprocess.run(
            argv, capture_output=True, text=True, timeout=timeout,
        )
    except subprocess.TimeoutExpired:
        _remove_container(container_name)
        message = (f"container timeout after {timeout}s (max_ticks={max_ticks}; "
                   "budget=120s + 2s per 100 ticks)")
        return _error_result(message, seed=seed, cls=cls, race=race,
                             strategy=strategy, worker_db=worker_db)
    except OSError as e:
        message = f"docker infra failure before start: {type(e).__name__}: {e}"
        return _error_result(message, seed=seed, cls=cls, race=race,
                             strategy=strategy, worker_db=worker_db)
    finally:
        if 'proc' not in locals():
            shutil.rmtree(out_dir, ignore_errors=True)

    try:
        shard_moved = _move_shard_artifacts(shard_host_path, worker_db)
    except OSError as e:
        shutil.rmtree(out_dir, ignore_errors=True)
        message = f"docker infra failure moving shard: {type(e).__name__}: {e}"
        return _error_result(message, seed=seed, cls=cls, race=race,
                             strategy=strategy, worker_db=worker_db)
    finally:
        shutil.rmtree(out_dir, ignore_errors=True)

    result = parse_container_result(proc.returncode, proc.stdout, proc.stderr,
                                    seed=seed, cls=cls, race=race,
                                    strategy=strategy, worker_db=worker_db)
    if "error" not in result and not shard_moved:
        message = ("docker infra failure: container reported success but did "
                   f"not produce shard {shard_container_path}; "
                   f"stderr: {_snippet(proc.stderr)}; stdout: {_snippet(proc.stdout)}")
        return _error_result(message, seed=seed, cls=cls, race=race,
                             strategy=strategy, worker_db=worker_db)
    return result


def _format_eta(seconds: float) -> str:
    if seconds < 60:
        return f"{seconds:.0f}s"
    if seconds < 3600:
        return f"{int(seconds // 60)}m{int(seconds % 60):02d}s"
    return f"{int(seconds // 3600)}h{int((seconds % 3600) // 60):02d}m"


def main() -> int:
    p = argparse.ArgumentParser(
        description="balance-sim parallel runner",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    p.add_argument("--runs", type=int, default=1,
                   help="Runs per (class, race, strategy) combo")
    p.add_argument("--classes", nargs="+",
                   default=["Warrior", "Wizard", "Rogue", "Ranger", "Necromancer"])
    p.add_argument("--races", nargs="+",
                   default=["Human", "Elf", "Dwarf"])
    p.add_argument("--strategies", nargs="+",
                   default=["greedy", "balanced", "conservative"])
    p.add_argument("--tuning-label", default="v1.24-soft")
    p.add_argument("--target-level", type=int, default=60)
    p.add_argument("--max-ticks", type=int, default=0,
                   help="0 = auto-derive from target-level")
    p.add_argument("--snapshot-every", type=int, default=50)
    p.add_argument("--parallel", type=int, default=4)
    p.add_argument("--db", default=str(db.DEFAULT_DB_PATH))
    p.add_argument("--seed-base", type=int, default=1000)
    p.add_argument("--min-arena-tier", type=int, default=1, choices=[1, 2, 3, 4, 5],
                   help="Strategies only attempt arena at this tier index or higher "
                        "(1=Pit, 2=Gauntlet, 3=Colosseum, 4=Abyssal, 5=Godslayer)")
    args = p.parse_args()

    max_ticks = args.max_ticks or player.auto_max_ticks(args.target_level)

    jobs = []
    seed = args.seed_base
    for cls in args.classes:
        for race in args.races:
            for strat in args.strategies:
                for _ in range(args.runs):
                    jobs.append((cls, race, strat, seed, args.tuning_label,
                                 args.db, args.target_level, max_ticks,
                                 args.snapshot_every, args.min_arena_tier))
                    seed += 1

    tier_names = ["", "Pit", "Gauntlet", "Colosseum", "Abyssal", "Godslayer"]
    print(f"╭─ balance-sim ─────────────────────────────")
    print(f"│  {len(jobs)} sims × {args.parallel} workers")
    print(f"│  target L{args.target_level} · max {max_ticks} ticks · arena ≥ {tier_names[args.min_arena_tier]}")
    print(f"│  tuning_label: {args.tuning_label}")
    print(f"│  db: {args.db}")
    print(f"╰───────────────────────────────────────────")
    start = time.time()
    completed = 0
    successes = 0
    errors = 0
    with mp.Pool(processes=args.parallel) as pool:
        for result in pool.imap_unordered(_worker, jobs):
            completed += 1
            elapsed = time.time() - start
            avg_per_sim = elapsed / completed
            eta = avg_per_sim * (len(jobs) - completed) / max(1, args.parallel)
            prefix = f"[{completed:>3d}/{len(jobs)} · {completed * 100 // len(jobs):>3d}% · ETA {_format_eta(eta)}]"
            if "error" in result:
                errors += 1
                print(f"{prefix} ✗ {result['class']}-{result['race']}-{result['strategy']} seed={result['seed']}: {result['error']}")
            else:
                successes += 1
                fs = result["final_state"]
                end = result["ended_reason"]
                icon = "✓" if end == "target_reached" else ("⏱" if end == "max_ticks" else "?")
                print(f"{prefix} {icon} run={result['run_id']:<3d} L{fs['level']:<3d} "
                      f"HP={fs['max_hp']:<4d} gold={fs['gold']:<6d} ticks={result['ticks']:<5d} {end}")
    elapsed = time.time() - start
    print(f"\n✓ {successes}/{len(jobs)} succeeded, {errors} errors in {_format_eta(elapsed)}")
    print(f"Merging {args.parallel}+ worker DBs into {args.db}...")
    worker_dbs = sorted(Path(args.db).parent.glob("runs-w*.db"))
    merged = db.merge_dbs(Path(args.db), worker_dbs)
    print(f"Merged {merged} runs. Next: python3 dashboard.py --primary {args.tuning_label}")
    for wdb in worker_dbs:
        wdb.unlink(missing_ok=True)
        Path(str(wdb) + "-wal").unlink(missing_ok=True)
        Path(str(wdb) + "-shm").unlink(missing_ok=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())

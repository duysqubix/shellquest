#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
from __future__ import annotations

import argparse
import multiprocessing as mp
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from simulator import db, player


def _worker(args: tuple) -> dict:
    (cls, race, strategy, seed, tuning_label, db_path,
     target_level, max_ticks, snapshot_every, min_arena_tier_index) = args
    worker_db = Path(db_path).with_name(f"runs-w{seed}.db")
    try:
        result = player.simulate_one(
            cls, race, strategy, seed, tuning_label, worker_db,
            target_level=target_level, max_ticks=max_ticks,
            snapshot_every=snapshot_every,
            min_arena_tier_index=min_arena_tier_index,
        )
        result["worker_db"] = str(worker_db)
        return result
    except Exception as e:
        return {"error": str(e), "seed": seed, "class": cls,
                "race": race, "strategy": strategy,
                "worker_db": str(worker_db)}


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

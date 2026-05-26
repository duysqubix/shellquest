#!/usr/bin/env python3
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
     target_level, max_ticks, snapshot_every) = args
    worker_db = Path(db_path).with_name(f"runs-w{seed}.db")
    try:
        result = player.simulate_one(
            cls, race, strategy, seed, tuning_label, worker_db,
            target_level=target_level, max_ticks=max_ticks,
            snapshot_every=snapshot_every,
        )
        result["worker_db"] = str(worker_db)
        return result
    except Exception as e:
        return {"error": str(e), "seed": seed, "class": cls,
                "race": race, "strategy": strategy,
                "worker_db": str(worker_db)}


def main() -> int:
    p = argparse.ArgumentParser()
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
    p.add_argument("--max-ticks", type=int, default=4000)
    p.add_argument("--snapshot-every", type=int, default=50)
    p.add_argument("--parallel", type=int, default=4)
    p.add_argument("--db", default=str(db.DEFAULT_DB_PATH))
    p.add_argument("--seed-base", type=int, default=1000)
    args = p.parse_args()

    jobs = []
    seed = args.seed_base
    for cls in args.classes:
        for race in args.races:
            for strat in args.strategies:
                for _ in range(args.runs):
                    jobs.append((cls, race, strat, seed, args.tuning_label,
                                 args.db, args.target_level, args.max_ticks,
                                 args.snapshot_every))
                    seed += 1

    print(f"Dispatching {len(jobs)} sims across {args.parallel} workers")
    print(f"Database: {args.db}")
    print(f"Tuning label: {args.tuning_label}")
    start = time.time()
    completed = 0
    with mp.Pool(processes=args.parallel) as pool:
        for result in pool.imap_unordered(_worker, jobs):
            completed += 1
            if "error" in result:
                print(f"  [{completed}/{len(jobs)}] ERROR {result['class']}-{result['race']}-{result['strategy']} seed={result['seed']}: {result['error']}")
            else:
                fs = result["final_state"]
                print(f"  [{completed}/{len(jobs)}] run={result['run_id']:<5d} "
                      f"L{fs['level']:<3d} HP={fs['max_hp']:<4d} gold={fs['gold']:<7d} "
                      f"ticks={result['ticks']:<5d} {result['ended_reason']}")
    elapsed = time.time() - start
    print(f"\nSims done in {elapsed:.1f}s. Merging worker DBs into {args.db}...")
    worker_dbs = sorted(Path(args.db).parent.glob("runs-w*.db"))
    merged = db.merge_dbs(Path(args.db), worker_dbs)
    print(f"Merged {merged} runs into {args.db}")
    for wdb in worker_dbs:
        wdb.unlink(missing_ok=True)
        Path(str(wdb) + "-wal").unlink(missing_ok=True)
        Path(str(wdb) + "-shm").unlink(missing_ok=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())

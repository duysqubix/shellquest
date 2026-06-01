#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
from __future__ import annotations

import argparse
import contextlib
import importlib
import io
import json
import sys
import traceback
from pathlib import Path
from typing import Any, cast

SOURCE_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SOURCE_ROOT))

player = cast(Any, importlib.import_module("simulator.player"))


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description="balance-sim container entrypoint",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    p.add_argument("--class", dest="cls", required=True)
    p.add_argument("--race", required=True)
    p.add_argument("--strategy", required=True)
    p.add_argument("--seed", type=int, required=True)
    p.add_argument("--tuning-label", required=True)
    p.add_argument("--target-level", type=int, required=True)
    p.add_argument("--start-level", type=int, default=1)
    p.add_argument("--start-prestige", type=int, default=0)
    p.add_argument("--max-ticks", type=int, required=True,
                   help="0 = auto-derive from target-level and start-level")
    p.add_argument("--snapshot-every", type=int, default=50)
    p.add_argument("--min-arena-tier", type=int, default=1,
                   choices=[1, 2, 3, 4, 5])
    p.add_argument("--shard-out", required=True)
    return p.parse_args()


def run_simulation(args: argparse.Namespace) -> dict[str, Any]:
    max_ticks = args.max_ticks or player.auto_max_ticks(args.target_level, args.start_level)
    result = player.simulate_one(
        args.cls, args.race, args.strategy, args.seed, args.tuning_label,
        db_path=Path(args.shard_out),
        target_level=args.target_level, start_level=args.start_level,
        start_prestige=args.start_prestige, max_ticks=max_ticks,
        snapshot_every=args.snapshot_every,
        min_arena_tier_index=args.min_arena_tier,
    )
    ended_reason = result["ended_reason"]
    if isinstance(ended_reason, str) and ended_reason.startswith("error:"):
        raise RuntimeError(ended_reason)
    return {
        "ok": True,
        "run_id": result["run_id"],
        "ended_reason": ended_reason,
        "final_state": result["final_state"],
        "ticks": result["ticks"],
    }


def main() -> int:
    args = parse_args()
    captured_stdout = io.StringIO()
    try:
        with contextlib.redirect_stdout(captured_stdout):
            result = run_simulation(args)
        stray_stdout = captured_stdout.getvalue()
        if stray_stdout:
            print(stray_stdout, file=sys.stderr, end="")
        exit_code = 0
    except Exception as e:
        traceback.print_exc(file=sys.stderr)
        result = {
            "ok": False,
            "error": f"{type(e).__name__}: {e}",
            "seed": args.seed,
            "class": args.cls,
            "race": args.race,
            "strategy": args.strategy,
        }
        exit_code = 1
    print(json.dumps(result, separators=(",", ":")))
    return exit_code


if __name__ == "__main__":
    sys.exit(main())

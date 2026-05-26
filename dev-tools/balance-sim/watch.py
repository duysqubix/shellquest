#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
from __future__ import annotations

import argparse
import sqlite3
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from simulator import db


def query_state(conn: sqlite3.Connection) -> dict:
    total = conn.execute("SELECT COUNT(*) FROM run").fetchone()[0]
    in_flight = conn.execute(
        "SELECT COUNT(*) FROM run WHERE ended_at IS NULL"
    ).fetchone()[0]
    done = total - in_flight
    by_status = conn.execute(
        "SELECT ended_reason, COUNT(*) FROM run "
        "WHERE ended_at IS NOT NULL GROUP BY ended_reason"
    ).fetchall()
    arena_totals = conn.execute(
        "SELECT COUNT(*), SUM(CASE WHEN outcome='defeat' THEN 1 ELSE 0 END) "
        "FROM arena_attempt"
    ).fetchone()
    tier_totals = conn.execute(
        "SELECT tier, COUNT(*) FROM arena_attempt GROUP BY tier_index, tier "
        "ORDER BY tier_index"
    ).fetchall()
    return {
        "total": total, "in_flight": in_flight, "done": done,
        "by_status": list(by_status),
        "arena_attempts": arena_totals[0] or 0,
        "arena_defeats": arena_totals[1] or 0,
        "tier_attempts": list(tier_totals),
    }


def query_recent_activity(conn: sqlite3.Connection, since: float) -> list[dict]:
    rows = conn.execute(
        """
        SELECT r.id, r.class, r.race, r.strategy, MAX(ts.tick_no) AS tick_no,
               MAX(ts.level) AS level, MAX(ts.gold) AS gold
          FROM run r LEFT JOIN tick_snapshot ts ON ts.run_id = r.id
         WHERE r.started_at >= ?
         GROUP BY r.id
         ORDER BY r.id DESC LIMIT 12
        """, (since,)
    ).fetchall()
    return [dict(r) for r in rows]


def render(state: dict, recent: list[dict], iter_no: int) -> None:
    sys.stdout.write("\033[2J\033[H")
    print(f"╭─ balance-sim watch · iter {iter_no} ──────────")
    print(f"│  Total runs in DB: {state['total']}")
    print(f"│  In flight: {state['in_flight']}    Done: {state['done']}")
    if state["by_status"]:
        statuses = "  ".join(f"{s[0]}:{s[1]}" for s in state["by_status"])
        print(f"│  Outcomes: {statuses}")
    print(f"│  Arena attempts: {state['arena_attempts']}  "
          f"(defeats: {state['arena_defeats']})")
    if state["tier_attempts"]:
        print(f"│  Tier mix: " + "  ".join(f"{t[0]}:{t[1]}" for t in state["tier_attempts"]))
    print(f"╰───────────────────────────────────────────")
    print()
    print(f"{'RUN':>4} {'CLASS':<12} {'RACE':<7} {'STRAT':<14} {'TICKS':>6} {'LVL':>4} {'GOLD':>6}")
    for r in recent:
        print(f"{r['id']:>4} {r['class']:<12} {r['race']:<7} {r['strategy']:<14} "
              f"{(r['tick_no'] or 0):>6} {(r['level'] or 0):>4} {(r['gold'] or 0):>6}")
    sys.stdout.flush()


def main() -> int:
    p = argparse.ArgumentParser(description="live tail of balance-sim DB")
    p.add_argument("--db", default=str(db.DEFAULT_DB_PATH))
    p.add_argument("--interval", type=float, default=2.0,
                   help="refresh interval in seconds")
    p.add_argument("--since-minutes", type=float, default=30.0,
                   help="show runs started in the last N minutes")
    args = p.parse_args()

    db_path = Path(args.db)
    if not db_path.exists():
        print(f"DB {db_path} does not exist yet — start a sim first.", file=sys.stderr)
        return 1

    since = time.time() - args.since_minutes * 60.0
    iter_no = 0
    try:
        while True:
            iter_no += 1
            conn = db.open_db(db_path)
            try:
                state = query_state(conn)
                recent = query_recent_activity(conn, since)
            finally:
                conn.close()
            render(state, recent, iter_no)
            time.sleep(args.interval)
    except KeyboardInterrupt:
        print("\nbye")
        return 0


if __name__ == "__main__":
    sys.exit(main())

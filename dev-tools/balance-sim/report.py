#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from simulator import db


def ascii_hist(values: list[float], width: int = 40, buckets: int = 10) -> str:
    if not values:
        return "(no data)"
    lo, hi = min(values), max(values)
    if lo == hi:
        return f"{lo:.1f} ×{len(values)}"
    edges = [lo + (hi - lo) * i / buckets for i in range(buckets + 1)]
    counts = [0] * buckets
    for v in values:
        idx = min(buckets - 1, int((v - lo) / (hi - lo) * buckets))
        counts[idx] += 1
    max_count = max(counts)
    lines = []
    for i, c in enumerate(counts):
        bar = "█" * int(c / max_count * width) if max_count else ""
        lines.append(f"  [{edges[i]:>7.1f}–{edges[i+1]:>7.1f}] {bar} {c}")
    return "\n".join(lines)


def run_summary(conn, tuning_label: str) -> str:
    rows = conn.execute(
        """
        SELECT class, race, strategy,
               COUNT(*) AS n_runs,
               AVG(final_level) AS avg_level,
               AVG(final_gold)  AS avg_gold,
               AVG(final_kills) AS avg_kills,
               AVG(final_deaths) AS avg_deaths,
               AVG(total_ticks) AS avg_ticks,
               AVG(final_max_hp) AS avg_max_hp,
               AVG(final_attack_power) AS avg_atk,
               AVG(final_defense) AS avg_def
          FROM run
         WHERE tuning_label = ?
         GROUP BY class, race, strategy
         ORDER BY class, race, strategy
        """, (tuning_label,)
    ).fetchall()
    if not rows:
        return f"No runs for tuning_label='{tuning_label}'"
    lines = ["| Class | Race | Strategy | N | AvgLvl | AvgGold | Kills | Deaths | Ticks | MaxHP | ATK | DEF |",
             "|---|---|---|---|---|---|---|---|---|---|---|---|"]
    for r in rows:
        lines.append(
            f"| {r['class']} | {r['race']} | {r['strategy']} | {r['n_runs']} "
            f"| {r['avg_level']:.1f} | {int(r['avg_gold'])} "
            f"| {r['avg_kills']:.1f} | {r['avg_deaths']:.1f} "
            f"| {int(r['avg_ticks'])} | {int(r['avg_max_hp'])} "
            f"| {int(r['avg_atk'])} | {int(r['avg_def'])} |"
        )
    return "\n".join(lines)


def arena_summary(conn, tuning_label: str) -> str:
    rows = conn.execute(
        """
        SELECT r.class, r.strategy, aa.tier,
               COUNT(*) AS attempts,
               SUM(CASE WHEN aa.outcome='victory' OR aa.outcome='cashout' THEN 1 ELSE 0 END) AS survived,
               SUM(CASE WHEN aa.outcome='defeat' THEN 1 ELSE 0 END) AS deaths,
               AVG(aa.rounds_won) AS avg_rounds,
               AVG(aa.dmg_taken) AS avg_dmg_taken,
               AVG(aa.enemy_crits) AS avg_e_crits,
               AVG(aa.player_crits) AS avg_p_crits
          FROM arena_attempt aa
          JOIN run r ON r.id = aa.run_id
         WHERE r.tuning_label = ?
         GROUP BY r.class, r.strategy, aa.tier
         ORDER BY r.class, r.strategy, aa.tier_index
        """, (tuning_label,)
    ).fetchall()
    if not rows:
        return "(no arena attempts recorded)"
    lines = ["| Class | Strategy | Tier | N | Survived | Deaths | Avg Rounds | Avg Dmg Taken | Avg E-Crits | Avg P-Crits |",
             "|---|---|---|---|---|---|---|---|---|---|"]
    for r in rows:
        lines.append(
            f"| {r['class']} | {r['strategy']} | {r['tier']} | {r['attempts']} "
            f"| {r['survived']} | {r['deaths']} | {r['avg_rounds']:.1f} "
            f"| {r['avg_dmg_taken']:.1f} | {r['avg_e_crits']:.2f} | {r['avg_p_crits']:.2f} |"
        )
    return "\n".join(lines)


def progression_curve(conn, tuning_label: str, cls: str, strategy: str) -> str:
    rows = conn.execute(
        """
        SELECT ts.tick_no, AVG(ts.level) AS lvl, AVG(ts.gold) AS gold
          FROM tick_snapshot ts
          JOIN run r ON r.id = ts.run_id
         WHERE r.tuning_label = ? AND r.class = ? AND r.strategy = ?
         GROUP BY ts.tick_no
         ORDER BY ts.tick_no
        """, (tuning_label, cls, strategy)
    ).fetchall()
    if not rows:
        return f"(no snapshots for {cls} {strategy})"
    levels = [r["lvl"] for r in rows]
    return ascii_hist(levels, width=30, buckets=12)


def time_to_level(conn, tuning_label: str, threshold: int = 25) -> str:
    rows = conn.execute(
        """
        SELECT r.class, r.strategy, MIN(ts.tick_no) AS ticks_to_lvl
          FROM run r
          JOIN tick_snapshot ts ON ts.run_id = r.id
         WHERE r.tuning_label = ? AND ts.level >= ?
         GROUP BY r.id, r.class, r.strategy
        """, (tuning_label, threshold)
    ).fetchall()
    if not rows:
        return f"(no runs reached L{threshold})"
    by_combo = {}
    for r in rows:
        k = (r["class"], r["strategy"])
        by_combo.setdefault(k, []).append(r["ticks_to_lvl"])
    lines = [f"### Time to L{threshold} (median ticks)",
             "| Class | Strategy | Runs | Median | P10 | P90 |",
             "|---|---|---|---|---|---|"]
    for (cls, strat), ticks in sorted(by_combo.items()):
        ticks = sorted(ticks)
        median = ticks[len(ticks) // 2]
        p10 = ticks[max(0, len(ticks) // 10)]
        p90 = ticks[min(len(ticks) - 1, len(ticks) * 9 // 10)]
        lines.append(f"| {cls} | {strat} | {len(ticks)} | {median} | {p10} | {p90} |")
    return "\n".join(lines)


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--tuning-label", required=True)
    p.add_argument("--db", default=str(db.DEFAULT_DB_PATH))
    p.add_argument("--output", default=None)
    args = p.parse_args()

    conn = db.open_db(Path(args.db))
    out_lines = [
        f"# balance-sim report — tuning_label = `{args.tuning_label}`",
        "",
        "## Lifetime Summary (per class × race × strategy)",
        "",
        run_summary(conn, args.tuning_label),
        "",
        "## Arena Performance",
        "",
        arena_summary(conn, args.tuning_label),
        "",
        time_to_level(conn, args.tuning_label, 25),
        "",
        time_to_level(conn, args.tuning_label, 60),
        "",
    ]
    text = "\n".join(out_lines)
    if args.output:
        Path(args.output).write_text(text)
        print(f"Report written to {args.output}")
    else:
        print(text)
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
from __future__ import annotations

import argparse
import sys
import sqlite3
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


def boss_summary(conn, tuning_label: str) -> str:
    rows = conn.execute(
        """
        SELECT r.class, r.strategy, oe.enemy_name,
               COUNT(*) AS encounters,
               SUM(CASE WHEN oe.outcome='win' THEN 1 ELSE 0 END) AS wins,
               SUM(CASE WHEN oe.outcome='loss' THEN 1 ELSE 0 END) AS losses,
               SUM(CASE WHEN oe.outcome='flee' THEN 1 ELSE 0 END) AS flees,
               AVG(oe.dmg_dealt) AS avg_dmg_dealt,
               AVG(oe.dmg_taken) AS avg_dmg_taken
          FROM overworld_encounter oe
          JOIN run r ON r.id = oe.run_id
         WHERE r.tuning_label = ? AND oe.kind = 'boss'
         GROUP BY r.class, r.strategy, oe.enemy_name
         ORDER BY r.class, r.strategy, oe.enemy_name
        """, (tuning_label,)
    ).fetchall()
    if not rows:
        return "(no boss encounters recorded — bosses spawn at 1/500, short sweeps may have none)"
    lines = ["| Class | Strategy | Boss | N | Wins | Losses | Flees | Win % | Avg Dmg Dealt | Avg Dmg Taken |",
             "|---|---|---|---|---|---|---|---|---|---|"]
    for r in rows:
        win_rate = 100 * r['wins'] / r['encounters'] if r['encounters'] else 0
        lines.append(
            f"| {r['class']} | {r['strategy']} | {r['enemy_name']} | {r['encounters']} "
            f"| {r['wins']} | {r['losses']} | {r['flees']} | {win_rate:.0f}% "
            f"| {r['avg_dmg_dealt']:.1f} | {r['avg_dmg_taken']:.1f} |"
        )
    return "\n".join(lines)


def mob_summary(conn, tuning_label: str) -> str:
    rows = conn.execute(
        """
        SELECT r.class, r.strategy,
               COUNT(*) AS encounters,
               SUM(CASE WHEN oe.outcome='kill' THEN 1 ELSE 0 END) AS kills,
               SUM(CASE WHEN oe.outcome='draw' THEN 1 ELSE 0 END) AS draws,
               SUM(CASE WHEN oe.outcome='death' THEN 1 ELSE 0 END) AS deaths,
               SUM(oe.elite) AS elites,
               AVG(oe.dmg_dealt) AS avg_dmg_dealt,
               AVG(oe.dmg_taken) AS avg_dmg_taken
          FROM overworld_encounter oe
          JOIN run r ON r.id = oe.run_id
         WHERE r.tuning_label = ? AND oe.kind = 'mob'
         GROUP BY r.class, r.strategy
         ORDER BY r.class, r.strategy
        """, (tuning_label,)
    ).fetchall()
    if not rows:
        return "(no mob encounters recorded)"
    lines = ["| Class | Strategy | N | Kills | Draws | Deaths | Elites | Avg Dmg Dealt | Avg Dmg Taken |",
             "|---|---|---|---|---|---|---|---|---|"]
    for r in rows:
        lines.append(
            f"| {r['class']} | {r['strategy']} | {r['encounters']} "
            f"| {r['kills']} | {r['draws']} | {r['deaths']} | {r['elites']} "
            f"| {r['avg_dmg_dealt']:.1f} | {r['avg_dmg_taken']:.1f} |"
        )
    return "\n".join(lines)


def damage_summary(conn, tuning_label: str) -> str:
    rows = conn.execute(
        """
        SELECT r.class, r.strategy,
               SUM(CASE WHEN oe.kind='mob' THEN oe.dmg_dealt ELSE 0 END) AS mob_dmg_dealt,
               SUM(CASE WHEN oe.kind='mob' THEN oe.dmg_taken ELSE 0 END) AS mob_dmg_taken,
               SUM(CASE WHEN oe.kind='boss' THEN oe.dmg_dealt ELSE 0 END) AS boss_dmg_dealt,
               SUM(CASE WHEN oe.kind='boss' THEN oe.dmg_taken ELSE 0 END) AS boss_dmg_taken,
               AVG(CASE WHEN oe.kind='mob' THEN oe.dmg_dealt END) AS avg_mob_dmg_dealt,
               AVG(CASE WHEN oe.kind='mob' THEN oe.dmg_taken END) AS avg_mob_dmg_taken,
               AVG(CASE WHEN oe.kind='boss' THEN oe.dmg_dealt END) AS avg_boss_dmg_dealt,
               AVG(CASE WHEN oe.kind='boss' THEN oe.dmg_taken END) AS avg_boss_dmg_taken
          FROM overworld_encounter oe
          JOIN run r ON r.id = oe.run_id
         WHERE r.tuning_label = ?
         GROUP BY r.class, r.strategy
         ORDER BY r.class, r.strategy
        """, (tuning_label,)
    ).fetchall()
    if not rows:
        return "(no overworld encounters recorded)"

    def fmt(v):
        return f"{v:.1f}" if v is not None else "—"

    lines = ["| Class | Strategy | Mob Dmg Dealt (tot/avg) | Mob Dmg Taken (tot/avg) | Boss Dmg Dealt (tot/avg) | Boss Dmg Taken (tot/avg) |",
             "|---|---|---|---|---|---|"]
    for r in rows:
        lines.append(
            f"| {r['class']} | {r['strategy']} "
            f"| {int(r['mob_dmg_dealt'])} / {fmt(r['avg_mob_dmg_dealt'])} "
            f"| {int(r['mob_dmg_taken'])} / {fmt(r['avg_mob_dmg_taken'])} "
            f"| {int(r['boss_dmg_dealt'])} / {fmt(r['avg_boss_dmg_dealt'])} "
            f"| {int(r['boss_dmg_taken'])} / {fmt(r['avg_boss_dmg_taken'])} |"
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


def final_loadout_summary(conn, tuning_label: str) -> str:
    """End-state gear/inventory rollup per class × strategy: avg equipped
    effective power, avg held-inventory size, and equipped Rare+ count.
    Guarded against an old DB that predates the final_item table."""
    try:
        rows = conn.execute(
            """
            SELECT r.class, r.strategy,
                   COUNT(DISTINCT r.id) AS n_runs,
                   AVG(CASE WHEN fi.equipped=1 THEN fi.power + fi.enchant_level END) AS avg_eq_power,
                   SUM(CASE WHEN fi.equipped=1 THEN 1 ELSE 0 END) AS eq_count,
                   SUM(CASE WHEN fi.equipped=0 THEN 1 ELSE 0 END) AS inv_count,
                   SUM(CASE WHEN fi.equipped=1 AND fi.rarity IN ('Rare','Epic','Legendary')
                            THEN 1 ELSE 0 END) AS eq_rare_plus
              FROM final_item fi JOIN run r ON r.id = fi.run_id
             WHERE r.tuning_label = ?
             GROUP BY r.class, r.strategy
             ORDER BY r.class, r.strategy
            """, (tuning_label,)
        ).fetchall()
    except sqlite3.OperationalError as e:
        if "no such table" in str(e).lower():
            return "(no final_item data — re-run sims to populate end-state loadout)"
        raise
    if not rows:
        return "(no final_item data for this tuning_label)"

    def fmt(v):
        return f"{v:.1f}" if v is not None else "—"

    lines = ["| Class | Strategy | Runs | Avg Equipped Power | Avg Inv Size | Equipped Rare+ |",
             "|---|---|---|---|---|---|"]
    for r in rows:
        n = r["n_runs"] or 1
        lines.append(
            f"| {r['class']} | {r['strategy']} | {r['n_runs']} "
            f"| {fmt(r['avg_eq_power'])} "
            f"| {r['inv_count'] / n:.1f} "
            f"| {r['eq_rare_plus']} |"
        )
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
        "## Boss Performance",
        "",
        boss_summary(conn, args.tuning_label),
        "",
        "## Mob Encounters",
        "",
        mob_summary(conn, args.tuning_label),
        "",
        "## Damage Summary (overworld mob vs boss)",
        "",
        damage_summary(conn, args.tuning_label),
        "",
        time_to_level(conn, args.tuning_label, 25),
        "",
        time_to_level(conn, args.tuning_label, 60),
        "",
        "## Final Loadout (end-state equipped gear + inventory)",
        "",
        final_loadout_summary(conn, args.tuning_label),
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

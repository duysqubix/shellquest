#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
from __future__ import annotations

import argparse
import json
import sqlite3
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from simulator import db
import os
import subprocess

RARITY_ORDER = ["Common", "Uncommon", "Rare", "Epic", "Legendary"]
SLOT_ORDER = ["Weapon", "Armor", "Ring", "Potion"]


def _resolve_sq_bin() -> str | None:
    """Find the host sq binary for the static item-catalog command.
    The dashboard runs on the host (not a container), so a host-built
    (even Mach-O) binary is fine. Returns None if not found."""
    env = os.environ.get("SQ_BIN_HOST")
    if env and Path(env).is_file():
        return env
    repo_root = Path(__file__).resolve().parents[2]
    for rel in ("target/debug/sq", "target/release/sq"):
        cand = repo_root / rel
        if cand.is_file():
            return str(cand)
    return None


def load_catalog() -> dict | None:
    """Run `sq items --json` to fetch the static 160-item catalog.
    Catalog is label-independent and needs no runs.db. Returns None on any
    failure (missing binary / non-zero exit / bad JSON) so the dashboard
    never crashes — the Items tab renders a placeholder instead."""
    sq = _resolve_sq_bin()
    if not sq:
        return None
    try:
        proc = subprocess.run(
            [sq, "items", "--json"],
            capture_output=True, text=True, timeout=20,
        )
        if proc.returncode != 0:
            return None
        cat = json.loads(proc.stdout)
    except (OSError, subprocess.SubprocessError, json.JSONDecodeError):
        return None
    if not isinstance(cat, dict) or not cat.get("items"):
        return None
    return cat


def load_bestiary() -> dict | None:
    """Run `sq bestiary --json` to fetch the static boss roster + monster
    bestiary. Label-independent, needs no runs.db. Returns None on any failure
    so the dashboard never crashes — the Bestiary tab renders a placeholder."""
    sq = _resolve_sq_bin()
    if not sq:
        return None
    try:
        proc = subprocess.run(
            [sq, "bestiary", "--json"],
            capture_output=True, text=True, timeout=20,
        )
        if proc.returncode != 0:
            return None
        best = json.loads(proc.stdout)
    except (OSError, subprocess.SubprocessError, json.JSONDecodeError):
        return None
    if not isinstance(best, dict) or not best.get("bosses") or not best.get("monsters"):
        return None
    return best

TIER_ORDER = ["Pit", "Gauntlet", "Colosseum", "Abyssal", "Godslayer"]


def list_tuning_labels(conn: sqlite3.Connection) -> list[str]:
    try:
        return [r[0] for r in conn.execute(
            "SELECT DISTINCT tuning_label FROM run ORDER BY tuning_label"
        )]
    except sqlite3.OperationalError as e:
        # Only treat a missing 'run' table (fresh/empty DB) as 'no sim data';
        # re-raise real DB failures (locked db, corruption, etc.).
        if "no such table" in str(e).lower():
            return []
        raise


def lifetime_summary(conn: sqlite3.Connection, label: str) -> list[dict]:
    return [dict(r) for r in conn.execute(
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
          FROM run WHERE tuning_label = ?
         GROUP BY class, race, strategy
         ORDER BY class, strategy
        """, (label,)
    )]


def arena_summary(conn: sqlite3.Connection, label: str) -> list[dict]:
    return [dict(r) for r in conn.execute(
        """
        SELECT r.class, r.strategy, aa.tier, aa.tier_index,
               COUNT(*) AS attempts,
               SUM(CASE WHEN aa.outcome IN ('victory','cashout') THEN 1 ELSE 0 END) AS survived,
               SUM(CASE WHEN aa.outcome = 'defeat' THEN 1 ELSE 0 END) AS deaths,
               AVG(aa.rounds_won) AS avg_rounds,
               AVG(aa.dmg_taken) AS avg_dmg_taken,
               AVG(aa.dmg_dealt) AS avg_dmg_dealt,
               AVG(aa.enemy_crits) AS avg_e_crits,
               AVG(aa.player_crits) AS avg_p_crits,
               AVG(aa.player_swings) AS avg_p_swings,
               AVG(aa.enemy_swings) AS avg_e_swings
          FROM arena_attempt aa JOIN run r ON r.id = aa.run_id
         WHERE r.tuning_label = ?
         GROUP BY r.class, r.strategy, aa.tier, aa.tier_index
         ORDER BY r.class, r.strategy, aa.tier_index
        """, (label,)
    )]


def boss_summary(conn: sqlite3.Connection, label: str) -> list[dict]:
    return [dict(r) for r in conn.execute(
        """
        SELECT r.class, r.strategy, oe.enemy_name,
               COUNT(*) AS encounters,
               SUM(CASE WHEN oe.outcome='win' THEN 1 ELSE 0 END) AS wins,
               SUM(CASE WHEN oe.outcome='loss' THEN 1 ELSE 0 END) AS losses,
               SUM(CASE WHEN oe.outcome='flee' THEN 1 ELSE 0 END) AS flees,
               AVG(oe.dmg_dealt) AS avg_dmg_dealt,
               AVG(oe.dmg_taken) AS avg_dmg_taken
          FROM overworld_encounter oe JOIN run r ON r.id = oe.run_id
         WHERE r.tuning_label = ? AND oe.kind = 'boss'
         GROUP BY r.class, r.strategy, oe.enemy_name
         ORDER BY r.class, r.strategy, oe.enemy_name
        """, (label,)
    )]


def mob_summary(conn: sqlite3.Connection, label: str) -> list[dict]:
    return [dict(r) for r in conn.execute(
        """
        SELECT r.class, r.strategy,
               COUNT(*) AS encounters,
               SUM(CASE WHEN oe.outcome='kill' THEN 1 ELSE 0 END) AS kills,
               SUM(CASE WHEN oe.outcome='draw' THEN 1 ELSE 0 END) AS draws,
               SUM(CASE WHEN oe.outcome='death' THEN 1 ELSE 0 END) AS deaths,
               SUM(oe.elite) AS elites,
               AVG(oe.dmg_dealt) AS avg_dmg_dealt,
               AVG(oe.dmg_taken) AS avg_dmg_taken
          FROM overworld_encounter oe JOIN run r ON r.id = oe.run_id
         WHERE r.tuning_label = ? AND oe.kind = 'mob'
         GROUP BY r.class, r.strategy
         ORDER BY r.class, r.strategy
        """, (label,)
    )]


def damage_summary(conn: sqlite3.Connection, label: str) -> list[dict]:
    return [dict(r) for r in conn.execute(
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
          FROM overworld_encounter oe JOIN run r ON r.id = oe.run_id
         WHERE r.tuning_label = ?
         GROUP BY r.class, r.strategy
         ORDER BY r.class, r.strategy
        """, (label,)
    )]


def progression_curve(conn: sqlite3.Connection, label: str) -> dict:
    rows = conn.execute(
        """
        SELECT r.class, r.strategy, ts.tick_no,
               AVG(ts.level) AS avg_level,
               AVG(ts.attack_power) AS avg_atk,
               AVG(ts.defense) AS avg_def,
               AVG(ts.gold) AS avg_gold,
               AVG(ts.max_hp) AS avg_max_hp
          FROM tick_snapshot ts JOIN run r ON r.id = ts.run_id
         WHERE r.tuning_label = ?
         GROUP BY r.class, r.strategy, ts.tick_no
         ORDER BY r.class, r.strategy, ts.tick_no
        """, (label,)
    ).fetchall()
    series: dict[str, list] = {}
    for r in rows:
        key = f"{r['class']}_{r['strategy']}"
        series.setdefault(key, []).append({
            "tick": r["tick_no"],
            "level": round(r["avg_level"], 1),
            "atk": round(r["avg_atk"], 1),
            "def": round(r["avg_def"], 1),
            "gold": round(r["avg_gold"], 0),
            "max_hp": round(r["avg_max_hp"], 1),
        })
    return series


def time_to_level(conn: sqlite3.Connection, label: str, threshold: int) -> list[dict]:
    rows = conn.execute(
        """
        SELECT r.class, r.strategy, MIN(ts.tick_no) AS ticks
          FROM run r JOIN tick_snapshot ts ON ts.run_id = r.id
         WHERE r.tuning_label = ? AND ts.level >= ?
         GROUP BY r.id, r.class, r.strategy
        """, (label, threshold)
    ).fetchall()
    by_combo: dict[tuple, list[int]] = {}
    for r in rows:
        by_combo.setdefault((r["class"], r["strategy"]), []).append(r["ticks"])
    out = []
    for (cls, strat), ticks in sorted(by_combo.items()):
        ticks.sort()
        n = len(ticks)
        out.append({
            "class": cls, "strategy": strat, "n": n,
            "median": ticks[n // 2] if n else None,
            "p10": ticks[max(0, n // 10)] if n else None,
            "p90": ticks[min(n - 1, n * 9 // 10)] if n else None,
            "min": ticks[0] if n else None,
            "max": ticks[-1] if n else None,
        })
    return out


def final_loadout(conn: sqlite3.Connection, label: str) -> list[dict]:
    """Every end-state item (equipped gear + held inventory) for a label,
    joined to its run's metadata. Powers the Loadout tab's per-run drill-down
    AND its aggregates (the JS groups these rows by run_id). Guarded against
    an old DB that predates the final_item table (returns [])."""
    try:
        return [dict(r) for r in conn.execute(
            """
            SELECT fi.run_id, r.class, r.race, r.strategy, r.seed,
                   fi.slot, fi.equipped, fi.name, fi.rarity,
                   fi.power, fi.enchant_level
              FROM final_item fi JOIN run r ON r.id = fi.run_id
             WHERE r.tuning_label = ?
             ORDER BY fi.run_id, fi.equipped DESC, fi.slot
            """, (label,)
        )]
    except sqlite3.OperationalError as e:
        if "no such table" in str(e).lower():
            return []
        raise


def build_label_payload(conn: sqlite3.Connection, label: str) -> dict:
    run_count = conn.execute(
        "SELECT COUNT(*) FROM run WHERE tuning_label = ?", (label,)
    ).fetchone()[0]
    return {
        "label": label,
        "run_count": run_count,
        "lifetime": lifetime_summary(conn, label),
        "arena": arena_summary(conn, label),
        "boss": boss_summary(conn, label),
        "mob": mob_summary(conn, label),
        "damage": damage_summary(conn, label),
        "progression": progression_curve(conn, label),
        "time_to_l25": time_to_level(conn, label, 25),
        "time_to_l40": time_to_level(conn, label, 40),
        "time_to_l60": time_to_level(conn, label, 60),
        "loadout": final_loadout(conn, label),
    }


def render_html(data: dict) -> str:
    json_blob = json.dumps(data, default=str)
    return f"""<!doctype html>
<html lang=\"en\">
<head>
<meta charset=\"utf-8\">
<title>shellquest balance-sim dashboard</title>
<script src=\"https://cdn.jsdelivr.net/npm/chart.js@4.4.0/dist/chart.umd.min.js\"></script>
<style>
:root {{
  --bg: #0d1117; --fg: #e6edf3; --muted: #8b949e; --card: #161b22;
  --border: #30363d; --accent: #58a6ff; --good: #3fb950;
  --warn: #d29922; --bad: #f85149;
}}
* {{ box-sizing: border-box; }}
body {{ margin: 0; font: 14px -apple-system, BlinkMacSystemFont, 'Segoe UI',
  Helvetica, Arial, sans-serif; background: var(--bg); color: var(--fg); }}
header {{ padding: 16px 24px; border-bottom: 1px solid var(--border);
  display: flex; align-items: center; gap: 24px; flex-wrap: wrap; }}
header h1 {{ font-size: 18px; margin: 0; font-weight: 600; }}
header .meta {{ color: var(--muted); font-size: 12px; }}
.controls {{ margin-left: auto; display: flex; gap: 12px; align-items: center; }}
.controls label {{ color: var(--muted); font-size: 12px; }}
.controls select {{ background: var(--card); color: var(--fg);
  border: 1px solid var(--border); padding: 6px 10px; border-radius: 6px; font: inherit; }}
main {{ padding: 24px; display: grid; gap: 16px; max-width: 1600px; margin: 0 auto; }}
.grid {{ display: grid; gap: 16px; grid-template-columns: repeat(auto-fit, minmax(420px, 1fr)); }}
.card {{ background: var(--card); border: 1px solid var(--border); border-radius: 8px;
  padding: 16px; }}
.card h2 {{ margin: 0 0 12px; font-size: 14px; font-weight: 600; color: var(--fg); }}
.card .desc {{ color: var(--muted); font-size: 12px; margin-bottom: 12px; }}
canvas {{ max-height: 320px; }}
table {{ width: 100%; border-collapse: collapse; font-size: 12px; }}
th, td {{ padding: 6px 8px; text-align: left; border-bottom: 1px solid var(--border); }}
th {{ color: var(--muted); font-weight: 500; }}
.stats-grid {{ display: grid; gap: 8px; grid-template-columns: repeat(auto-fit, minmax(140px, 1fr)); }}
.stat {{ background: rgba(88, 166, 255, 0.08); border-left: 3px solid var(--accent);
  padding: 8px 12px; border-radius: 4px; }}
.stat .label {{ color: var(--muted); font-size: 11px; text-transform: uppercase;
  letter-spacing: 0.5px; }}
.stat .value {{ font-size: 20px; font-weight: 600; color: var(--fg); }}
.stat .delta {{ font-size: 11px; color: var(--muted); }}
.stat .delta.up {{ color: var(--good); }}
.stat .delta.down {{ color: var(--bad); }}
.tabbar {{ display: flex; gap: 4px; border-bottom: 1px solid var(--border); margin-bottom: 16px; }}
.tab {{ padding: 8px 16px; background: none; border: none; color: var(--muted);
  cursor: pointer; font: inherit; border-bottom: 2px solid transparent; }}
.tab.active {{ color: var(--accent); border-bottom-color: var(--accent); }}
.tab-content {{ display: none; }}
.tab-content.active {{ display: block; }}
.empty {{ color: var(--muted); padding: 16px; text-align: center; font-style: italic; }}
</style>
</head>
<body>
<header>
  <h1>🛡️ shellquest balance-sim</h1>
  <span class=\"meta\" id=\"meta\"></span>
  <div class=\"controls\">
    <label>primary: <select id=\"primary-select\"></select></label>
    <label>compare: <select id=\"compare-select\"><option value=\"\">(none)</option></select></label>
  </div>
</header>
<main>
  <div class=\"tabbar\">
    <button class=\"tab active\" data-tab=\"summary\">Summary</button>
    <button class=\"tab\" data-tab=\"progression\">Progression</button>
    <button class=\"tab\" data-tab=\"arena\">Arena</button>
    <button class=\"tab\" data-tab=\"combat\">Combat</button>
    <button class=\"tab\" data-tab=\"loadout\">Loadout</button>
    <button class=\"tab\" data-tab=\"items\">Items</button>
    <button class=\"tab\" data-tab=\"bestiary\">Bestiary</button>
    <button class=\"tab\" data-tab=\"ab\">A/B Compare</button>
  </div>

  <section id=\"tab-summary\" class=\"tab-content active\">
    <div class=\"card\"><h2>Overview</h2><div class=\"stats-grid\" id=\"overview-cards\"></div></div>
    <div class=\"card\"><h2>Lifetime Summary (class × strategy)</h2>
      <div id=\"lifetime-table\"></div></div>
  </section>

  <section id=\"tab-progression\" class=\"tab-content\">
    <div class=\"grid\">
      <div class=\"card\"><h2>Level over time</h2>
        <div class=\"desc\">Average character level by tick, grouped by class × strategy.</div>
        <canvas id=\"chart-level\"></canvas></div>
      <div class=\"card\"><h2>Attack power over time</h2>
        <canvas id=\"chart-atk\"></canvas></div>
      <div class=\"card\"><h2>Defense over time</h2>
        <canvas id=\"chart-def\"></canvas></div>
      <div class=\"card\"><h2>Gold over time</h2>
        <canvas id=\"chart-gold\"></canvas></div>
      <div class=\"card\"><h2>Time to reach milestone levels (median ticks)</h2>
        <canvas id=\"chart-ttl\"></canvas></div>
    </div>
  </section>

  <section id=\"tab-arena\" class=\"tab-content\">
    <div class=\"grid\">
      <div class=\"card\"><h2>Survival rate by tier × class</h2>
        <div class=\"desc\">Percent of arena attempts that ended in cashout or victory (not KO).</div>
        <canvas id=\"chart-survival\"></canvas></div>
      <div class=\"card\"><h2>Average rounds reached by tier × class</h2>
        <canvas id=\"chart-rounds\"></canvas></div>
      <div class=\"card\"><h2>Damage taken per attempt by tier</h2>
        <canvas id=\"chart-dmg-taken\"></canvas></div>
      <div class=\"card\"><h2>Enemy crits per attempt by tier</h2>
        <div class=\"desc\">Crit chance scales with wave: 0% in rounds 1-3, up to 22% past round 40.</div>
        <canvas id=\"chart-ecrits\"></canvas></div>
      <div class=\"card\"><h2>Arena attempts table</h2>
        <div id=\"arena-table\"></div></div>
    </div>
  </section>

  <section id=\"tab-combat\" class=\"tab-content\">
    <div class=\"grid\">
      <div class=\"card\"><h2>Boss win rate by class</h2>
        <div class=\"desc\">Boss encounters spawn at ~1/500 ticks — short sweeps may have none.</div>
        <canvas id=\"chart-boss-winrate\"></canvas></div>
      <div class=\"card\"><h2>Mob outcomes by class</h2>
        <div class=\"desc\">Overworld mob encounter outcomes (kills / draws / deaths), summed by class.</div>
        <canvas id=\"chart-mob-outcomes\"></canvas></div>
      <div class=\"card\"><h2>Avg damage dealt: mob vs boss</h2>
        <canvas id=\"chart-dmg-dealt-kind\"></canvas></div>
      <div class=\"card\"><h2>Avg damage taken: mob vs boss</h2>
        <canvas id=\"chart-dmg-taken-kind\"></canvas></div>
      <div class=\"card\"><h2>Boss performance table</h2>
        <div id=\"boss-table\"></div></div>
      <div class=\"card\"><h2>Mob encounters table</h2>
        <div id=\"mob-table\"></div></div>
    </div>
  </section>

  <section id=\"tab-loadout\" class=\"tab-content\">
    <div class=\"card\"><h2>Per-run final loadout</h2>
      <div class=\"desc\">The END-STATE gear a single run was wearing plus the inventory it was holding when the sim ended (from the <code>final_item</code> table). Pick a run below.</div>
      <div class=\"controls\" style=\"margin:0 0 12px\">
        <label>run: <select id=\"loadout-run-select\"></select></label>
      </div>
      <div id=\"loadout-drilldown\"></div></div>
    <div class=\"grid\">
      <div class=\"card\"><h2>Avg equipped gear power by class</h2>
        <div class=\"desc\">Effective power (base + enchant) of the weapon/armor/ring runs ended with, averaged by class.</div>
        <canvas id=\"chart-loadout-power\"></canvas></div>
      <div class=\"card\"><h2>Equipped rarity distribution by class</h2>
        <div class=\"desc\">Rarity of every equipped slot a run ended with, stacked per class.</div>
        <canvas id=\"chart-loadout-rarity\"></canvas></div>
      <div class=\"card\"><h2>Avg inventory size by class</h2>
        <div class=\"desc\">Number of held (non-equipped) items at run end, averaged by class.</div>
        <canvas id=\"chart-loadout-invsize\"></canvas></div>
      <div class=\"card\"><h2>Most common ending weapons</h2>
        <div id=\"loadout-top-weapon\"></div></div>
      <div class=\"card\"><h2>Most common ending armor</h2>
        <div id=\"loadout-top-armor\"></div></div>
      <div class=\"card\"><h2>Most common ending rings</h2>
        <div id=\"loadout-top-ring\"></div></div>
    </div>
  </section>

  <section id=\"tab-items\" class=\"tab-content\">
    <div id=\"items-unavailable\" style=\"display:none\" class=\"card empty\">
      Item catalog unavailable — build the sq binary (<code>cargo build --bin sq</code>) and regenerate the dashboard.</div>
    <div id=\"items-content\">
      <div class=\"card\"><h2>Item catalog</h2>
        <div class=\"desc\">The game's static loot tables (from <code>sq items --json</code>) — independent of any simulation. <span id=\"items-total\"></span></div></div>
      <div class=\"grid\">
        <div class=\"card\"><h2>Items by rarity</h2>
          <canvas id=\"chart-item-rarity\"></canvas></div>
        <div class=\"card\"><h2>Items by rarity × slot</h2>
          <canvas id=\"chart-item-rarity-slot\"></canvas></div>
        <div class=\"card\"><h2>Power range by rarity (min / avg / max)</h2>
          <canvas id=\"chart-item-power-rarity\"></canvas></div>
        <div class=\"card\"><h2>Power distribution (all items, by midpoint)</h2>
          <canvas id=\"chart-item-power-hist\"></canvas></div>
        <div class=\"card\"><h2>Items by slot</h2>
          <canvas id=\"chart-item-slot\"></canvas></div>
        <div class=\"card\"><h2>Price range by rarity (gold)</h2>
          <canvas id=\"chart-item-price-rarity\"></canvas></div>
        <div class=\"card\"><h2>Drop weights &amp; power multipliers</h2>
          <div class=\"desc\">Reference: rarity drop probability and the price/power multiplier.</div>
          <div id=\"items-meta-table\"></div></div>
      </div>
    </div>
  </section>

  <section id=\"tab-bestiary\" class=\"tab-content\">
    <div id=\"bestiary-unavailable\" style=\"display:none\" class=\"card empty\">
      Bestiary unavailable — build the sq binary (<code>cargo build --bin sq</code>) and regenerate the dashboard.</div>
    <div id=\"bestiary-content\">
      <div class=\"card\"><h2>Bestiary</h2>
        <div class=\"desc\">The game's static monster bestiary and boss roster (from <code>sq bestiary --json</code>) — independent of any simulation. <span id=\"bestiary-total\"></span></div></div>
      <h2 style=\"margin:18px 0 6px\">Monsters</h2>
      <div class=\"grid\">
        <div class=\"card\"><h2>Monsters by tier</h2>
          <canvas id=\"chart-mob-count\"></canvas></div>
        <div class=\"card\"><h2>HP by tier (min / avg / max)</h2>
          <canvas id=\"chart-mob-hp\"></canvas></div>
        <div class=\"card\"><h2>Attack by tier (min / avg / max)</h2>
          <canvas id=\"chart-mob-atk\"></canvas></div>
        <div class=\"card\"><h2>XP by tier (min / avg / max)</h2>
          <canvas id=\"chart-mob-xp\"></canvas></div>
        <div class=\"card\"><h2>Tier → zone-danger spawn gating</h2>
          <div class=\"desc\">Which monster tiers can spawn at each zone danger level.</div>
          <div id=\"mob-tier-danger-table\"></div></div>
        <div class=\"card\"><h2>Elite / enraged modifiers</h2>
          <div class=\"desc\">Static multipliers applied to ~1/8 of spawns (“Enraged” variants).</div>
          <div id=\"mob-elite-table\"></div></div>
      </div>
      <h2 style=\"margin:18px 0 6px\">Bosses</h2>
      <div class=\"card\"><div class=\"desc\" id=\"boss-spawn-note\"></div></div>
      <div class=\"grid\">
        <div class=\"card\"><h2>HP per boss</h2>
          <canvas id=\"chart-boss-hp\"></canvas></div>
        <div class=\"card\"><h2>Attack per boss</h2>
          <canvas id=\"chart-boss-atk\"></canvas></div>
        <div class=\"card\"><h2>XP reward per boss</h2>
          <canvas id=\"chart-boss-xp\"></canvas></div>
        <div class=\"card\"><h2>Gold reward per boss</h2>
          <canvas id=\"chart-boss-gold\"></canvas></div>
        <div class=\"card\"><h2>Boss roster</h2>
          <div id=\"boss-roster-table\"></div></div>
      </div>
    </div>
  </section>

  <section id=\"tab-ab\" class=\"tab-content\">
    <div id=\"ab-content\"></div>
  </section>
</main>

<script>
const DATA = {json_blob};

const CLASS_COLOR = {{
  Warrior: '#f85149', Wizard: '#58a6ff', Rogue: '#d29922',
  Ranger: '#3fb950', Necromancer: '#bc8cff',
}};
const STRATEGY_DASH = {{ greedy: [], balanced: [6, 3], conservative: [2, 2] }};
const TIER_ORDER = {json.dumps(TIER_ORDER)};
const RARITY_ORDER = {json.dumps(RARITY_ORDER)};
const SLOT_ORDER = {json.dumps(SLOT_ORDER)};
const RARITY_COLOR = {{ Common: '#8b949e', Uncommon: '#3fb950', Rare: '#58a6ff', Epic: '#bc8cff', Legendary: '#d29922' }};

function getColor(cls, alpha) {{
  const c = CLASS_COLOR[cls] || '#8b949e';
  if (alpha === undefined) return c;
  const r = parseInt(c.slice(1, 3), 16);
  const g = parseInt(c.slice(3, 5), 16);
  const b = parseInt(c.slice(5, 7), 16);
  return `rgba(${{r}},${{g}},${{b}},${{alpha}})`;
}}

function setupChart(canvas, type, data, options) {{
  return new Chart(canvas, {{ type, data, options: Object.assign({{
    responsive: true, maintainAspectRatio: false,
    plugins: {{ legend: {{ labels: {{ color: '#e6edf3' }} }} }},
    scales: {{
      x: {{ ticks: {{ color: '#8b949e' }}, grid: {{ color: '#30363d' }} }},
      y: {{ ticks: {{ color: '#8b949e' }}, grid: {{ color: '#30363d' }} }},
    }},
  }}, options || {{}}) }});
}}

let chartsByLabel = {{}};

function renderOverview(payload) {{
  const totalRuns = payload.run_count;
  const reachedTarget = payload.lifetime.reduce((sum, r) => sum + r.n_runs, 0);
  const arenaAttempts = payload.arena.reduce((sum, r) => sum + r.attempts, 0);
  const arenaSurvived = payload.arena.reduce((sum, r) => sum + r.survived, 0);
  const survRate = arenaAttempts ? (100 * arenaSurvived / arenaAttempts).toFixed(0) : 0;
  const tiles = [
    ['Runs', totalRuns, ''],
    ['Class × Strategy', payload.lifetime.length, 'combos'],
    ['Arena Attempts', arenaAttempts, ''],
    ['Arena Survival', `${{survRate}}%`, ''],
  ];
  document.getElementById('overview-cards').innerHTML = tiles.map(([l, v, d]) =>
    `<div class=\"stat\"><div class=\"label\">${{l}}</div><div class=\"value\">${{v}}</div><div class=\"delta\">${{d}}</div></div>`
  ).join('');
}}

function renderLifetimeTable(payload) {{
  const headers = ['Class', 'Race', 'Strategy', 'N', 'AvgLvl', 'AvgGold', 'Kills', 'Deaths', 'Ticks', 'MaxHP', 'ATK', 'DEF'];
  const cols = ['class', 'race', 'strategy', 'n_runs', 'avg_level', 'avg_gold', 'avg_kills', 'avg_deaths', 'avg_ticks', 'avg_max_hp', 'avg_atk', 'avg_def'];
  const rows = payload.lifetime.map(r => `<tr>${{cols.map(c => {{
    let v = r[c];
    if (typeof v === 'number' && !Number.isInteger(v)) v = v.toFixed(1);
    return `<td>${{v}}</td>`;
  }}).join('')}}</tr>`).join('');
  document.getElementById('lifetime-table').innerHTML =
    `<table><thead><tr>${{headers.map(h => `<th>${{h}}</th>`).join('')}}</tr></thead><tbody>${{rows}}</tbody></table>`;
}}

function progressionDataset(payload, metric) {{
  return Object.entries(payload.progression).map(([key, points]) => {{
    const [cls, strat] = key.split('_');
    return {{
      label: key,
      data: points.map(p => ({{ x: p.tick, y: p[metric] }})),
      borderColor: getColor(cls),
      backgroundColor: getColor(cls, 0.15),
      borderDash: STRATEGY_DASH[strat] || [],
      tension: 0.2, pointRadius: 1,
    }};
  }});
}}

function renderProgression(payload) {{
  ['chart-level', 'chart-atk', 'chart-def', 'chart-gold'].forEach(id => {{
    if (chartsByLabel[id]) {{ chartsByLabel[id].destroy(); }}
  }});
  const opts = (ylabel) => ({{
    scales: {{
      x: {{ type: 'linear', title: {{ display: true, text: 'tick #', color: '#8b949e' }},
            ticks: {{ color: '#8b949e' }}, grid: {{ color: '#30363d' }} }},
      y: {{ title: {{ display: true, text: ylabel, color: '#8b949e' }},
            ticks: {{ color: '#8b949e' }}, grid: {{ color: '#30363d' }} }},
    }},
  }});
  chartsByLabel['chart-level'] = setupChart(
    document.getElementById('chart-level'), 'line',
    {{ datasets: progressionDataset(payload, 'level') }}, opts('avg level'));
  chartsByLabel['chart-atk'] = setupChart(
    document.getElementById('chart-atk'), 'line',
    {{ datasets: progressionDataset(payload, 'atk') }}, opts('avg attack_power'));
  chartsByLabel['chart-def'] = setupChart(
    document.getElementById('chart-def'), 'line',
    {{ datasets: progressionDataset(payload, 'def') }}, opts('avg defense'));
  chartsByLabel['chart-gold'] = setupChart(
    document.getElementById('chart-gold'), 'line',
    {{ datasets: progressionDataset(payload, 'gold') }}, opts('avg gold'));

  if (chartsByLabel['chart-ttl']) chartsByLabel['chart-ttl'].destroy();
  const combos = [...new Set([
    ...payload.time_to_l25.map(r => `${{r.class}}_${{r.strategy}}`),
    ...payload.time_to_l40.map(r => `${{r.class}}_${{r.strategy}}`),
    ...payload.time_to_l60.map(r => `${{r.class}}_${{r.strategy}}`),
  ])].sort();
  const ttlData = (rows) => combos.map(c => {{
    const [cls, strat] = c.split('_');
    const r = rows.find(x => x.class === cls && x.strategy === strat);
    return r ? r.median : null;
  }});
  chartsByLabel['chart-ttl'] = setupChart(
    document.getElementById('chart-ttl'), 'bar',
    {{ labels: combos, datasets: [
      {{ label: 'L25', data: ttlData(payload.time_to_l25), backgroundColor: getColor('Wizard', 0.7) }},
      {{ label: 'L40', data: ttlData(payload.time_to_l40), backgroundColor: getColor('Warrior', 0.7) }},
      {{ label: 'L60', data: ttlData(payload.time_to_l60), backgroundColor: getColor('Ranger', 0.7) }},
    ] }},
    {{ scales: {{
      y: {{ title: {{ display: true, text: 'median ticks', color: '#8b949e' }},
            ticks: {{ color: '#8b949e' }}, grid: {{ color: '#30363d' }} }},
      x: {{ ticks: {{ color: '#8b949e', autoSkip: false, maxRotation: 60, minRotation: 45 }},
            grid: {{ color: '#30363d' }} }} }} }});
}}

function arenaByTier(payload, valueFn) {{
  const tiers = TIER_ORDER;
  const classes = [...new Set(payload.arena.map(r => r.class))].sort();
  const datasets = classes.map(cls => ({{
    label: cls,
    data: tiers.map(tier => {{
      const subset = payload.arena.filter(r => r.class === cls && r.tier === tier);
      if (!subset.length) return null;
      return valueFn(subset);
    }}),
    backgroundColor: getColor(cls, 0.7),
  }}));
  return {{ labels: tiers, datasets }};
}}

function renderArena(payload) {{
  ['chart-survival', 'chart-rounds', 'chart-dmg-taken', 'chart-ecrits'].forEach(id => {{
    if (chartsByLabel[id]) chartsByLabel[id].destroy();
  }});

  const survival = arenaByTier(payload, subset => {{
    const totalAttempts = subset.reduce((s, r) => s + r.attempts, 0);
    const totalSurv = subset.reduce((s, r) => s + r.survived, 0);
    return totalAttempts ? Math.round(100 * totalSurv / totalAttempts) : null;
  }});
  chartsByLabel['chart-survival'] = setupChart(
    document.getElementById('chart-survival'), 'bar', survival,
    {{ scales: {{ y: {{ beginAtZero: true, max: 100,
      title: {{ display: true, text: 'survival %', color: '#8b949e' }} }} }} }});

  const rounds = arenaByTier(payload, subset => {{
    const totalRounds = subset.reduce((s, r) => s + r.avg_rounds * r.attempts, 0);
    const totalAttempts = subset.reduce((s, r) => s + r.attempts, 0);
    return totalAttempts ? +(totalRounds / totalAttempts).toFixed(1) : null;
  }});
  chartsByLabel['chart-rounds'] = setupChart(
    document.getElementById('chart-rounds'), 'bar', rounds,
    {{ scales: {{ y: {{ title: {{ display: true, text: 'avg rounds reached', color: '#8b949e' }} }} }} }});

  const dmg = arenaByTier(payload, subset => {{
    const totalDmg = subset.reduce((s, r) => s + r.avg_dmg_taken * r.attempts, 0);
    const totalAttempts = subset.reduce((s, r) => s + r.attempts, 0);
    return totalAttempts ? Math.round(totalDmg / totalAttempts) : null;
  }});
  chartsByLabel['chart-dmg-taken'] = setupChart(
    document.getElementById('chart-dmg-taken'), 'bar', dmg,
    {{ scales: {{ y: {{ title: {{ display: true, text: 'avg dmg taken / attempt', color: '#8b949e' }} }} }} }});

  const eCrits = arenaByTier(payload, subset => {{
    const totalCrits = subset.reduce((s, r) => s + r.avg_e_crits * r.attempts, 0);
    const totalAttempts = subset.reduce((s, r) => s + r.attempts, 0);
    return totalAttempts ? +(totalCrits / totalAttempts).toFixed(2) : null;
  }});
  chartsByLabel['chart-ecrits'] = setupChart(
    document.getElementById('chart-ecrits'), 'bar', eCrits,
    {{ scales: {{ y: {{ beginAtZero: true,
      title: {{ display: true, text: 'enemy crits / attempt', color: '#8b949e' }} }} }} }});

  const arenaCols = ['class', 'strategy', 'tier', 'attempts', 'survived', 'deaths',
                     'avg_rounds', 'avg_dmg_taken', 'avg_e_crits', 'avg_p_crits'];
  const arenaHeaders = ['Class', 'Strategy', 'Tier', 'N', 'Survived', 'Deaths',
                        'Avg Rounds', 'Avg Dmg Taken', 'Avg E-Crits', 'Avg P-Crits'];
  const rows = payload.arena.map(r => `<tr>${{arenaCols.map(c => {{
    let v = r[c];
    if (typeof v === 'number' && !Number.isInteger(v)) v = v.toFixed(2);
    return `<td>${{v}}</td>`;
  }}).join('')}}</tr>`).join('');
  document.getElementById('arena-table').innerHTML =
    `<table><thead><tr>${{arenaHeaders.map(h => `<th>${{h}}</th>`).join('')}}</tr></thead><tbody>${{rows}}</tbody></table>`;
}}

function renderCombat(payload) {{
  ['chart-boss-winrate', 'chart-mob-outcomes', 'chart-dmg-dealt-kind', 'chart-dmg-taken-kind'].forEach(id => {{
    if (chartsByLabel[id]) chartsByLabel[id].destroy();
  }});
  const boss = payload.boss || [];
  const mob = payload.mob || [];
  const damage = payload.damage || [];
  const classes = [...new Set([
    ...boss.map(r => r.class), ...mob.map(r => r.class), ...damage.map(r => r.class),
  ])].sort();

  const sumBy = (arr, cls, field) => arr.filter(r => r.class === cls)
    .reduce((s, r) => s + (r[field] || 0), 0);
  const mobEnc = cls => sumBy(mob, cls, 'encounters');
  const bossEnc = cls => sumBy(boss, cls, 'encounters');
  const avgDmg = (cls, totField, encFn) => {{
    const e = encFn(cls);
    return e ? +(sumBy(damage, cls, totField) / e).toFixed(1) : null;
  }};

  const bossWin = classes.map(cls => {{
    const enc = bossEnc(cls);
    const wins = sumBy(boss, cls, 'wins');
    return enc ? Math.round(100 * wins / enc) : null;
  }});
  chartsByLabel['chart-boss-winrate'] = setupChart(
    document.getElementById('chart-boss-winrate'), 'bar',
    {{ labels: classes, datasets: [{{ label: 'win %', data: bossWin,
       backgroundColor: classes.map(c => getColor(c, 0.7)) }}] }},
    {{ scales: {{ y: {{ beginAtZero: true, max: 100,
      title: {{ display: true, text: 'boss win %', color: '#8b949e' }} }} }} }});

  const mobOut = (field) => classes.map(cls => sumBy(mob, cls, field));
  chartsByLabel['chart-mob-outcomes'] = setupChart(
    document.getElementById('chart-mob-outcomes'), 'bar',
    {{ labels: classes, datasets: [
      {{ label: 'Kills', data: mobOut('kills'), backgroundColor: getColor('Ranger', 0.7) }},
      {{ label: 'Draws', data: mobOut('draws'), backgroundColor: getColor('Wizard', 0.7) }},
      {{ label: 'Deaths', data: mobOut('deaths'), backgroundColor: getColor('Warrior', 0.7) }},
    ] }},
    {{ scales: {{ x: {{ stacked: true }}, y: {{ stacked: true, beginAtZero: true,
      title: {{ display: true, text: 'mob encounters', color: '#8b949e' }} }} }} }});

  const dmgChart = (canvasId, mobTot, bossTot, ylabel) => {{
    chartsByLabel[canvasId] = setupChart(
      document.getElementById(canvasId), 'bar',
      {{ labels: classes, datasets: [
        {{ label: 'Mob', data: classes.map(c => avgDmg(c, mobTot, mobEnc)),
           backgroundColor: getColor('Rogue', 0.7) }},
        {{ label: 'Boss', data: classes.map(c => avgDmg(c, bossTot, bossEnc)),
           backgroundColor: getColor('Necromancer', 0.7) }},
      ] }},
      {{ scales: {{ y: {{ beginAtZero: true,
        title: {{ display: true, text: ylabel, color: '#8b949e' }} }} }} }});
  }};
  dmgChart('chart-dmg-dealt-kind', 'mob_dmg_dealt', 'boss_dmg_dealt', 'avg dmg dealt');
  dmgChart('chart-dmg-taken-kind', 'mob_dmg_taken', 'boss_dmg_taken', 'avg dmg taken');

  const tableHTML = (rows, cols, headers) => {{
    if (!rows.length) return '<div class=\"empty\">(no encounters recorded)</div>';
    const body = rows.map(r => `<tr>${{cols.map(c => {{
      let v = r[c];
      if (typeof v === 'number' && !Number.isInteger(v)) v = v.toFixed(1);
      return `<td>${{v}}</td>`;
    }}).join('')}}</tr>`).join('');
    return `<table><thead><tr>${{headers.map(h => `<th>${{h}}</th>`).join('')}}</tr></thead><tbody>${{body}}</tbody></table>`;
  }};
  const bossCols = ['class','strategy','enemy_name','encounters','wins','losses','flees','avg_dmg_dealt','avg_dmg_taken'];
  const bossHeaders = ['Class','Strategy','Boss','N','Wins','Losses','Flees','Avg Dmg Dealt','Avg Dmg Taken'];
  const mobCols = ['class','strategy','encounters','kills','draws','deaths','elites','avg_dmg_dealt','avg_dmg_taken'];
  const mobHeaders = ['Class','Strategy','N','Kills','Draws','Deaths','Elites','Avg Dmg Dealt','Avg Dmg Taken'];
  document.getElementById('boss-table').innerHTML = tableHTML(boss, bossCols, bossHeaders);
  document.getElementById('mob-table').innerHTML = tableHTML(mob, mobCols, mobHeaders);
}}

function renderLoadout(payload) {{
  const rows = payload.loadout || [];
  ['chart-loadout-power','chart-loadout-rarity','chart-loadout-invsize'].forEach(id => {{
    if (chartsByLabel[id]) {{ chartsByLabel[id].destroy(); delete chartsByLabel[id]; }}
  }});
  const sel = document.getElementById('loadout-run-select');
  const drill = document.getElementById('loadout-drilldown');

  // group flat rows by run_id
  const runs = {{}};
  rows.forEach(r => {{
    if (!runs[r.run_id]) runs[r.run_id] = {{
      run_id: r.run_id, class: r.class, race: r.race,
      strategy: r.strategy, seed: r.seed, equipped: [], inventory: [],
    }};
    (r.equipped ? runs[r.run_id].equipped : runs[r.run_id].inventory).push(r);
  }});
  const runIds = Object.keys(runs).sort((a, b) => a - b);

  // per-run drill-down + selector
  sel.innerHTML = '';
  if (!runIds.length) {{
    drill.innerHTML = '<div class=\"empty\">(no final-loadout data for this label)</div>';
  }} else {{
    runIds.forEach(rid => {{
      const r = runs[rid];
      sel.add(new Option(`${{r.class}} ${{r.race}} · ${{r.strategy}} · seed ${{r.seed}} (run ${{rid}})`, rid));
    }});
    const drawDrill = (rid) => {{
      const r = runs[rid];
      if (!r) {{ drill.innerHTML = ''; return; }}
      const slotRow = (slot) => {{
        const it = r.equipped.find(x => x.slot === slot);
        if (!it) return `<tr><td>${{slot}}</td><td colspan=\"4\" class=\"empty\">(none)</td></tr>`;
        return `<tr><td>${{slot}}</td><td>${{it.name}}</td><td>${{it.rarity}}</td><td>${{it.power}}</td><td>+${{it.enchant_level}}</td></tr>`;
      }};
      const eq = `<table><thead><tr><th>Slot</th><th>Name</th><th>Rarity</th><th>Power</th><th>Enchant</th></tr></thead><tbody>${{['Weapon','Armor','Ring'].map(slotRow).join('')}}</tbody></table>`;
      const inv = r.inventory.length
        ? `<table><thead><tr><th>Slot</th><th>Name</th><th>Rarity</th><th>Power</th><th>Enchant</th></tr></thead><tbody>${{r.inventory.map(it => `<tr><td>${{it.slot}}</td><td>${{it.name}}</td><td>${{it.rarity}}</td><td>${{it.power}}</td><td>+${{it.enchant_level}}</td></tr>`).join('')}}</tbody></table>`
        : '<div class=\"empty\">(empty inventory)</div>';
      drill.innerHTML = `<h2 style=\"margin:6px 0\">Equipped</h2>${{eq}}<h2 style=\"margin:14px 0 6px\">Inventory (${{r.inventory.length}})</h2>${{inv}}`;
    }};
    sel.onchange = () => drawDrill(sel.value);
    drawDrill(runIds[0]);
  }}

  // aggregates by class
  const classes = [...new Set(rows.map(r => r.class))].sort();
  const eqRows = rows.filter(r => r.equipped);

  const avgPower = classes.map(cls => {{
    const subset = eqRows.filter(r => r.class === cls);
    if (!subset.length) return null;
    return +(subset.reduce((s, r) => s + r.power + r.enchant_level, 0) / subset.length).toFixed(1);
  }});
  chartsByLabel['chart-loadout-power'] = setupChart(
    document.getElementById('chart-loadout-power'), 'bar',
    {{ labels: classes, datasets: [{{ label: 'avg effective power', data: avgPower,
       backgroundColor: classes.map(c => getColor(c, 0.7)) }}] }},
    {{ plugins: {{ legend: {{ display: false }} }}, scales: {{ y: {{ beginAtZero: true,
      title: {{ display: true, text: 'avg equipped power', color: '#8b949e' }} }} }} }});

  chartsByLabel['chart-loadout-rarity'] = setupChart(
    document.getElementById('chart-loadout-rarity'), 'bar',
    {{ labels: classes, datasets: RARITY_ORDER.map(rar => ({{
      label: rar,
      data: classes.map(cls => eqRows.filter(r => r.class === cls && r.rarity === rar).length),
      backgroundColor: RARITY_COLOR[rar] || '#8b949e',
    }})) }},
    {{ scales: {{ x: {{ stacked: true }}, y: {{ stacked: true, beginAtZero: true,
      title: {{ display: true, text: 'equipped items', color: '#8b949e' }} }} }} }});

  const invByRun = {{}};
  rows.forEach(r => {{ if (!r.equipped) invByRun[r.run_id] = (invByRun[r.run_id] || 0) + 1; }});
  const avgInv = classes.map(cls => {{
    const rids = runIds.filter(rid => runs[rid].class === cls);
    if (!rids.length) return null;
    return +(rids.reduce((s, rid) => s + (invByRun[rid] || 0), 0) / rids.length).toFixed(1);
  }});
  chartsByLabel['chart-loadout-invsize'] = setupChart(
    document.getElementById('chart-loadout-invsize'), 'bar',
    {{ labels: classes, datasets: [{{ label: 'avg inventory size', data: avgInv,
       backgroundColor: classes.map(c => getColor(c, 0.7)) }}] }},
    {{ plugins: {{ legend: {{ display: false }} }}, scales: {{ y: {{ beginAtZero: true,
      title: {{ display: true, text: 'avg items held', color: '#8b949e' }} }} }} }});

  const topTable = (slot, elId) => {{
    const counts = {{}};
    eqRows.filter(r => r.slot === slot).forEach(r => {{
      const k = r.name + '|' + r.rarity;
      counts[k] = (counts[k] || 0) + 1;
    }});
    const sorted = Object.entries(counts).sort((a, b) => b[1] - a[1]).slice(0, 10);
    const el = document.getElementById(elId);
    if (!sorted.length) {{ el.innerHTML = '<div class=\"empty\">(none equipped)</div>'; return; }}
    const body = sorted.map(([k, n]) => {{
      const parts = k.split('|');
      return `<tr><td>${{parts[0]}}</td><td>${{parts[1]}}</td><td>${{n}}</td></tr>`;
    }}).join('');
    el.innerHTML = `<table><thead><tr><th>Name</th><th>Rarity</th><th>Count</th></tr></thead><tbody>${{body}}</tbody></table>`;
  }};
  topTable('Weapon', 'loadout-top-weapon');
  topTable('Armor', 'loadout-top-armor');
  topTable('Ring', 'loadout-top-ring');
}}

function renderItems() {{
  const cat = DATA.catalog;
  const unavail = document.getElementById('items-unavailable');
  const content = document.getElementById('items-content');
  if (!cat || !cat.items || !cat.items.length) {{
    unavail.style.display = '';
    content.style.display = 'none';
    return;
  }}
  unavail.style.display = 'none';
  content.style.display = '';
  const items = cat.items;
  document.getElementById('items-total').textContent =
    `${{items.length}} items · ${{RARITY_ORDER.length}} rarities × ${{SLOT_ORDER.length}} slots`;
  const rarityColors = RARITY_ORDER.map(r => RARITY_COLOR[r] || '#8b949e');
  // Destroy any item charts from a prior render() (dropdown changes re-run
  // render()->renderItems(); recreating on a live canvas crashes Chart.js).
  ['chart-item-rarity','chart-item-rarity-slot','chart-item-power-rarity',
   'chart-item-power-hist','chart-item-slot','chart-item-price-rarity'].forEach(id => {{
    if (chartsByLabel[id]) {{ chartsByLabel[id].destroy(); delete chartsByLabel[id]; }}
  }});

  // 1. Items by rarity
  const byRarity = RARITY_ORDER.map(r => items.filter(i => i.rarity === r).length);
  chartsByLabel['chart-item-rarity'] = setupChart(
    document.getElementById('chart-item-rarity'), 'bar',
    {{ labels: RARITY_ORDER, datasets: [{{ label: 'items', data: byRarity,
       backgroundColor: rarityColors }}] }},
    {{ plugins: {{ legend: {{ display: false }} }}, scales: {{ y: {{ beginAtZero: true,
      title: {{ display: true, text: 'item count', color: '#8b949e' }} }} }} }});

  // 2. Rarity x slot (stacked)
  chartsByLabel['chart-item-rarity-slot'] = setupChart(
    document.getElementById('chart-item-rarity-slot'), 'bar',
    {{ labels: RARITY_ORDER, datasets: SLOT_ORDER.map((slot, idx) => ({{
      label: slot,
      data: RARITY_ORDER.map(r => items.filter(i => i.rarity === r && i.slot === slot).length),
      backgroundColor: getColor(['Warrior','Ranger','Wizard','Necromancer'][idx], 0.7),
    }})) }},
    {{ scales: {{ x: {{ stacked: true }}, y: {{ stacked: true, beginAtZero: true,
      title: {{ display: true, text: 'item count', color: '#8b949e' }} }} }} }});

  // 3. Power range by rarity (min/avg/max)
  const stat = (r, fn) => {{
    const subset = items.filter(i => i.rarity === r);
    if (!subset.length) return 0;
    return fn(subset);
  }};
  const powMin = RARITY_ORDER.map(r => stat(r, s => Math.min(...s.map(i => i.power_min))));
  const powMax = RARITY_ORDER.map(r => stat(r, s => Math.max(...s.map(i => i.power_max))));
  const powAvg = RARITY_ORDER.map(r => stat(r, s => +(s.reduce((a, i) => a + (i.power_min + i.power_max) / 2, 0) / s.length).toFixed(1)));
  chartsByLabel['chart-item-power-rarity'] = setupChart(
    document.getElementById('chart-item-power-rarity'), 'bar',
    {{ labels: RARITY_ORDER, datasets: [
      {{ label: 'min', data: powMin, backgroundColor: getColor('Wizard', 0.6) }},
      {{ label: 'avg', data: powAvg, backgroundColor: getColor('Ranger', 0.6) }},
      {{ label: 'max', data: powMax, backgroundColor: getColor('Warrior', 0.6) }},
    ] }},
    {{ scales: {{ y: {{ beginAtZero: true,
      title: {{ display: true, text: 'power', color: '#8b949e' }} }} }} }});

  // 4. Power histogram (midpoint of each item's range)
  const mids = items.map(i => Math.round((i.power_min + i.power_max) / 2));
  const maxMid = Math.max(...mids);
  const buckets = {{}};
  for (let p = 0; p <= maxMid; p++) buckets[p] = 0;
  mids.forEach(m => {{ buckets[m] = (buckets[m] || 0) + 1; }});
  const histLabels = Object.keys(buckets).map(Number).sort((a, b) => a - b);
  chartsByLabel['chart-item-power-hist'] = setupChart(
    document.getElementById('chart-item-power-hist'), 'bar',
    {{ labels: histLabels, datasets: [{{ label: 'items', data: histLabels.map(p => buckets[p]),
       backgroundColor: getColor('Rogue', 0.7) }}] }},
    {{ plugins: {{ legend: {{ display: false }} }}, scales: {{ x: {{
      title: {{ display: true, text: 'power (range midpoint)', color: '#8b949e' }} }},
      y: {{ beginAtZero: true, title: {{ display: true, text: 'item count', color: '#8b949e' }} }} }} }});

  // 5. Items by slot
  const bySlot = SLOT_ORDER.map(s => items.filter(i => i.slot === s).length);
  chartsByLabel['chart-item-slot'] = setupChart(
    document.getElementById('chart-item-slot'), 'bar',
    {{ labels: SLOT_ORDER, datasets: [{{ label: 'items', data: bySlot,
       backgroundColor: getColor('Ranger', 0.7) }}] }},
    {{ plugins: {{ legend: {{ display: false }} }}, scales: {{ y: {{ beginAtZero: true,
      title: {{ display: true, text: 'item count', color: '#8b949e' }} }} }} }});

  // 6. Price range by rarity
  const priceMin = RARITY_ORDER.map(r => stat(r, s => Math.min(...s.map(i => i.price_min))));
  const priceMax = RARITY_ORDER.map(r => stat(r, s => Math.max(...s.map(i => i.price_max))));
  chartsByLabel['chart-item-price-rarity'] = setupChart(
    document.getElementById('chart-item-price-rarity'), 'bar',
    {{ labels: RARITY_ORDER, datasets: [
      {{ label: 'min price', data: priceMin, backgroundColor: getColor('Wizard', 0.6) }},
      {{ label: 'max price', data: priceMax, backgroundColor: getColor('Warrior', 0.6) }},
    ] }},
    {{ scales: {{ y: {{ beginAtZero: true,
      title: {{ display: true, text: 'gold', color: '#8b949e' }} }} }} }});

  // 7. Weights + multipliers reference table
  const weights = cat.rarity_weights || {{}};
  const mults = cat.rarity_multipliers || {{}};
  const rows = RARITY_ORDER.map(r => {{
    const w = weights[r];
    const pct = (typeof w === 'number') ? (w * 100).toFixed(2) + '%' : '—';
    const m = (mults[r] !== undefined) ? mults[r] : '—';
    return `<tr><td>${{r}}</td><td>${{pct}}</td><td>${{m}}×</td></tr>`;
  }}).join('');
  document.getElementById('items-meta-table').innerHTML =
    `<table><thead><tr><th>Rarity</th><th>Drop chance</th><th>Multiplier</th></tr></thead><tbody>${{rows}}</tbody></table>`;
}}

function renderBestiary() {{
  const best = DATA.bestiary;
  const unavail = document.getElementById('bestiary-unavailable');
  const content = document.getElementById('bestiary-content');
  if (!best || !best.bosses || !best.bosses.length || !best.monsters || !best.monsters.length) {{
    unavail.style.display = '';
    content.style.display = 'none';
    return;
  }}
  unavail.style.display = 'none';
  content.style.display = '';
  const monsters = best.monsters;
  const bosses = best.bosses;
  const meta = best.meta || {{}};
  const tierOrder = meta.tier_order || ['Vermin','Bruiser','Hunter','Horror','BossAdjacent'];
  document.getElementById('bestiary-total').textContent =
    `${{monsters.length}} monsters · ${{tierOrder.length}} tiers · ${{bosses.length}} bosses`;

  ['chart-mob-count','chart-mob-hp','chart-mob-atk','chart-mob-xp',
   'chart-boss-hp','chart-boss-atk','chart-boss-xp','chart-boss-gold'].forEach(id => {{
    if (chartsByLabel[id]) {{ chartsByLabel[id].destroy(); delete chartsByLabel[id]; }}
  }});

  const tierColors = tierOrder.map((t, i) => getColor(['Ranger','Wizard','Rogue','Warrior','Necromancer'][i % 5], 0.7));
  const tierStat = (tier, field, fn) => {{
    const subset = monsters.filter(m => m.tier === tier);
    if (!subset.length) return 0;
    return fn(subset.map(m => m[field]));
  }};

  // 1. Monsters by tier
  chartsByLabel['chart-mob-count'] = setupChart(
    document.getElementById('chart-mob-count'), 'bar',
    {{ labels: tierOrder, datasets: [{{ label: 'monsters', data: tierOrder.map(t => monsters.filter(m => m.tier === t).length),
       backgroundColor: tierColors }}] }},
    {{ plugins: {{ legend: {{ display: false }} }}, scales: {{ y: {{ beginAtZero: true,
      title: {{ display: true, text: 'monster count', color: '#8b949e' }} }} }} }});

  // 2-4. HP / Attack / XP by tier (min/avg/max)
  const minmaxavg = (canvasId, field, ylabel) => {{
    const mn = tierOrder.map(t => tierStat(t, field, a => Math.min(...a)));
    const mx = tierOrder.map(t => tierStat(t, field, a => Math.max(...a)));
    const av = tierOrder.map(t => tierStat(t, field, a => +(a.reduce((s, v) => s + v, 0) / a.length).toFixed(1)));
    chartsByLabel[canvasId] = setupChart(
      document.getElementById(canvasId), 'bar',
      {{ labels: tierOrder, datasets: [
        {{ label: 'min', data: mn, backgroundColor: getColor('Wizard', 0.6) }},
        {{ label: 'avg', data: av, backgroundColor: getColor('Ranger', 0.6) }},
        {{ label: 'max', data: mx, backgroundColor: getColor('Warrior', 0.6) }},
      ] }},
      {{ scales: {{ y: {{ beginAtZero: true,
        title: {{ display: true, text: ylabel, color: '#8b949e' }} }} }} }});
  }};
  minmaxavg('chart-mob-hp', 'hp', 'HP');
  minmaxavg('chart-mob-atk', 'attack', 'attack');
  minmaxavg('chart-mob-xp', 'xp', 'XP');

  // 5. Tier -> danger gating table
  const td = meta.tier_danger || {{}};
  const dangerRows = Object.keys(td).sort().map(d =>
    `<tr><td>${{d}}</td><td>${{(td[d] || []).join(', ')}}</td></tr>`).join('');
  document.getElementById('mob-tier-danger-table').innerHTML =
    `<table><thead><tr><th>Danger</th><th>Spawnable tiers</th></tr></thead><tbody>${{dangerRows}}</tbody></table>`;

  // 6. Elite modifiers table
  const el = meta.elite || {{}};
  const eliteRows = [
    ['Attack (base ×)', el.attack_base_mult],
    ['Attack (+ per danger)', el.attack_per_danger],
    ['HP (×)', el.hp_mult],
    ['XP (×)', el.xp_mult],
  ].map(([k, v]) => `<tr><td>${{k}}</td><td>${{v === undefined ? '—' : v}}</td></tr>`).join('');
  document.getElementById('mob-elite-table').innerHTML =
    `<table><thead><tr><th>Modifier</th><th>Value</th></tr></thead><tbody>${{eliteRows}}</tbody></table>`;

  // 7-10. Per-boss bars
  const bossLabels = bosses.map(b => b.name);
  const bossBar = (canvasId, field, ylabel, cls) => {{
    chartsByLabel[canvasId] = setupChart(
      document.getElementById(canvasId), 'bar',
      {{ labels: bossLabels, datasets: [{{ label: ylabel, data: bosses.map(b => b[field]),
         backgroundColor: getColor(cls, 0.7) }}] }},
      {{ plugins: {{ legend: {{ display: false }} }}, scales: {{ y: {{ beginAtZero: true,
        title: {{ display: true, text: ylabel, color: '#8b949e' }} }} }} }});
  }};
  bossBar('chart-boss-hp', 'hp', 'HP', 'Warrior');
  bossBar('chart-boss-atk', 'attack', 'attack', 'Necromancer');
  bossBar('chart-boss-xp', 'xp_reward', 'XP reward', 'Ranger');
  bossBar('chart-boss-gold', 'gold_reward', 'gold reward', 'Wizard');

  // 11. Boss roster table
  const bossRows = bosses.map(b =>
    `<tr><td>${{b.name}}</td><td>${{b.hp}}</td><td>${{b.attack}}</td><td>${{b.xp_reward}}</td><td>${{b.gold_reward}}</td><td>${{b.dex_mod}}</td></tr>`).join('');
  document.getElementById('boss-roster-table').innerHTML =
    `<table><thead><tr><th>Boss</th><th>HP</th><th>Attack</th><th>XP</th><th>Gold</th><th>Dex</th></tr></thead><tbody>${{bossRows}}</tbody></table>`;

  // 12. Spawn rate note
  const rate = meta.boss_spawn_rate;
  if (typeof rate === 'number' && rate > 0) {{
    document.getElementById('boss-spawn-note').textContent =
      `Bosses spawn at ~1 in ${{Math.round(1 / rate)}} ticks (rate ${{rate}}). No warning.`;
  }}
}}

function renderAB(primary, compare) {{
  const root = document.getElementById('ab-content');
  if (!compare) {{
    root.innerHTML = '<div class=\"card empty\">Select a comparison tuning_label in the header dropdown to see A/B diff charts.</div>';
    return;
  }}
  root.innerHTML = `
    <div class=\"grid\">
      <div class=\"card\"><h2>Arena survival % — ${{primary.label}} vs ${{compare.label}}</h2>
        <canvas id=\"ab-survival\"></canvas></div>
      <div class=\"card\"><h2>Avg rounds reached</h2>
        <canvas id=\"ab-rounds\"></canvas></div>
      <div class=\"card\"><h2>Avg dmg taken</h2>
        <canvas id=\"ab-dmg\"></canvas></div>
      <div class=\"card\"><h2>Enemy crits per attempt</h2>
        <canvas id=\"ab-ecrits\"></canvas></div>
    </div>
  `;
  const tiers = TIER_ORDER;
  const aggByTier = (payload, valueFn) => tiers.map(tier => {{
    const subset = payload.arena.filter(r => r.tier === tier);
    if (!subset.length) return null;
    return valueFn(subset);
  }});
  const buildAB = (canvasId, valueFn, ylabel) => {{
    if (chartsByLabel[canvasId]) chartsByLabel[canvasId].destroy();
    chartsByLabel[canvasId] = setupChart(
      document.getElementById(canvasId), 'bar',
      {{ labels: tiers, datasets: [
        {{ label: primary.label, data: aggByTier(primary, valueFn),
           backgroundColor: 'rgba(88,166,255,0.7)' }},
        {{ label: compare.label, data: aggByTier(compare, valueFn),
           backgroundColor: 'rgba(248,81,73,0.7)' }},
      ] }},
      {{ scales: {{ y: {{ beginAtZero: true,
        title: {{ display: true, text: ylabel, color: '#8b949e' }} }} }} }});
  }};
  buildAB('ab-survival', subset => {{
    const a = subset.reduce((s, r) => s + r.attempts, 0);
    const sv = subset.reduce((s, r) => s + r.survived, 0);
    return a ? Math.round(100 * sv / a) : null;
  }}, 'survival %');
  buildAB('ab-rounds', subset => {{
    const tot = subset.reduce((s, r) => s + r.avg_rounds * r.attempts, 0);
    const a = subset.reduce((s, r) => s + r.attempts, 0);
    return a ? +(tot / a).toFixed(1) : null;
  }}, 'avg rounds');
  buildAB('ab-dmg', subset => {{
    const tot = subset.reduce((s, r) => s + r.avg_dmg_taken * r.attempts, 0);
    const a = subset.reduce((s, r) => s + r.attempts, 0);
    return a ? Math.round(tot / a) : null;
  }}, 'avg dmg taken');
  buildAB('ab-ecrits', subset => {{
    const tot = subset.reduce((s, r) => s + r.avg_e_crits * r.attempts, 0);
    const a = subset.reduce((s, r) => s + r.attempts, 0);
    return a ? +(tot / a).toFixed(2) : null;
  }}, 'enemy crits / attempt');
}}

function currentPayload(label) {{
  return DATA.labels[label];
}}

function render(primaryLabel, compareLabel) {{
  renderItems();
  renderBestiary();
  const primary = primaryLabel ? currentPayload(primaryLabel) : null;
  if (!primary) {{
    document.getElementById('meta').textContent =
      `no sim data · generated ${{DATA.generated_at}}`;
    return;
  }}
  document.getElementById('meta').textContent =
    `${{primary.run_count}} runs · generated ${{DATA.generated_at}}`;
  renderOverview(primary);
  renderLifetimeTable(primary);
  renderProgression(primary);
  renderArena(primary);
  renderCombat(primary);
  renderLoadout(primary);
  renderAB(primary, compareLabel ? currentPayload(compareLabel) : null);
}}

function init() {{
  const primarySel = document.getElementById('primary-select');
  const compareSel = document.getElementById('compare-select');
  for (const lbl of DATA.tuning_labels) {{
    primarySel.add(new Option(lbl, lbl));
    compareSel.add(new Option(lbl, lbl));
  }}
  primarySel.value = DATA.primary_label;
  primarySel.addEventListener('change', () => render(primarySel.value, compareSel.value || null));
  compareSel.addEventListener('change', () => render(primarySel.value, compareSel.value || null));
  document.querySelectorAll('.tab').forEach(t => {{
    t.addEventListener('click', () => {{
      document.querySelectorAll('.tab').forEach(x => x.classList.remove('active'));
      document.querySelectorAll('.tab-content').forEach(x => x.classList.remove('active'));
      t.classList.add('active');
      document.getElementById('tab-' + t.dataset.tab).classList.add('active');
    }});
  }});
  render(DATA.primary_label || null, null);
}}
init();
</script>
</body>
</html>"""


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--db", default=str(db.DEFAULT_DB_PATH))
    parser.add_argument("--output", default=None,
                        help="output HTML path (default: dashboard.html next to db)")
    parser.add_argument("--primary", default=None,
                        help="primary tuning_label (default: most recent)")
    args = parser.parse_args()

    conn = db.open_db(Path(args.db))
    labels = list_tuning_labels(conn)
    catalog = load_catalog()
    bestiary = load_bestiary()

    if not labels:
        # No sim data: still emit a catalog-only dashboard (the Items tab is
        # independent of runs.db). Warn but do not fail.
        if catalog is None and bestiary is None:
            print("No tuning_labels in database and no item catalog / bestiary "
                  "(build sq: cargo build --bin sq) — nothing to render",
                  file=sys.stderr)
            return 1
        print("No tuning_labels in database — rendering item catalog only "
              "(run sims to populate the other tabs)", file=sys.stderr)
        primary_label = None
        label_payloads = {}
    else:
        primary_label = args.primary or labels[-1]
        if primary_label not in labels:
            print(f"tuning_label '{primary_label}' not found. Available: {labels}",
                  file=sys.stderr)
            return 1
        label_payloads = {lbl: build_label_payload(conn, lbl) for lbl in labels}

    data = {
        "tuning_labels": labels,
        "primary_label": primary_label,
        "generated_at": __import__("datetime").datetime.now().isoformat(timespec="seconds"),
        "labels": label_payloads,
        "catalog": catalog,
        "bestiary": bestiary,
    }

    output_path = (
        Path(args.output) if args.output
        else Path(args.db).with_name("dashboard.html")
    )
    output_path.write_text(render_html(data))
    print(f"Dashboard written to {output_path}")
    print(f"Open in browser: file://{output_path.resolve()}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

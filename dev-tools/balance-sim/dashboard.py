#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sqlite3
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from simulator import db

TIER_ORDER = ["Pit", "Gauntlet", "Colosseum", "Abyssal", "Godslayer"]


def list_tuning_labels(conn: sqlite3.Connection) -> list[str]:
    return [r[0] for r in conn.execute(
        "SELECT DISTINCT tuning_label FROM run ORDER BY tuning_label"
    )]


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


def build_label_payload(conn: sqlite3.Connection, label: str) -> dict:
    run_count = conn.execute(
        "SELECT COUNT(*) FROM run WHERE tuning_label = ?", (label,)
    ).fetchone()[0]
    return {
        "label": label,
        "run_count": run_count,
        "lifetime": lifetime_summary(conn, label),
        "arena": arena_summary(conn, label),
        "progression": progression_curve(conn, label),
        "time_to_l25": time_to_level(conn, label, 25),
        "time_to_l40": time_to_level(conn, label, 40),
        "time_to_l60": time_to_level(conn, label, 60),
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
  const primary = currentPayload(primaryLabel);
  document.getElementById('meta').textContent =
    `${{primary.run_count}} runs · generated ${{DATA.generated_at}}`;
  renderOverview(primary);
  renderLifetimeTable(primary);
  renderProgression(primary);
  renderArena(primary);
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
  render(DATA.primary_label, null);
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
    if not labels:
        print("No tuning_labels in database — run sims first", file=sys.stderr)
        return 1

    primary_label = args.primary or labels[-1]
    if primary_label not in labels:
        print(f"tuning_label '{primary_label}' not found. Available: {labels}",
              file=sys.stderr)
        return 1

    data = {
        "tuning_labels": labels,
        "primary_label": primary_label,
        "generated_at": __import__("datetime").datetime.now().isoformat(timespec="seconds"),
        "labels": {lbl: build_label_payload(conn, lbl) for lbl in labels},
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

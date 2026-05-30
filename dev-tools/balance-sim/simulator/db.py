"""SQLite persistence layer for balance-sim.

Single-writer, multi-reader. Each SimPlayer process opens its own connection
via WAL mode so parallel runs can write concurrently without lock contention.
"""
from __future__ import annotations

import json
import sqlite3
import time
from pathlib import Path
from typing import Any

BALANCE_SIM_DIR = Path(__file__).resolve().parents[1]
DEFAULT_DB_PATH = BALANCE_SIM_DIR / "runs.db"
SCHEMA_PATH = BALANCE_SIM_DIR / "schema.sql"


def open_db(db_path: Path = DEFAULT_DB_PATH) -> sqlite3.Connection:
    db_path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(str(db_path), timeout=60.0)
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("PRAGMA synchronous=NORMAL")
    conn.execute("PRAGMA busy_timeout=60000")
    conn.execute("PRAGMA foreign_keys=ON")
    conn.row_factory = sqlite3.Row
    return conn


def merge_dbs(target_path: Path, source_paths: list[Path]) -> int:
    target = open_db(target_path)
    init_schema(target)
    merged_runs = 0
    for src in source_paths:
        if not src.exists():
            continue
        target.execute(f"ATTACH DATABASE '{src}' AS src")
        try:
            id_map = {}
            for row in target.execute(
                "SELECT id, seed, class, race, strategy, tuning_label, target_level, "
                "max_ticks, started_at, ended_at, final_level, final_xp, final_gold, "
                "final_kills, final_deaths, final_prestige, final_max_hp, "
                "final_attack_power, final_defense, total_ticks, ended_reason "
                "FROM src.run"
            ).fetchall():
                cur = target.execute(
                    "INSERT INTO run (seed, class, race, strategy, tuning_label, "
                    "target_level, max_ticks, started_at, ended_at, final_level, "
                    "final_xp, final_gold, final_kills, final_deaths, final_prestige, "
                    "final_max_hp, final_attack_power, final_defense, total_ticks, ended_reason) "
                    "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    (row["seed"], row["class"], row["race"], row["strategy"],
                     row["tuning_label"], row["target_level"], row["max_ticks"],
                     row["started_at"], row["ended_at"], row["final_level"],
                     row["final_xp"], row["final_gold"], row["final_kills"],
                     row["final_deaths"], row["final_prestige"], row["final_max_hp"],
                     row["final_attack_power"], row["final_defense"],
                     row["total_ticks"], row["ended_reason"]),
                )
                id_map[row["id"]] = cur.lastrowid
            for src_id, new_id in id_map.items():
                target.execute(
                    "INSERT INTO tick_snapshot SELECT ?, tick_no, level, xp, hp, max_hp, "
                    "gold, kills, deaths, strength, dexterity, intelligence, attack_power, "
                    "defense, inventory_count, weapon_power, armor_power, ring_power, "
                    "weapon_rarity, armor_rarity, ring_rarity FROM src.tick_snapshot WHERE run_id = ?",
                    (new_id, src_id),
                )
                target.execute(
                    "INSERT INTO action_log (run_id, tick_no, action, details, outcome) "
                    "SELECT ?, tick_no, action, details, outcome FROM src.action_log WHERE run_id = ?",
                    (new_id, src_id),
                )
                target.execute(
                    "INSERT INTO arena_attempt (run_id, tick_no, character_level, tier, "
                    "tier_index, entry_fee, rounds_attempted, rounds_won, outcome, "
                    "gold_earned, xp_earned, dmg_dealt, dmg_taken, enemy_crits, player_crits, "
                    "player_swings, enemy_swings) "
                    "SELECT ?, tick_no, character_level, tier, tier_index, entry_fee, "
                    "rounds_attempted, rounds_won, outcome, gold_earned, xp_earned, "
                    "dmg_dealt, dmg_taken, enemy_crits, player_crits, player_swings, enemy_swings "
                    "FROM src.arena_attempt WHERE run_id = ?",
                    (new_id, src_id),
                )
                target.execute(
                    "INSERT INTO item_event (run_id, tick_no, event_type, item_name, rarity, "
                    "slot, power, enchant_level, gold_cost, was_equipped) "
                    "SELECT ?, tick_no, event_type, item_name, rarity, slot, power, "
                    "enchant_level, gold_cost, was_equipped FROM src.item_event WHERE run_id = ?",
                    (new_id, src_id),
                )
                target.execute(
                    "INSERT INTO sq_invocation (run_id, tick_no, argv, cwd, exit_code, "
                    "stdout, stderr, duration_ms) "
                    "SELECT ?, tick_no, argv, cwd, exit_code, stdout, stderr, duration_ms "
                    "FROM src.sq_invocation WHERE run_id = ?",
                    (new_id, src_id),
                )
                target.execute(
                    "INSERT INTO overworld_encounter (run_id, tick_no, character_level, kind, "
                    "enemy_name, elite, dmg_dealt, dmg_taken, outcome, xp_earned, gold_earned) "
                    "SELECT ?, tick_no, character_level, kind, enemy_name, elite, dmg_dealt, "
                    "dmg_taken, outcome, xp_earned, gold_earned FROM src.overworld_encounter WHERE run_id = ?",
                    (new_id, src_id),
                )
                target.execute(
                    "INSERT INTO final_item (run_id, slot, equipped, name, rarity, "
                    "power, enchant_level) "
                    "SELECT ?, slot, equipped, name, rarity, power, enchant_level "
                    "FROM src.final_item WHERE run_id = ?",
                    (new_id, src_id),
                )
            merged_runs += len(id_map)
            target.commit()
        finally:
            target.execute("DETACH DATABASE src")
    target.commit()
    target.close()
    return merged_runs


def init_schema(conn: sqlite3.Connection) -> None:
    ddl = SCHEMA_PATH.read_text()
    conn.executescript(ddl)
    conn.commit()


def insert_run(
    conn: sqlite3.Connection,
    *,
    seed: int,
    cls: str,
    race: str,
    strategy: str,
    tuning_label: str,
    target_level: int,
    max_ticks: int,
) -> int:
    cur = conn.execute(
        """
        INSERT INTO run (seed, class, race, strategy, tuning_label,
                         target_level, max_ticks, started_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (seed, cls, race, strategy, tuning_label,
         target_level, max_ticks, time.time()),
    )
    conn.commit()
    return cur.lastrowid


def finalize_run(
    conn: sqlite3.Connection,
    run_id: int,
    *,
    final_state: dict,
    total_ticks: int,
    ended_reason: str,
) -> None:
    conn.execute(
        """
        UPDATE run
           SET ended_at = ?,
               final_level = ?, final_xp = ?, final_gold = ?,
               final_kills = ?, final_deaths = ?, final_prestige = ?,
               final_max_hp = ?, final_attack_power = ?, final_defense = ?,
               total_ticks = ?, ended_reason = ?
         WHERE id = ?
        """,
        (
            time.time(),
            final_state["level"], final_state["xp"], final_state["gold"],
            final_state["kills"], final_state["deaths"],
            final_state.get("prestige", 0),
            final_state["max_hp"],
            final_state["attack_power"], final_state["defense"],
            total_ticks, ended_reason, run_id,
        ),
    )
    conn.commit()


def insert_tick_snapshot(
    conn: sqlite3.Connection, run_id: int, tick_no: int, state: dict
) -> None:
    conn.execute(
        """
        INSERT OR REPLACE INTO tick_snapshot
        (run_id, tick_no, level, xp, hp, max_hp, gold, kills, deaths,
         strength, dexterity, intelligence, attack_power, defense,
         inventory_count, weapon_power, armor_power, ring_power,
         weapon_rarity, armor_rarity, ring_rarity)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            run_id, tick_no,
            state["level"], state["xp"], state["hp"], state["max_hp"],
            state["gold"], state["kills"], state["deaths"],
            state["strength"], state["dexterity"], state["intelligence"],
            state["attack_power"], state["defense"],
            state["inventory_count"],
            state.get("weapon_power"), state.get("armor_power"), state.get("ring_power"),
            state.get("weapon_rarity"), state.get("armor_rarity"), state.get("ring_rarity"),
        ),
    )


def insert_action(
    conn: sqlite3.Connection,
    run_id: int,
    tick_no: int,
    action: str,
    *,
    details: dict | None = None,
    outcome: str | None = None,
) -> None:
    conn.execute(
        "INSERT INTO action_log (run_id, tick_no, action, details, outcome) "
        "VALUES (?, ?, ?, ?, ?)",
        (run_id, tick_no, action,
         json.dumps(details) if details else None, outcome),
    )


def insert_arena_attempt(
    conn: sqlite3.Connection, run_id: int, tick_no: int, attempt: dict
) -> None:
    conn.execute(
        """
        INSERT INTO arena_attempt
        (run_id, tick_no, character_level, tier, tier_index, entry_fee,
         rounds_attempted, rounds_won, outcome, gold_earned, xp_earned,
         dmg_dealt, dmg_taken, enemy_crits, player_crits,
         player_swings, enemy_swings)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            run_id, tick_no, attempt["character_level"],
            attempt["tier"], attempt["tier_index"], attempt["entry_fee"],
            attempt["rounds_attempted"], attempt["rounds_won"],
            attempt["outcome"], attempt["gold_earned"], attempt["xp_earned"],
            attempt["dmg_dealt"], attempt["dmg_taken"],
            attempt["enemy_crits"], attempt["player_crits"],
            attempt["player_swings"], attempt["enemy_swings"],
        ),
    )


def insert_item_event(
    conn: sqlite3.Connection, run_id: int, tick_no: int, event: dict
) -> None:
    conn.execute(
        """
        INSERT INTO item_event
        (run_id, tick_no, event_type, item_name, rarity, slot,
         power, enchant_level, gold_cost, was_equipped)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (run_id, tick_no, event["event_type"], event["item_name"],
         event["rarity"], event["slot"], event["power"],
         event["enchant_level"], event.get("gold_cost"),
         1 if event.get("was_equipped") else 0),
    )


def insert_sq_invocation(
    conn: sqlite3.Connection,
    run_id: int,
    tick_no: int,
    *,
    argv: str,
    cwd: str | None,
    exit_code: int | None,
    stdout: str | None,
    stderr: str | None,
    duration_ms: int | None,
) -> None:
    conn.execute(
        """
        INSERT INTO sq_invocation
        (run_id, tick_no, argv, cwd, exit_code, stdout, stderr, duration_ms)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (run_id, tick_no, argv, cwd, exit_code, stdout, stderr, duration_ms),
    )


def insert_overworld_encounter(
    conn: sqlite3.Connection, run_id: int, tick_no: int, enc: dict
) -> None:
    conn.execute(
        """
        INSERT INTO overworld_encounter
        (run_id, tick_no, character_level, kind, enemy_name, elite,
         dmg_dealt, dmg_taken, outcome, xp_earned, gold_earned)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (run_id, tick_no, enc["character_level"], enc["kind"],
         enc["enemy_name"], 1 if enc.get("elite") else 0,
         enc["dmg_dealt"], enc["dmg_taken"], enc["outcome"],
         enc["xp_earned"], enc["gold_earned"]),
    )


def insert_final_items(
    conn: sqlite3.Connection, run_id: int, items: list[dict]
) -> None:
    """Persist a run's END-STATE items (see player.extract_final_items).
    One INSERT per item; the equipped flag is supplied by the caller."""
    for it in items:
        conn.execute(
            """
            INSERT INTO final_item
            (run_id, slot, equipped, name, rarity, power, enchant_level)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            """,
            (run_id, it["slot"], 1 if it.get("equipped") else 0,
             it["name"], it["rarity"], it["power"], it["enchant_level"]),
        )


def commit(conn: sqlite3.Connection) -> None:
    conn.commit()


def close(conn: sqlite3.Connection) -> None:
    try:
        conn.commit()
    except Exception:
        pass
    conn.close()

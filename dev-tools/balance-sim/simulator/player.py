from __future__ import annotations

import json
import os
import random
import shutil
import sqlite3
import tempfile
import time
from pathlib import Path
from typing import Any

from . import db
from . import driver
from . import strategies

CLASS_BASE = {
    "Warrior":     (16, 8, 6),
    "Wizard":      (6,  8, 16),
    "Rogue":       (8, 16, 6),
    "Ranger":      (10, 14, 6),
    "Necromancer": (6,  6, 18),
}
RACE_BONUS = {
    "Human":  (1, 1, 1),
    "Elf":    (0, 2, 2),
    "Dwarf":  (3, 0, 1),
    "Orc":    (3, 1, -1),
    "Goblin": (0, 3, 1),
}


def _home_parent() -> str | None:
    candidate = Path(os.environ.get("SQ_SIM_HOME_PARENT", "/sim-home"))
    if candidate.is_dir():
        return str(candidate)
    return None


def starter_save(cls: str, race: str, seed: int) -> dict[str, Any]:
    bs, bd, bi = CLASS_BASE[cls]
    rs, rd, ri = RACE_BONUS[race]
    strength = bs + rs
    dexterity = bd + rd
    intelligence = bi + ri
    max_hp = 20 + strength * 2
    return {
        "character": {
            "name": f"Sim-{seed}",
            "class": cls, "race": race,
            "level": 1, "xp": 0, "xp_to_next": 25,
            "hp": max_hp, "max_hp": max_hp,
            "strength": strength, "dexterity": dexterity, "intelligence": intelligence,
            "gold": 10, "kills": 0, "deaths": 0, "commands_run": 0,
            "weapon": None, "armor": None, "ring": None,
            "inventory": [],
            "title": "Terminal Novice",
            "prestige": 0, "subclass": None, "total_prestiges": 0,
            "tournament_wins": 0, "best_tournament_round": 0,
        },
        "journal": [],
        "created_at": "2026-05-26T00:00:00Z",
        "last_tick": "2026-05-26T00:00:00Z",
        "latest_version": "1.23.0", "last_version_check": "2026-05-26T00:00:00Z",
        "last_sage_shown": "2026-05-26T00:00:00Z",
        "last_announced_version": "1.23.0",
        "shop_items": [], "shop_refreshed": "2026-05-26T00:00:00Z",
        "last_heal_at": "2026-05-26T00:00:00Z",
        "active_boss": None, "permadeath": False,
    }


def extract_final_items(save: dict[str, Any]) -> list[dict[str, Any]]:
    """Flatten a save's END-STATE gear + held inventory into final_item rows.

    Returns one dict per item with keys slot/equipped/name/rarity/power/
    enchant_level. The three equipped slots (weapon/armor/ring) get
    equipped=1 (null gear skipped); every inventory item gets equipped=0
    keeping its own slot (potions included). Null-safe against a missing
    character, missing gear, or missing item fields.
    """
    character = (save or {}).get("character") or {}

    def _row(item: dict[str, Any], equipped: int) -> dict[str, Any]:
        return {
            "slot": item.get("slot") or "Unknown",
            "equipped": equipped,
            "name": item.get("name") or "Unknown",
            "rarity": item.get("rarity") or "Common",
            "power": int(item.get("power") or 0),
            "enchant_level": int(item.get("enchant_level") or 0),
        }

    items: list[dict[str, Any]] = []
    for slot_key in ("weapon", "armor", "ring"):
        gear = character.get(slot_key)
        if gear:
            items.append(_row(gear, 1))
    for inv_item in character.get("inventory") or []:
        if inv_item:
            items.append(_row(inv_item, 0))
    return items


class SimPlayer:
    def __init__(
        self,
        cls: str, race: str, strategy_name: str, seed: int,
        tuning_label: str, db_path: Path,
        target_level: int = 100, max_ticks: int = 8000,
        snapshot_every: int = 50,
        min_arena_tier_index: int = 1,
    ):
        self.cls = cls
        self.race = race
        self.strategy = strategies.STRATEGIES[strategy_name](
            min_arena_tier_index=min_arena_tier_index,
        )
        self.seed = seed
        self.tuning_label = tuning_label
        self.rng = random.Random(seed)
        self.target_level = target_level
        self.max_ticks = max_ticks
        self.snapshot_every = snapshot_every
        self.home = Path(tempfile.mkdtemp(prefix=f"sq-bench-{seed}-", dir=_home_parent()))
        self.conn = db.open_db(db_path)
        db.init_schema(self.conn)
        self.run_id = db.insert_run(
            self.conn, seed=seed, cls=cls, race=race,
            strategy=strategy_name, tuning_label=tuning_label,
            target_level=target_level, max_ticks=max_ticks,
        )
        self.tick_no = 0
        self._enchant_failure_count = 0
        driver.set_invocation_recorder(self._record_sq_invocation)
        driver.init_save(self.home, starter_save(cls, race, seed))

    def cleanup(self) -> None:
        driver.set_invocation_recorder(None)
        try:
            db.close(self.conn)
        finally:
            shutil.rmtree(self.home, ignore_errors=True)

    def _record_sq_invocation(self, argv: list[str], cwd: str, exit_code: int,
                              stdout: str, stderr: str, duration_ms: int) -> None:
        db.insert_sq_invocation(
            self.conn, self.run_id, self.tick_no,
            argv=" ".join(argv), cwd=cwd, exit_code=exit_code,
            stdout=stdout, stderr=stderr, duration_ms=duration_ms,
        )

    def _read(self) -> tuple[dict[str, Any], dict[str, Any]]:
        save = driver.read_save(self.home)
        return save, driver.derive_state(save)

    def _snapshot(self, state: dict[str, Any]) -> None:
        db.insert_tick_snapshot(self.conn, self.run_id, self.tick_no, state)

    def _log(self, action: str, details: dict[str, Any] | None = None,
             outcome: str | None = None) -> None:
        db.insert_action(self.conn, self.run_id, self.tick_no, action,
                         details=details, outcome=outcome)

    def _execute_tick(self, decision: strategies.Decision) -> None:
        kind = decision.payload.get("cmd_kind", "craft")
        danger = decision.payload.get("danger", 2)
        cwd = driver.cwd_for_danger(danger, self.home)
        if kind == "craft":
            cmd = self.rng.choice(driver.CRAFT_CMDS)
            exit_code = 0
        elif kind == "fail":
            cmd = self.rng.choice(driver.FAIL_CMDS)
            exit_code = 1
        else:
            cmd = self.rng.choice(driver.BENIGN_CMDS)
            exit_code = 0
        rc, stderr = driver.cmd_tick(self.home, cmd, cwd, exit_code)
        if rc != 0:
            stderr_summary = stderr.strip() or "<empty stderr>"
            raise RuntimeError(
                f"sq tick failed rc={rc} cmd={cmd!r} cwd={cwd!r}: {stderr_summary}"
            )
        encounters = driver.parse_encounter_lines(stderr)
        if encounters:
            level = driver.read_save(self.home)["character"]["level"]
            for enc in encounters:
                enc["character_level"] = level
                db.insert_overworld_encounter(self.conn, self.run_id, self.tick_no, enc)
        self._log("tick", {"cmd": cmd, "cwd": cwd, "danger": danger,
                            "exit_code": exit_code})

    def _execute_equip(self, decision: strategies.Decision) -> None:
        item = decision.payload["item"]
        save = driver.read_save(self.home)
        c = save["character"]
        slot = item["slot"].lower()
        old = c.get(slot)
        inv = c["inventory"]
        new = None
        for idx, existing in enumerate(inv):
            if (existing["name"] == item["name"]
                    and existing["power"] == item["power"]
                    and existing["enchant_level"] == item["enchant_level"]):
                new = inv.pop(idx)
                break
        if new is None:
            self._log("equip", {"item": item["name"]}, outcome="not_found")
            return
        c[slot] = new
        if old is not None and len(inv) < 20:
            inv.append(old)
        driver.write_save(self.home, save)
        db.insert_item_event(self.conn, self.run_id, self.tick_no, {
            "event_type": "equip", "item_name": item["name"],
            "rarity": item["rarity"], "slot": slot,
            "power": item["power"], "enchant_level": item["enchant_level"],
            "gold_cost": None, "was_equipped": True,
        })
        self._log("equip", {"item": item["name"], "slot": slot}, outcome="ok")

    def _execute_enchant(self, decision: strategies.Decision) -> None:
        slot = decision.payload["slot"]
        item = decision.payload["item"]
        before_level = item["enchant_level"]
        rc, stderr = driver.cmd_enchant(self.home, item["name"])
        save_after = driver.read_save(self.home)
        new_item = save_after["character"].get(slot)
        actual_level = new_item["enchant_level"] if new_item else before_level
        succeeded = actual_level > before_level
        outcome = "ok" if succeeded else ("not_in_home" if "home directory" in stderr.lower() else f"no_change_rc{rc}")
        db.insert_item_event(self.conn, self.run_id, self.tick_no, {
            "event_type": "enchant", "item_name": item["name"],
            "rarity": item["rarity"], "slot": slot,
            "power": item["power"], "enchant_level": actual_level,
            "gold_cost": decision.payload.get("cost"),
            "was_equipped": True,
        })
        self._log("enchant", {"item": item["name"], "before": before_level, "after": actual_level}, outcome=outcome)
        if not succeeded:
            self._enchant_failure_count = getattr(self, "_enchant_failure_count", 0) + 1
        else:
            self._enchant_failure_count = 0

    def _execute_arena(self, decision: strategies.Decision) -> None:
        tier_idx = decision.payload["tier_index"]
        tier_name = decision.payload["tier_name"]
        max_rounds = {1: 5, 2: 10, 3: 15, 4: 25, 5: 50}[tier_idx]
        save_before = driver.read_save(self.home)
        level = save_before["character"]["level"]
        raw = driver.cmd_arena(self.home, tier_idx, max_rounds, timeout_s=max(60, max_rounds * 6))
        attempt = driver.parse_arena_output(raw, level, tier_name, tier_idx)
        save_after = driver.read_save(self.home)
        attempt["gold_earned"] = max(0, save_after["character"]["gold"] - save_before["character"]["gold"] + attempt["entry_fee"])
        attempt["xp_earned"] = max(0, save_after["character"]["xp"] - save_before["character"]["xp"])
        db.insert_arena_attempt(self.conn, self.run_id, self.tick_no, attempt)
        self._log("arena", {"tier": tier_name, "outcome": attempt["outcome"],
                             "rounds_won": attempt["rounds_won"]},
                  outcome=attempt["outcome"])

    def run(self) -> dict[str, Any]:
        ended_reason = "max_ticks"
        try:
            while self.tick_no < self.max_ticks:
                self.tick_no += 1
                save, state = self._read()
                decision = self.strategy.decide(state, save, self.rng)
                if getattr(self, "_enchant_failure_count", 0) >= 3 and decision.action == "enchant":
                    decision = strategies.Decision("tick", {"cmd_kind": "craft", "danger": 2})
                if decision.action == "tick":
                    self._execute_tick(decision)
                elif decision.action == "equip":
                    self._execute_equip(decision)
                elif decision.action == "enchant":
                    self._execute_enchant(decision)
                elif decision.action == "arena":
                    self._execute_arena(decision)
                else:
                    self._log("noop", {})
                if self.tick_no % self.snapshot_every == 0:
                    _, snapshot_state = self._read()
                    self._snapshot(snapshot_state)
                    db.commit(self.conn)
                if state["level"] >= self.target_level:
                    ended_reason = "target_reached"
                    break
        except Exception as e:
            ended_reason = f"error: {type(e).__name__}: {e}"
        save, final_state = self._read()
        self._snapshot(final_state)
        db.insert_final_items(self.conn, self.run_id, extract_final_items(save))
        db.finalize_run(self.conn, self.run_id,
                        final_state=final_state,
                        total_ticks=self.tick_no,
                        ended_reason=ended_reason)
        return {
            "run_id": self.run_id, "ended_reason": ended_reason,
            "final_state": final_state, "ticks": self.tick_no,
        }


def simulate_one(
    cls: str, race: str, strategy_name: str, seed: int,
    tuning_label: str, db_path: Path, **kwargs: Any,
) -> dict[str, Any]:
    p = SimPlayer(cls, race, strategy_name, seed, tuning_label, db_path, **kwargs)
    try:
        return p.run()
    finally:
        p.cleanup()


def auto_max_ticks(target_level: int) -> int:
    if target_level <= 25:  return 1500
    if target_level <= 40:  return 3000
    if target_level <= 60:  return 6000
    if target_level <= 100: return 18000
    if target_level <= 130: return 45000
    return 90000

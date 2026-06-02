from __future__ import annotations

import json
import importlib
import os
import subprocess
import sys
import tempfile
import threading
import unittest
from pathlib import Path
from typing import Any, cast
from unittest import mock

BALANCE_SIM_DIR = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(BALANCE_SIM_DIR))

runner = importlib.import_module("runner")  # noqa: E402
db = cast(Any, importlib.import_module("simulator.db"))  # noqa: E402


def _snapshot_state(level: int) -> dict[str, Any]:
    return {
        "level": level,
        "xp": level * 10,
        "hp": 20 + level,
        "max_hp": 30 + level,
        "gold": 100 + level,
        "kills": level,
        "deaths": 0,
        "strength": 10 + level,
        "dexterity": 8 + level,
        "intelligence": 6 + level,
        "attack_power": 12 + level,
        "defense": 7 + level,
        "inventory_count": 1,
        "weapon_power": 3,
        "armor_power": 2,
        "ring_power": 1,
        "weapon_rarity": "Common",
        "armor_rarity": "Common",
        "ring_rarity": "Common",
    }


class TestSharedRunsDb(unittest.TestCase):
    def test_two_concurrent_writers_get_distinct_run_ids_and_intact_child_rows(self):
        with tempfile.TemporaryDirectory(prefix="shared-runs-db-") as tmp:
            db_path = Path(tmp) / "runs.db"
            host_conn = db.open_db(db_path)
            db.init_schema(host_conn)
            db.close(host_conn)

            barrier = threading.Barrier(2)
            run_ids: list[int] = []
            errors: list[BaseException] = []
            lock = threading.Lock()

            def writer(seed: int, level: int) -> None:
                conn = db.open_db(db_path)
                try:
                    barrier.wait(timeout=5)
                    run_id = db.insert_run(
                        conn,
                        seed=seed,
                        cls="Warrior" if seed == 1 else "Wizard",
                        race="Human",
                        strategy="greedy",
                        tuning_label="wal-concurrency",
                        target_level=15,
                        max_ticks=100,
                    )
                    db.insert_tick_snapshot(conn, run_id, 1, _snapshot_state(level))
                    db.insert_action(conn, run_id, 1, "tick", details={"seed": seed}, outcome="ok")
                    db.insert_overworld_encounter(conn, run_id, 1, {
                        "character_level": level,
                        "kind": "mob",
                        "enemy_name": "test goblin",
                        "elite": False,
                        "dmg_dealt": 12,
                        "dmg_taken": 3,
                        "outcome": "kill",
                        "xp_earned": 5,
                        "gold_earned": 2,
                    })
                    db.commit(conn)
                    with lock:
                        run_ids.append(run_id)
                except BaseException as exc:
                    with lock:
                        errors.append(exc)
                finally:
                    db.close(conn)

            threads = [
                threading.Thread(target=writer, args=(1, 2)),
                threading.Thread(target=writer, args=(2, 3)),
            ]
            for t in threads:
                t.start()
            for t in threads:
                t.join(timeout=10)

            self.assertEqual(errors, [])
            self.assertEqual(len(run_ids), 2)
            self.assertEqual(len(set(run_ids)), 2)

            conn = db.open_db(db_path)
            try:
                self.assertEqual(conn.execute("SELECT COUNT(*) FROM run").fetchone()[0], 2)
                self.assertEqual(conn.execute("SELECT COUNT(*) FROM tick_snapshot").fetchone()[0], 2)
                self.assertEqual(conn.execute("SELECT COUNT(*) FROM action_log").fetchone()[0], 2)
                self.assertEqual(conn.execute("SELECT COUNT(*) FROM overworld_encounter").fetchone()[0], 2)
                child_run_ids = {
                    row[0]
                    for row in conn.execute(
                        "SELECT run_id FROM tick_snapshot UNION SELECT run_id FROM action_log "
                        "UNION SELECT run_id FROM overworld_encounter"
                    )
                }
                self.assertEqual(child_run_ids, set(run_ids))
            finally:
                db.close(conn)

    def test_host_initialized_db_is_queryable_immediately_after_container_style_insert(self):
        with tempfile.TemporaryDirectory(prefix="shared-runs-live-") as tmp:
            db_path = Path(tmp) / "runs.db"
            host_conn = db.open_db(db_path)
            db.init_schema(host_conn)
            db.close(host_conn)

            container_conn = db.open_db(db_path)
            db.init_schema(container_conn)
            run_id = db.insert_run(
                container_conn,
                seed=100,
                cls="Rogue",
                race="Elf",
                strategy="balanced",
                tuning_label="live-test",
                target_level=15,
                max_ticks=100,
            )
            db.insert_tick_snapshot(container_conn, run_id, 1, _snapshot_state(4))
            db.commit(container_conn)

            reader_conn = db.open_db(db_path)
            try:
                row = reader_conn.execute(
                    "SELECT r.id, r.ended_at, ts.tick_no, ts.level "
                    "FROM run r JOIN tick_snapshot ts ON ts.run_id = r.id "
                    "WHERE r.id = ?",
                    (run_id,),
                ).fetchone()
                self.assertIsNotNone(row)
                self.assertIsNone(row["ended_at"])
                self.assertEqual(row["tick_no"], 1)
                self.assertEqual(row["level"], 4)
            finally:
                db.close(reader_conn)
                db.close(container_conn)


class TestRunnerSharedDbMount(unittest.TestCase):
    def test_worker_mounts_shared_db_directory_and_does_not_require_a_shard(self):
        with tempfile.TemporaryDirectory(prefix="runner-shared-db-") as tmp:
            tmp_path = Path(tmp)
            sq_bin = tmp_path / "sq"
            sq_bin.write_text("fake sq")
            db_path = tmp_path / "runs.db"
            captured_argv: list[str] = []

            def fake_run(argv: list[str], **kwargs: Any) -> subprocess.CompletedProcess[str]:
                captured_argv.extend(argv)
                stdout = json.dumps({
                    "ok": True,
                    "run_id": 42,
                    "ended_reason": "target_reached",
                    "final_state": {"level": 15, "max_hp": 80, "gold": 200},
                    "ticks": 25,
                })
                return subprocess.CompletedProcess(argv, 0, stdout=stdout, stderr="")

            env = {
                "SQ_BIN_HOST": str(sq_bin),
                "SIM_DIR_HOST": str(BALANCE_SIM_DIR),
                "SIM_IMAGE": "shellquest-sim-test",
            }
            with mock.patch.dict(os.environ, env, clear=False), \
                    mock.patch("runner.subprocess.run", side_effect=fake_run):
                result = runner._worker((
                    "Warrior", "Human", "greedy", 9001, "shared-mount",
                    str(db_path), 15, 1, 0, 100, 10, 1,
                ))

            self.assertNotIn("error", result)
            self.assertEqual(result["run_id"], 42)
            self.assertIn("-v", captured_argv)
            self.assertIn(f"{tmp_path.resolve()}:/db", captured_argv)
            shard_arg_index = captured_argv.index("--shard-out") + 1
            self.assertEqual(captured_argv[shard_arg_index], "/db/runs.db")
            self.assertFalse((tmp_path / "_shards").exists())
            self.assertNotIn("/out", " ".join(captured_argv))


if __name__ == "__main__":
    unittest.main(verbosity=2)

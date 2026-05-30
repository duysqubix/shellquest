"""Tests for end-state final_item capture (extract + persist + merge).

Stdlib-only (unittest), mirroring the no-external-deps convention of the
balance simulator. Run from the balance-sim dir:

    python3 -m unittest tests.test_final_items -v
    # or:  python3 tests/test_final_items.py
"""
from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

BALANCE_SIM_DIR = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(BALANCE_SIM_DIR))

from simulator import db  # noqa: E402
from simulator.player import extract_final_items  # noqa: E402


def _crafted_save() -> dict:
    """A run that ended with an equipped weapon + armor, a NULL ring, and a
    3-item inventory (one of which is a potion)."""
    return {
        "character": {
            "name": "Sim-7",
            "class": "Warrior",
            "race": "Orc",
            "weapon": {
                "name": "Logic-Lash", "slot": "Weapon", "power": 14,
                "rarity": "Rare", "enchant_level": 2,
            },
            "armor": {
                "name": "Null-Plate", "slot": "Armor", "power": 9,
                "rarity": "Uncommon", "enchant_level": 0,
            },
            "ring": None,
            "inventory": [
                {"name": "Spare Dagger", "slot": "Weapon", "power": 5,
                 "rarity": "Common", "enchant_level": 0},
                {"name": "Healing Draught", "slot": "Potion", "power": 0,
                 "rarity": "Common", "enchant_level": 0},
                {"name": "Band of Bytes", "slot": "Ring", "power": 7,
                 "rarity": "Rare", "enchant_level": 1},
            ],
        },
    }


class TestExtractFinalItems(unittest.TestCase):
    def test_equipped_and_inventory_rows(self):
        rows = extract_final_items(_crafted_save())

        # 2 equipped (weapon + armor; null ring skipped) + 3 inventory = 5
        self.assertEqual(len(rows), 5)

        equipped = [r for r in rows if r["equipped"] == 1]
        held = [r for r in rows if r["equipped"] == 0]
        self.assertEqual(len(equipped), 2)
        self.assertEqual(len(held), 3)

        # null ring must NOT appear as an equipped row
        self.assertNotIn("Ring", [r["slot"] for r in equipped])

        weapon = next(r for r in equipped if r["slot"] == "Weapon")
        self.assertEqual(weapon["name"], "Logic-Lash")
        self.assertEqual(weapon["rarity"], "Rare")
        self.assertEqual(weapon["power"], 14)
        self.assertEqual(weapon["enchant_level"], 2)

        # the held potion is captured with its own slot, equipped=0
        potion = next(r for r in held if r["slot"] == "Potion")
        self.assertEqual(potion["name"], "Healing Draught")
        self.assertEqual(potion["equipped"], 0)

        # every row carries the full schema field set
        for r in rows:
            self.assertEqual(
                set(r),
                {"slot", "equipped", "name", "rarity", "power", "enchant_level"},
            )

    def test_null_safe_on_empty_character(self):
        self.assertEqual(extract_final_items({}), [])
        self.assertEqual(extract_final_items({"character": {}}), [])
        self.assertEqual(
            extract_final_items({"character": {"weapon": None, "armor": None,
                                               "ring": None, "inventory": []}}),
            [],
        )


class TestFinalItemMergeRoundTrip(unittest.TestCase):
    """Two shards each with a run + final_item rows must merge into the target
    with run_id remapped and all item values intact (mirrors the
    overworld_encounter / item_event merge behavior)."""

    def _make_shard(self, path: Path, seed: int, cls: str, items: list[dict]) -> None:
        conn = db.open_db(path)
        db.init_schema(conn)
        run_id = db.insert_run(
            conn, seed=seed, cls=cls, race="Human", strategy="greedy",
            tuning_label="merge-test", target_level=20, max_ticks=100,
        )
        db.insert_final_items(conn, run_id, items)
        db.commit(conn)
        db.close(conn)

    def test_merge_remaps_run_id_and_preserves_values(self):
        tmp = Path(tempfile.mkdtemp(prefix="final-item-merge-"))
        shard_a = tmp / "a.db"
        shard_b = tmp / "b.db"
        target = tmp / "runs.db"

        items_a = [
            {"slot": "Weapon", "equipped": 1, "name": "Axe of A",
             "rarity": "Epic", "power": 20, "enchant_level": 3},
            {"slot": "Potion", "equipped": 0, "name": "Potion A",
             "rarity": "Common", "power": 0, "enchant_level": 0},
        ]
        items_b = [
            {"slot": "Armor", "equipped": 1, "name": "Mail of B",
             "rarity": "Rare", "power": 11, "enchant_level": 1},
        ]
        self._make_shard(shard_a, seed=1, cls="Warrior", items=items_a)
        self._make_shard(shard_b, seed=2, cls="Wizard", items=items_b)

        merged = db.merge_dbs(target, [shard_a, shard_b])
        self.assertEqual(merged, 2)

        conn = db.open_db(target)
        rows = [dict(r) for r in conn.execute(
            "SELECT r.seed, r.class, fi.run_id, fi.slot, fi.equipped, fi.name, "
            "fi.rarity, fi.power, fi.enchant_level "
            "FROM final_item fi JOIN run r ON r.id = fi.run_id "
            "ORDER BY r.seed, fi.equipped DESC, fi.slot"
        )]

        # every final_item row points at a real (remapped) run id
        run_ids = {r["run_id"] for r in rows}
        valid_ids = {r[0] for r in conn.execute("SELECT id FROM run")}
        self.assertTrue(run_ids.issubset(valid_ids))

        self.assertEqual(len(rows), 3)
        axe = next(r for r in rows if r["name"] == "Axe of A")
        self.assertEqual(axe["seed"], 1)
        self.assertEqual(axe["rarity"], "Epic")
        self.assertEqual(axe["power"], 20)
        self.assertEqual(axe["enchant_level"], 3)
        self.assertEqual(axe["equipped"], 1)

        mail = next(r for r in rows if r["name"] == "Mail of B")
        self.assertEqual(mail["seed"], 2)
        self.assertEqual(mail["slot"], "Armor")

        # the two shards' rows landed on two DISTINCT remapped run ids
        self.assertEqual(len({r["run_id"] for r in rows}), 2)
        db.close(conn)


if __name__ == "__main__":
    unittest.main(verbosity=2)

from __future__ import annotations

import sys
import unittest
import importlib
from pathlib import Path

BALANCE_SIM_DIR = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(BALANCE_SIM_DIR))

starter_save = importlib.import_module("simulator.player").starter_save


CLASS_BASE = {
    "Warrior": (16, 8, 6),
    "Wizard": (6, 8, 16),
    "Rogue": (8, 16, 6),
    "Ranger": (10, 14, 6),
    "Necromancer": (6, 6, 18),
}
RACE_BONUS = {
    "Human": (1, 1, 1),
    "Elf": (0, 2, 2),
    "Dwarf": (3, 0, 1),
    "Orc": (3, 1, -1),
    "Goblin": (0, 3, 1),
}
FIRST_SUBCLASS = {
    "Warrior": ("Berserker", (3, 1, 0)),
    "Wizard": ("Archmage", (0, 1, 3)),
    "Rogue": ("Assassin", (1, 3, 0)),
    "Ranger": ("Beastmaster", (2, 2, 0)),
    "Necromancer": ("Lich", (0, 0, 4)),
}


def expected_xp_to_next(level: int) -> int:
    if level <= 10:
        return level * 30 + 40
    if level <= 30:
        return level * 32 + 30
    if level <= 60:
        return level * 45 + 80
    if level <= 100:
        return level * 80 + 200
    if level <= 130:
        return level * 120 + 400
    return level * 170 + 800


def expected_stats(cls: str, race: str, level: int, prestige: int = 0) -> dict[str, int]:
    base_str, base_dex, base_int = CLASS_BASE[cls]
    race_str, race_dex, race_int = RACE_BONUS[race]
    level_gain = level - 1
    subclass_str, subclass_dex, subclass_int = (0, 0, 0)
    if prestige:
        _, subclass_bonus = FIRST_SUBCLASS[cls]
        subclass_str, subclass_dex, subclass_int = subclass_bonus
    prestige_bonus = prestige * 2
    level_one_str = base_str + race_str + prestige_bonus + subclass_str
    level_one_dex = base_dex + race_dex + prestige_bonus + subclass_dex
    level_one_int = base_int + race_int + prestige_bonus + subclass_int
    return {
        "level": level,
        "xp": 0,
        "xp_to_next": expected_xp_to_next(level),
        "strength": level_one_str + level_gain,
        "dexterity": level_one_dex + level_gain,
        "intelligence": level_one_int + level_gain,
        "max_hp": 20 + level_one_str * 2 + prestige * 10 + 5 * level_gain,
    }


class TestStarterSaveStartLevel(unittest.TestCase):
    def assert_seeded_character(self, cls: str, race: str, level: int) -> None:
        character = starter_save(cls, race, seed=123, start_level=level)["character"]
        expected = expected_stats(cls, race, level)
        for key, value in expected.items():
            self.assertEqual(character[key], value, f"{cls} {race} L{level} {key}")
        self.assertEqual(character["hp"], expected["max_hp"])
        self.assertIsNone(character["weapon"])
        self.assertIsNone(character["armor"])
        self.assertIsNone(character["ring"])

    def test_level_one_matches_existing_starter_behavior(self):
        character = starter_save("Warrior", "Human", seed=123, start_level=1)["character"]
        self.assertEqual(character["level"], 1)
        self.assertEqual(character["xp"], 0)
        self.assertEqual(character["xp_to_next"], 70)
        self.assertEqual(character["strength"], 17)
        self.assertEqual(character["dexterity"], 9)
        self.assertEqual(character["intelligence"], 7)
        self.assertEqual(character["max_hp"], 54)
        self.assertEqual(character["hp"], 54)
        self.assertEqual(character["gold"], 10)

    def test_seeded_levels_match_rust_level_up_core_math(self):
        cases = [
            ("Warrior", "Human", 30),
            ("Wizard", "Elf", 60),
            ("Rogue", "Dwarf", 100),
            ("Necromancer", "Goblin", 150),
        ]
        for cls, race, level in cases:
            with self.subTest(cls=cls, race=race, level=level):
                self.assert_seeded_character(cls, race, level)

    def test_start_prestige_zero_matches_current_seeded_level_behavior(self):
        character = starter_save(
            "Wizard", "Elf", seed=123, start_level=60, start_prestige=0,
        )["character"]
        expected = expected_stats("Wizard", "Elf", 60, 0)
        for key, value in expected.items():
            self.assertEqual(character[key], value)
        self.assertEqual(character["prestige"], 0)
        self.assertEqual(character["total_prestiges"], 0)
        self.assertIsNone(character["subclass"])

    def test_start_prestige_matches_rust_prestige_then_level_math(self):
        cases = [
            ("Warrior", "Human", 150, 3),
            ("Wizard", "Elf", 100, 2),
            ("Necromancer", "Goblin", 150, 3),
        ]
        for cls, race, level, prestige in cases:
            with self.subTest(cls=cls, race=race, level=level, prestige=prestige):
                character = starter_save(
                    cls, race, seed=123, start_level=level, start_prestige=prestige,
                )["character"]
                expected = expected_stats(cls, race, level, prestige)
                for key, value in expected.items():
                    self.assertEqual(character[key], value)
                self.assertEqual(character["hp"], expected["max_hp"])
                self.assertEqual(character["prestige"], prestige)
                self.assertEqual(character["total_prestiges"], prestige)
                self.assertEqual(character["subclass"], FIRST_SUBCLASS[cls][0])


if __name__ == "__main__":
    unittest.main(verbosity=2)
